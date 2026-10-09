//! Mirrors `contracts/interfaces/IExternalExposure.sol`.
use soroban_sdk::{contractclient, Env, I256};

/// An external reporter of additional protocol exposure (prediction market layer).
#[contractclient(name = "ExternalExposureClient")]
pub trait IExternalExposure {
    fn reported_exposure(env: Env) -> I256;
}
