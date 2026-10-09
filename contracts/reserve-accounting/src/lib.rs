#![no_std]
//! # ReserveAccounting
//!
//! Port of `contracts/reserve/ReserveAccounting.sol`.
//!
//! The bookkeeper for the protocol's stable dollars. Tracks the total reserve (all USDT and USDC held), how much
//! is committed to guaranteed obligations (the settlement escrow), and how much is free (the slack). This is
//! where the reserve is split so that the same dollar is never promised twice.
//!
//! Only the Solvency Engine may update the committed escrow (Committer role), and only the Peg Stability
//! Modules may record reserve movements (Recorder role).

use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, panic_with_error, Address, Env, Symbol,
    I256,
};

use shared::access::{grant_role, require_role, set_role_admin};
use shared::constants::{COMMITTER, RECORDER, WARD};
use shared::errors::Error;
use shared::math::{self, zero};

/// Storage keys.
#[contracttype]
pub enum DataKey {
    /// Total stable reserve held by the protocol [wad].
    TotalReserve,
    /// Amount of the reserve committed to guaranteed obligations [wad].
    CommittedEscrow,
}

/// Emitted when the reserve grows. Mirrors `event RecordIncrease(uint256,uint256)`.
#[contractevent]
#[derive(Clone)]
pub struct RecordIncrease {
    pub wad: I256,
    pub total_reserve: I256,
}

/// Emitted when the reserve shrinks. Mirrors `event RecordDecrease(uint256,uint256)`.
#[contractevent]
#[derive(Clone)]
pub struct RecordDecrease {
    pub wad: I256,
    pub total_reserve: I256,
}

/// Emitted when the committed escrow is updated. Mirrors `event UpdateCommittedEscrow(uint256,uint256)`.
#[contractevent]
#[derive(Clone)]
pub struct UpdateCommittedEscrow {
    pub wad: I256,
    pub free_slack: I256,
}

#[contract]
pub struct ReserveAccounting;

#[contractimpl]
impl ReserveAccounting {
    /// Authorizes the deployer.
    pub fn __constructor(env: Env, admin: Address) {
        admin.require_auth();

        set_role_admin(&env, &COMMITTER, &WARD);
        set_role_admin(&env, &RECORDER, &WARD);
        set_role_admin(&env, &WARD, &WARD);

        grant_role(&env, &WARD, &admin);
    }

    /* ========================== VIEWS ========================== */

    /// Returns the total stable reserve [wad].
    pub fn total_reserve(env: Env) -> I256 {
        read_total(&env)
    }

    /// Returns the amount committed to guaranteed obligations [wad].
    pub fn committed_escrow(env: Env) -> I256 {
        read_escrow(&env)
    }

    /// Returns the uncommitted part of the reserve [wad]. Never negative, because
    /// `update_committed_escrow` forbids committing more than exists.
    pub fn free_slack(env: Env) -> I256 {
        math::sub(&env, &read_total(&env), &read_escrow(&env))
    }

    /* ========================== MUTATIONS ========================== */

    /// Records reserve arriving from a Peg Stability Module. Mirrors `recordIncrease`.
    pub fn record_increase(env: Env, caller: Address, wad: I256) {
        caller.require_auth();
        require_role(&env, &RECORDER, &caller);

        let total = read_total(&env);
        let updated = math::add(&env, &total, &wad);

        env.storage()
            .instance()
            .set(&DataKey::TotalReserve, &updated);

        RecordIncrease {
            wad,
            total_reserve: updated,
        }
        .publish(&env);
    }

    /// Records reserve leaving to a Peg Stability Module. Mirrors `recordDecrease`.
    pub fn record_decrease(env: Env, caller: Address, wad: I256) {
        caller.require_auth();
        require_role(&env, &RECORDER, &caller);

        let total = math::sub(&env, &read_total(&env), &wad);

        // The reserve must never drop below the committed escrow: `free_slack()` would underflow and every
        // consumer of the split (redemption above all) would revert. The PSM checks free slack before
        // recording, so this is defence in depth against any future recorder that does not.
        if total < read_escrow(&env) {
            panic_with_error!(&env, Error::ReserveBelowEscrow);
        }

        env.storage().instance().set(&DataKey::TotalReserve, &total);

        RecordDecrease {
            wad,
            total_reserve: total,
        }
        .publish(&env);
    }

    /// Updates the amount committed to guaranteed obligations. Mirrors `updateCommittedEscrow`.
    pub fn update_committed_escrow(env: Env, caller: Address, wad: I256) {
        caller.require_auth();
        require_role(&env, &COMMITTER, &caller);

        let total = read_total(&env);

        // The committed amount must not exceed the total reserve. This is the solvency guarantee expressed at
        // the accounting level.
        if wad > total {
            panic_with_error!(&env, Error::EscrowExceedsReserve);
        }

        let free_slack = math::sub(&env, &total, &wad);

        env.storage()
            .instance()
            .set(&DataKey::CommittedEscrow, &wad);

        UpdateCommittedEscrow { wad, free_slack }.publish(&env);
    }

    /* ========================== ROLE MANAGEMENT ========================== */

    /// Grants `role` to `account`. Mirrors OpenZeppelin `AccessControl.grantRole`.
    pub fn grant(env: Env, caller: Address, role: Symbol, account: Address) {
        shared::access::grant_role_checked(&env, &role, &caller, &account);
    }

    /// Revokes `role` from `account`. Mirrors OpenZeppelin `AccessControl.revokeRole`.
    pub fn revoke(env: Env, caller: Address, role: Symbol, account: Address) {
        shared::access::revoke_role_checked(&env, &role, &caller, &account);
    }

    /// Returns whether `account` holds `role`. Mirrors OpenZeppelin `AccessControl.hasRole`.
    pub fn has(env: Env, role: Symbol, account: Address) -> bool {
        shared::access::has_role(&env, &role, &account)
    }
}

/* ========================== INTERNAL HELPERS ========================== */

fn read_total(env: &Env) -> I256 {
    env.storage()
        .instance()
        .get(&DataKey::TotalReserve)
        .unwrap_or_else(|| zero(env))
}

fn read_escrow(env: &Env) -> I256 {
    env.storage()
        .instance()
        .get(&DataKey::CommittedEscrow)
        .unwrap_or_else(|| zero(env))
}

#[cfg(test)]
mod test;
