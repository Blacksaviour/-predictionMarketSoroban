//! Mirrors `contracts/interfaces/IGovernor.sol`.
use soroban_sdk::{contractclient, Address, BytesN, Env, Symbol, Val, Vec};

use shared::types::Change;

/// The timelocked parameter changer and emergency pause.
#[contractclient(name = "GovernorClient")]
pub trait IGovernor {
    fn schedule(env: Env, caller: Address, target: Address, fn_name: Symbol, args: Vec<Val>)
        -> u64;
    fn execute(env: Env, id: u64) -> Val;
    fn cancel(env: Env, caller: Address, id: u64);
    fn pause(env: Env, caller: Address, scope: BytesN<32>);
    fn unpause(env: Env, sender: Address);
    fn pause_max(env: Env) -> u64;
    fn pause_scope(env: Env) -> BytesN<32>;
    fn delay(env: Env) -> u64;
    fn paused_at(env: Env) -> u64;
    fn change_count(env: Env) -> u64;
    fn paused(env: Env) -> bool;
    fn changes(env: Env, id: u64) -> Change;
}
