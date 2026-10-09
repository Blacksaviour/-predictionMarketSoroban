#![no_std]
//! # PriceCurve
//!
//! Port of `contracts/liquidation/PriceCurve.sol` (Maker's Abacus).
//!
//! A pure calculator. Given an auction's starting price, its start time and how long it should run, it returns
//! the current price at any moment. USDR uses a straight-line decline: the price falls steadily from the start
//! value to zero over the auction's lifetime.

use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, panic_with_error, Address, Env, Symbol,
    I256,
};

use shared::access::{grant_role, require_role, set_role_admin};
use shared::constants::WARD;
use shared::errors::Error;
use shared::math::{self, zero};
use shared::types::FileValue;

/// Storage keys.
#[contracttype]
pub enum DataKey {
    /// Auction lifetime in seconds.
    Tau,
}

/// Emitted when a numeric parameter is updated. Mirrors `event File(bytes32,uint256)`.
#[contractevent]
#[derive(Clone)]
pub struct FileNum {
    #[topic]
    pub what: Symbol,
    pub data: I256,
}

#[contract]
pub struct PriceCurve;

#[contractimpl]
impl PriceCurve {
    /// Authorizes the deployer.
    pub fn __constructor(env: Env, admin: Address) {
        admin.require_auth();

        set_role_admin(&env, &WARD, &WARD);
        grant_role(&env, &WARD, &admin);
    }

    /* ========================== VIEWS ========================== */

    /// Returns the auction lifetime in seconds.
    pub fn tau(env: Env) -> I256 {
        env.storage()
            .instance()
            .get(&DataKey::Tau)
            .unwrap_or_else(|| zero(&env))
    }

    /// Returns the current auction price. Mirrors `price(uint256 top, uint256 dur)`.
    ///
    /// `dur` is the time elapsed since the auction started.
    pub fn price(env: Env, top: I256, dur: I256) -> I256 {
        let tau = PriceCurve::tau(env.clone());

        // Past the lifetime, the price is zero and never negative.
        if dur >= tau {
            return zero(&env);
        }

        // Current price = starting price * (1 - time elapsed / lifetime).
        top.mul(&math::sub(&env, &tau, &dur)).div(&tau)
    }

    /* ========================== ADMINISTRATION ========================== */

    /// Updates a numeric parameter. Mirrors `file(bytes32,uint256)`.
    pub fn file(env: Env, caller: Address, what: Symbol, value: FileValue) {
        caller.require_auth();
        require_role(&env, &WARD, &caller);

        if what != Symbol::new(&env, "tau") {
            panic_with_error!(&env, Error::UnrecognizedParameter);
        }

        let data = match value {
            FileValue::Num(data) => data,
            FileValue::Addr(_) => panic_with_error!(&env, Error::UnrecognizedParameter),
        };

        env.storage().instance().set(&DataKey::Tau, &data);

        FileNum { what, data }.publish(&env);
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

#[cfg(test)]
mod test;
