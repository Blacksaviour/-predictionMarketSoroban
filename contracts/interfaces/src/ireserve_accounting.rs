//! Mirrors `contracts/interfaces/IReserveAccounting.sol`.
use soroban_sdk::{contractclient, Address, Env, I256};

/// The bookkeeper for the protocol's stable dollars.
#[contractclient(name = "ReserveAccountingClient")]
pub trait IReserveAccounting {
    fn record_increase(env: Env, caller: Address, wad: I256);
    fn record_decrease(env: Env, caller: Address, wad: I256);
    fn update_committed_escrow(env: Env, caller: Address, wad: I256);
    fn free_slack(env: Env) -> I256;
    fn total_reserve(env: Env) -> I256;
    fn committed_escrow(env: Env) -> I256;
}
