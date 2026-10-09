//! Mirrors `contracts/interfaces/ICircuitBreaker.sol`.
use soroban_sdk::{contractclient, Address, BytesN, Env, Symbol, I256};

use shared::types::FileValue;

/// Slows liquidations when the oracle price moves suspiciously fast.
#[contractclient(name = "CircuitBreakerClient")]
pub trait ICircuitBreaker {
    fn file(env: Env, caller: Address, what: Symbol, value: FileValue);
    fn check(env: Env);
    fn obs_count(env: Env) -> u32;
    fn ilk_id(env: Env) -> BytesN<32>;
    fn pip(env: Env) -> Address;
    fn threshold(env: Env) -> I256;
    fn calm_period(env: Env) -> I256;
    fn obs_interval(env: Env) -> I256;
    fn trend_price(env: Env) -> I256;
    fn activated_at(env: Env) -> I256;
    fn last_obs_timestamp(env: Env) -> I256;
    fn active(env: Env) -> bool;
}
