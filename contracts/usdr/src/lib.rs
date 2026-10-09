#![no_std]
//! # USDR — Rain Dollar
//!
//! Port of `contracts/token/USDR.sol`.
//!
//! A standard, transferable digital dollar that can only be minted or burned by authorized system contracts
//! (the Collateral Adapter and the Peg Stability Module). No administrator can create USDR out of nothing.
//!
//! ## Soroban adaptations
//! * `EIP-2612 permit` is dropped: Soroban handles signatures through the host's authorization entries.
//! * `msg.sender` becomes an explicit authenticated `Address` argument (`from` for `transfer`, `spender` for
//!   `transfer_from`, `sender` for `burn`) because Soroban exposes no ambient caller.
//! * `address(0)` does not exist on Soroban, so `mint`/`burn` events carry `Option<Address>` for the zero leg.

use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, panic_with_error, Address, Env, String,
    Symbol, I256,
};

use shared::access::{grant_role, has_role, require_role, set_role_admin};
use shared::constants::{BURNER, WARD};
use shared::errors::Error;
use shared::math;
use shared::storage;

/// Storage keys.
#[contracttype]
pub enum DataKey {
    /// Total supply of USDR [wad].
    TotalSupply,
    /// Balance of an account [wad].
    Balance(Address),
    /// Allowance granted by `owner` to `spender` [wad].
    Allowance(Address, Address),
}

/// Emitted on any balance movement. Mirrors the ERC-20 `Transfer` event.
#[contractevent]
#[derive(Clone)]
pub struct Transfer {
    /// Source account. `None` for mints.
    #[topic]
    pub from: Option<Address>,
    /// Destination account. `None` for burns.
    #[topic]
    pub to: Option<Address>,
    /// Amount moved [wad].
    pub value: I256,
}

/// Emitted when an allowance is set. Mirrors the ERC-20 `Approval` event.
#[contractevent]
#[derive(Clone)]
pub struct Approval {
    /// Owner of the funds.
    #[topic]
    pub owner: Address,
    /// Account allowed to spend.
    #[topic]
    pub spender: Address,
    /// Allowance amount [wad].
    pub value: I256,
}

#[contract]
pub struct Usdr;

#[contractimpl]
impl Usdr {
    /* ========================== CONSTRUCTOR ========================== */

    /// Initializes the token and authorizes the deployer, which later grants authorization to the Collateral
    /// Adapter and the Peg Stability Module during deployment. Mirrors the Solidity constructor.
    pub fn __constructor(env: Env, admin: Address) {
        admin.require_auth();

        set_role_admin(&env, &BURNER, &WARD);
        set_role_admin(&env, &WARD, &WARD);

        grant_role(&env, &WARD, &admin);

        env.storage()
            .instance()
            .set(&DataKey::TotalSupply, &math::zero(&env));
    }

    /* ========================== METADATA ========================== */

    /// Token name. Mirrors `ERC20("Rain Dollar", "USDR")`.
    pub fn name(env: Env) -> String {
        String::from_str(&env, "Rain Dollar")
    }

    /// Token symbol. Mirrors `ERC20("Rain Dollar", "USDR")`.
    pub fn symbol(env: Env) -> String {
        String::from_str(&env, "USDR")
    }

    /// Token decimals (18).
    pub fn decimals(_env: Env) -> u32 {
        18
    }

    /* ========================== VIEWS ========================== */

    /// Returns the total USDR in existence [wad].
    pub fn total_supply(env: Env) -> I256 {
        read_supply(&env)
    }

    /// Returns the balance of `account` [wad].
    pub fn balance_of(env: Env, account: Address) -> I256 {
        read_balance(&env, &account)
    }

    /// Returns the allowance `owner` has granted to `spender` [wad].
    pub fn allowance(env: Env, owner: Address, spender: Address) -> I256 {
        storage::get(&env, &DataKey::Allowance(owner, spender)).unwrap_or_else(|| math::zero(&env))
    }
}

#[contractimpl]
impl Usdr {
    /* ========================== ERC-20 ========================== */

    /// Approves `spender` to spend `amount` of the caller's USDR.
    pub fn approve(env: Env, owner: Address, spender: Address, amount: I256) {
        owner.require_auth();

        storage::set(
            &env,
            &DataKey::Allowance(owner.clone(), spender.clone()),
            &amount,
        );

        Approval {
            owner,
            spender,
            value: amount,
        }
        .publish(&env);
    }

    /// Transfers `amount` from `from` to `to`. Mirrors `transfer(to, amount)` with `msg.sender == from`.
    pub fn transfer(env: Env, from: Address, to: Address, amount: I256) {
        from.require_auth();
        do_transfer(&env, &from, &to, amount);
    }

