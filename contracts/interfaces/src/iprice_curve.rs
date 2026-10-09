//! Mirrors `contracts/interfaces/IPriceCurve.sol`.
use soroban_sdk::{contractclient, Address, Env, Symbol, I256};

use shared::types::FileValue;

/// A pure calculator for the descending auction price.
#[contractclient(name = "PriceCurveClient")]
pub trait IPriceCurve {
    fn file(env: Env, caller: Address, what: Symbol, value: FileValue);
    fn price(env: Env, top: I256, dur: I256) -> I256;
    fn tau(env: Env) -> I256;
}
