//! Math helpers for mixed signed and unsigned arithmetic.
//!
//! Port of `contracts/libraries/Math.sol` and the `_revert` helper from `contracts/shared/Globals.sol`.
//!
//! The Solidity library operates on `uint256`/`int256`. In this port every quantity is a signed 256-bit
//! [`I256`] (see [`crate::constants`]). "Unsigned" helpers therefore additionally assert that the result is
//! non-negative, reproducing the `uint256` return type of the original. Overflow of the *256-bit* range is
//! reported by the host as a trap, exactly like the Solidity custom errors, so no extra check is required for
//! the upper bound.
//!
//! All operands are passed by reference: `I256` is a host object that is `Clone` but not `Copy`.

use soroban_sdk::{panic_with_error, Env, I256};

use crate::errors::Error;

/// Returns the zero constant for the current environment.
#[inline]
pub fn zero(env: &Env) -> I256 {
    I256::from_i32(env, 0)
}

/// Convenience constructor from an `i128` literal.
#[inline]
pub fn i(env: &Env, v: i128) -> I256 {
    I256::from_i128(env, v)
}

/// Convenience constructor from a `u64` (vault ids, timestamps, counters).
#[inline]
pub fn from_u64(env: &Env, v: u64) -> I256 {
    if v > i128::MAX as u64 {
        panic_with_error!(env, Error::Overflow);
    }
    I256::from_i128(env, v as i128)
}

/// Narrows an `I256` to `u64`, trapping when the value does not fit.
#[inline]
pub fn to_u64(env: &Env, v: &I256) -> u64 {
    match v.to_i128() {
        Some(x) if x >= 0 && x <= u64::MAX as i128 => x as u64,
        _ => panic_with_error!(env, Error::Overflow),
    }
}

/// Narrows an `I256` to `i128`, trapping when the value does not fit.
#[inline]
pub fn to_i128(env: &Env, v: &I256) -> i128 {
    match v.to_i128() {
        Some(x) => x,
        None => panic_with_error!(env, Error::Overflow),
    }
}

/// Reverts unless `value` is strictly positive.
#[inline]
pub fn require_positive(env: &Env, value: &I256) {
    if *value <= zero(env) {
        panic_with_error!(env, Error::InvalidAmount);
    }
}

/// Returns `-v`. Mirrors the unary minus applied to the signed values in the Solidity source.
#[inline]
pub fn neg(env: &Env, v: &I256) -> I256 {
    zero(env).sub(v)
}

/// Adds a signed integer to an unsigned integer, reverting on over/underflow. Mirrors `Math.add(uint256,int256)`.
#[inline]
pub fn add(env: &Env, x: &I256, y: &I256) -> I256 {
    let z = x.add(y);
    if z < zero(env) {
        panic_with_error!(env, Error::AddUnderflow);
    }
    z
}

/// Subtracts a signed integer from an unsigned integer, reverting on over/underflow.
/// Mirrors `Math.sub(uint256,int256)`.
#[inline]
pub fn sub(env: &Env, x: &I256, y: &I256) -> I256 {
    let z = x.sub(y);
    if z < zero(env) {
        panic_with_error!(env, Error::SubUnderflow);
    }
    z
}

/// Multiplies an unsigned integer by a signed integer, reverting on overflow.
/// Mirrors `Math.mul(uint256,int256)` (the result is signed, so it may be negative).
#[inline]
pub fn mul(env: &Env, x: &I256, y: &I256) -> I256 {
    if *x < zero(env) {
        panic_with_error!(env, Error::MulOverflow);
    }
    x.mul(y)
}

/// Multiplies two unsigned integers, reverting on overflow.
/// Mirrors `Math.umul(uint256,uint256)`; the result must be non-negative.
#[inline]
pub fn umul(env: &Env, x: &I256, y: &I256) -> I256 {
    if *x < zero(env) || *y < zero(env) {
        panic_with_error!(env, Error::MulOverflow);
    }
    x.mul(y)
}

/// Returns the smaller of two values.
#[inline]
pub fn min(a: &I256, b: &I256) -> I256 {
    if a < b {
        a.clone()
    } else {
        b.clone()
    }
}

/// Returns the larger of two values.
#[inline]
pub fn max(a: &I256, b: &I256) -> I256 {
    if a > b {
        a.clone()
    } else {
        b.clone()
    }
}

/// `x * y / WAD` — multiply two wad values.
#[inline]
pub fn wmul(env: &Env, x: &I256, y: &I256) -> I256 {
    x.mul(y).div(&crate::constants::wad(env))
}

/// `x * WAD / y` — divide two wad values.
#[inline]
pub fn wdiv(env: &Env, x: &I256, y: &I256) -> I256 {
    x.mul(&crate::constants::wad(env)).div(y)
}

/// `x * y / RAY` — multiply two ray values.
#[inline]
pub fn rmul(env: &Env, x: &I256, y: &I256) -> I256 {
    x.mul(y).div(&crate::constants::ray(env))
}

/// `x * RAY / y` — divide two ray values.
#[inline]
pub fn rdiv(env: &Env, x: &I256, y: &I256) -> I256 {
    x.mul(&crate::constants::ray(env)).div(y)
}

/// Multiplies two *rad* values (45 decimals).
#[inline]
pub fn radmul(env: &Env, x: &I256, y: &I256) -> I256 {
    x.mul(y).div(&crate::constants::rad(env))
}
