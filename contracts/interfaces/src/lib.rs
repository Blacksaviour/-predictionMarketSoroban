#![no_std]
//! Cross-contract interfaces for the Rain USDR Soroban port.
//!
//! Every module mirrors one Solidity file under `contracts/interfaces`. The traits are annotated with
//! `#[contractclient]`, which generates a typed client (`VaultEngineClient`, `UsdrClient`, ...) that contracts
//! and tests use to call a deployed instance.
//!
//! Naming notes, forced by Rust/Soroban having no function overloading:
//! * `file(bytes32 what, uint256|address data)` and `file(bytes32 ilkId, bytes32 what, ...)` become
//!   `file(...)` / `file_ilk(...)` taking a [`shared::types::FileValue`].
//! * Solidity getters that return a tuple return the corresponding `shared::types` struct.

pub mod ibalance_sheet;
pub mod icircuit_breaker;
pub mod icollateral_adapter;
pub mod idutch_auction;
pub mod idutch_auction_callee;
pub mod iend;
pub mod iexternal_exposure;
pub mod igovernor;
pub mod iliquidation_trigger;
pub mod ioracle_security_module;
pub mod iprice_converter;
pub mod iprice_curve;
pub mod iprice_source;
pub mod iprice_stability_module;
pub mod ireserve_accounting;
pub mod isolvency_engine;
pub mod iusdr;
pub mod ivault_engine;

pub use shared::types::{
    AdapterIlk, Change, Feed, FileValue, Ilk, IlkId, IlkLiquidation, IlkOracle, OsmIlk, PsmIlk,
    Sale, Urn,
};
