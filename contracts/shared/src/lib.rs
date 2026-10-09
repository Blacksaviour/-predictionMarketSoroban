#![no_std]
//! Shared building blocks for the Rain USDR Soroban port.
//!
//! This crate mirrors the Solidity `contracts/shared` and `contracts/libraries` directories plus the parts of
//! `contracts/interfaces` that describe data structures used across contracts.
//!
//! * [`constants`]  <-> `shared/Constants.sol`
//! * [`errors`]     <-> `shared/Errors.sol`
//! * [`events`]     <-> `shared/Events.sol`
//! * [`math`]       <-> `libraries/Math.sol` and `shared/Globals.sol`
//! * [`types`]      <-> the `struct` definitions of the `I*.sol` interfaces
//! * [`access`]     <-> OpenZeppelin `AccessControl` (role management)

pub mod access;
pub mod constants;
pub mod errors;
pub mod events;
pub mod math;
pub mod storage;
pub mod types;
