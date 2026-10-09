//! Shared events.
//!
//! Port of `contracts/shared/Events.sol`. The `#[contractevent]` macro (Soroban SDK 27) replaces the raw
//! `env.events().publish` call, keeps the topic layout identical across the contracts that mirror the Solidity
//! `event Cage()` and also registers the event in the contract spec for indexers.

use soroban_sdk::{contractevent, BytesN};

/// Emitted when a contract is shut down. Mirrors `event Cage()`.
#[contractevent]
#[derive(Clone)]
pub struct Cage {}

/// Emitted when a per-ilk component is shut down. Mirrors `event Cage(bytes32 indexed ilkId)`.
#[contractevent]
#[derive(Clone)]
pub struct CageIlk {
    /// Identifier of the ilk that was shut down.
    #[topic]
    pub ilk_id: BytesN<32>,
}
