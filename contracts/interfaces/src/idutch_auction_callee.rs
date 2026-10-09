//! Mirrors `contracts/interfaces/IDutchAuctionCallee.sol`.
use soroban_sdk::{contractclient, Address, Bytes, Env, I256};

/// The callback invoked by a Dutch auction purchase to support flash-loan-style buying.
#[contractclient(name = "DutchAuctionCalleeClient")]
pub trait IDutchAuctionCallee {
    fn clipper_call(env: Env, sender: Address, owe: I256, slice: I256, data: Bytes);
}
