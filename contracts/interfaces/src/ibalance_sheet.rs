//! Mirrors `contracts/interfaces/IBalanceSheet.sol`.
use soroban_sdk::{contractclient, Address, Env, Symbol, I256};

use shared::types::FileValue;

/// The protocol's treasury and debt manager.
#[contractclient(name = "BalanceSheetClient")]
pub trait IBalanceSheet {
    fn file(env: Env, caller: Address, what: Symbol, value: FileValue);
    fn fess(env: Env, caller: Address, tab: I256);
    fn flog(env: Env, era: u64);
    fn heal(env: Env, caller: Address, rad: I256);
    fn suck(env: Env, caller: Address, kpr: Address, rad: I256);
    fn distribute_surplus(env: Env) -> I256;
    fn hump_target(env: Env) -> I256;
    fn vault_engine(env: Env) -> Address;
    fn hump_floor(env: Env) -> I256;
    fn hump_rate(env: Env) -> I256;
    fn wait(env: Env) -> I256;
    fn total_queued_sin(env: Env) -> I256;
    fn sin(env: Env, era: u64) -> I256;
    fn reserve_accounting(env: Env) -> Option<Address>;
    fn buyback_receiver(env: Env) -> Option<Address>;
}
