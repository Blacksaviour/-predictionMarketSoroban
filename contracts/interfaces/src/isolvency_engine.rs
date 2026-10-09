//! Mirrors `contracts/interfaces/ISolvencyEngine.sol`.
use soroban_sdk::{contractclient, Address, BytesN, Env, Symbol, Vec, I256};

use shared::types::FileValue;

/// The guardian that enforces the solvency invariant.
#[contractclient(name = "SolvencyEngineClient")]
pub trait ISolvencyEngine {
    fn file(env: Env, caller: Address, what: Symbol, value: FileValue);
    fn add_volatile_ilk(env: Env, caller: Address, ilk_id: BytesN<32>);
    fn remove_volatile_ilk(env: Env, caller: Address, ilk_id: BytesN<32>);
    fn check_invariant(env: Env) -> (I256, I256);
    fn is_breached(env: Env) -> bool;
    fn breach_threshold(env: Env) -> I256;
    fn worst_case_loss(env: Env) -> I256;
    fn vault_engine(env: Env) -> Address;
    fn reserve_accounting(env: Env) -> Address;
    fn stress_markdown(env: Env) -> I256;
    fn stress_depth(env: Env) -> I256;
    fn reserve_factor(env: Env) -> I256;
    fn exposure_cap(env: Env) -> I256;
    fn breached(env: Env) -> bool;
    fn osm(env: Env) -> Option<Address>;
    fn external_exposure(env: Env) -> Option<Address>;
    fn volatile_ilks(env: Env) -> Vec<BytesN<32>>;
    fn is_volatile(env: Env, ilk_id: BytesN<32>) -> bool;
}
