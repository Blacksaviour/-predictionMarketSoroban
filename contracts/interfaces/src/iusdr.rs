//! Mirrors `contracts/interfaces/IUSDR.sol` extended with the ERC-20 surface of the token.
use soroban_sdk::{contractclient, Address, Env, String, I256};

/// The Rain Dollar stablecoin, plus the ERC-20 surface carried by OpenZeppelin's `ERC20` in Solidity.
///
/// `EIP-2612 permit` has no Soroban equivalent (signatures are handled by the host's authorization entries),
/// so it is intentionally omitted.
#[contractclient(name = "UsdrClient")]
pub trait IUsdr {
    /* ========================== MINT / BURN ========================== */
    fn mint(env: Env, caller: Address, to: Address, amount: I256);
    fn burn(env: Env, sender: Address, from: Address, amount: I256);

    /* ========================== ERC-20 ========================== */
    fn name(env: Env) -> String;
    fn symbol(env: Env) -> String;
    fn decimals(env: Env) -> u32;
    fn total_supply(env: Env) -> I256;
    fn balance_of(env: Env, account: Address) -> I256;
    fn allowance(env: Env, owner: Address, spender: Address) -> I256;
    fn approve(env: Env, owner: Address, spender: Address, amount: I256);
    fn transfer(env: Env, from: Address, to: Address, amount: I256);
    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: I256);
}
