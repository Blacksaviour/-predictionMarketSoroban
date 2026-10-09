//! Mirrors `contracts/interfaces/IPriceSource.sol`.
use soroban_sdk::{contractclient, BytesN, Env};

/// A raw price source consumed by the Oracle Security Module.
#[contractclient(name = "PriceSourceClient")]
pub trait IPriceSource {
    fn peek(env: Env) -> (BytesN<32>, bool);
}