    /// Transfers `amount` from `from` to `to` using the caller's allowance.
    /// Mirrors `transferFrom(from, to, amount)` with `msg.sender == spender`.
    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: I256) {
        spender.require_auth();
        spend_allowance(&env, &from, &spender, &amount);
        do_transfer(&env, &from, &to, amount);
    }

    /* ========================== MINT / BURN ========================== */

    /// Mints `amount` to `to`. Restricted to the Ward role. Mirrors `mint(address,uint256)`.
    pub fn mint(env: Env, caller: Address, to: Address, amount: I256) {
        caller.require_auth();
        require_role(&env, &WARD, &caller);

        if amount <= math::zero(&env) {
            panic_with_error!(&env, Error::InvalidAmount);
        }

        do_mint(&env, &to, amount);
    }

    /// Burns `amount` from `from`. Mirrors `burn(address,uint256)`.
    ///
    /// When `from` is not the caller and the caller is not a Burner, the caller's allowance is spent.
    pub fn burn(env: Env, sender: Address, from: Address, amount: I256) {
        sender.require_auth();

        // Zero-amount burns are rejected rather than emitting no-op Transfer events that pollute indexers.
        if amount <= math::zero(&env) {
            panic_with_error!(&env, Error::InvalidAmount);
        }

        if from != sender && !has_role(&env, &BURNER, &sender) {
            spend_allowance(&env, &from, &sender, &amount);
        }

        do_burn(&env, &from, amount);
    }

    /* ========================== ROLE MANAGEMENT ========================== */

    /// Grants `role` to `account`. Mirrors OpenZeppelin `AccessControl.grantRole`.
    pub fn grant(env: Env, caller: Address, role: Symbol, account: Address) {
        shared::access::grant_role_checked(&env, &role, &caller, &account);
    }

    /// Revokes `role` from `account`. Mirrors OpenZeppelin `AccessControl.revokeRole`.
    pub fn revoke(env: Env, caller: Address, role: Symbol, account: Address) {
        shared::access::revoke_role_checked(&env, &role, &caller, &account);
    }

    /// Returns whether `account` holds `role`. Mirrors OpenZeppelin `AccessControl.hasRole`.
    pub fn has(env: Env, role: Symbol, account: Address) -> bool {
        shared::access::has_role(&env, &role, &account)
    }
}

/* ========================== INTERNAL HELPERS ========================== */

/// Reads the total supply, treating a missing entry as zero.
fn read_supply(env: &Env) -> I256 {
    env.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or_else(|| math::zero(env))
}

/// Writes the total supply.
fn write_supply(env: &Env, value: &I256) {
    env.storage().instance().set(&DataKey::TotalSupply, value);
}

/// Reads an account balance, treating a missing entry as zero.
fn read_balance(env: &Env, account: &Address) -> I256 {
    storage::get(env, &DataKey::Balance(account.clone())).unwrap_or_else(|| math::zero(env))
}

/// Writes an account balance.
fn write_balance(env: &Env, account: &Address, value: &I256) {
    storage::set(env, &DataKey::Balance(account.clone()), value);
}

/// Moves `amount` from `from` to `to` and emits `Transfer`.
fn do_transfer(env: &Env, from: &Address, to: &Address, amount: I256) {
    let from_balance = read_balance(env, from);
    if from_balance < amount {
        panic_with_error!(env, Error::InsufficientBalance);
    }

    write_balance(env, from, &math::sub(env, &from_balance, &amount));
    write_balance(env, to, &math::add(env, &read_balance(env, to), &amount));

    Transfer {
        from: Some(from.clone()),
        to: Some(to.clone()),
        value: amount,
    }
    .publish(env);
}

/// Spends `amount` from `owner`'s allowance to `spender`, mirroring `_spendAllowance`.
fn spend_allowance(env: &Env, owner: &Address, spender: &Address, amount: &I256) {
    let key = DataKey::Allowance(owner.clone(), spender.clone());
    let current = storage::get(env, &key).unwrap_or_else(|| math::zero(env));

    if current < *amount {
        panic_with_error!(env, Error::InsufficientAllowance);
    }

    storage::set(env, &key, &math::sub(env, &current, amount));
}

/// Mints `amount` to `to`, mirroring `_mint`.
fn do_mint(env: &Env, to: &Address, amount: I256) {
    write_supply(env, &math::add(env, &read_supply(env), &amount));
    write_balance(env, to, &math::add(env, &read_balance(env, to), &amount));

    Transfer {
        from: None,
        to: Some(to.clone()),
        value: amount,
    }
    .publish(env);
}

/// Burns `amount` from `from`, mirroring `_burn`.
fn do_burn(env: &Env, from: &Address, amount: I256) {
    let from_balance = read_balance(env, from);
    if from_balance < amount {
        panic_with_error!(env, Error::InsufficientBalance);
    }

    write_balance(env, from, &math::sub(env, &from_balance, &amount));
    write_supply(env, &math::sub(env, &read_supply(env), &amount));

    Transfer {
        from: Some(from.clone()),
        to: None,
        value: amount,
    }
    .publish(env);
}

#[cfg(test)]
mod test;
