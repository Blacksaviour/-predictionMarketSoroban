//! Fixed-point scalars, role identifiers and ilk helpers.
//!
//! Port of `contracts/shared/Constants.sol`.
//!
//! The Solidity source uses 256-bit integers throughout. Soroban's widest native integer is `i128`, which cannot
//! represent `_RAD` (10^45), so every ledger quantity in this port is a [`soroban_sdk::I256`] (signed 256-bit).
//! All values on the ledger are non-negative and far below `I256::max_value()`, so the signed representation is
//! faithful to the unsigned Solidity arithmetic while keeping the exact MakerDAO decimal semantics.

use soroban_sdk::{symbol_short, BytesN, Env, Symbol, I256};

/// Core authorization role. Mirrors `_WARD_ROLE` (`keccak256("WARD_ROLE")` in Solidity).
pub const WARD: Symbol = symbol_short!("WARD");

/// Grants the right to burn USDR from any address without an allowance. Mirrors `_BURNER_ROLE`.
pub const BURNER: Symbol = symbol_short!("BURNER");

/// Grants the right to update the committed escrow. Mirrors `_COMMITTER_ROLE`.
pub const COMMITTER: Symbol = symbol_short!("COMMITTER");

/// Grants price read access to the Oracle Security Module. Mirrors `_READER_ROLE`.
pub const READER: Symbol = symbol_short!("READER");

/// Grants the right to record reserve movements. Mirrors `_RECORDER_ROLE`.
pub const RECORDER: Symbol = symbol_short!("RECORDER");

/// Fixed point scalar with 18 decimals. Mirrors `_WAD`.
pub fn wad(env: &Env) -> I256 {
    I256::from_i128(env, 1_000_000_000_000_000_000)
}

/// Fixed point scalar with 27 decimals. Mirrors `_RAY`.
pub fn ray(env: &Env) -> I256 {
    I256::from_i128(env, 1_000_000_000_000_000_000_000_000_000)
}

/// Fixed point scalar with 45 decimals. Mirrors `_RAD`.
pub fn rad(env: &Env) -> I256 {
    I256::from_i128(env, 10).pow(45)
}

/// Builds a `bytes32`-style collateral identifier from an ASCII name.
///
/// Solidity literals such as `"USDR"` are converted to `bytes32` by right-padding with zero bytes. This helper
/// reproduces that layout exactly, so identifiers stay stable for indexers migrating from the EVM deployment.
pub fn ilk_id(env: &Env, name: &str) -> BytesN<32> {
    let mut buf = [0u8; 32];
    let bytes = name.as_bytes();
    let n = if bytes.len() > 32 { 32 } else { bytes.len() };
    let mut i = 0;
    while i < n {
        buf[i] = bytes[i];
        i += 1;
    }
    BytesN::from_array(env, &buf)
}

/// Reserved ilk identifier for USDR itself in the Collateral Adapter. Mirrors `_USDR_ILK`.
pub fn usdr_ilk(env: &Env) -> BytesN<32> {
    ilk_id(env, "USDR")
}
