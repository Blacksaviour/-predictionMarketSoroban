//! Role based access control.
//!
//! Port of the OpenZeppelin `AccessControl` behaviour used by every contract in the original system. Solidity
//! derives role identifiers as `keccak256` hashes; here they are short [`Symbol`]s (see
//! [`crate::constants`]), which keeps them readable and cheap while preserving the exact grant/revoke/admin
//! semantics.
//!
//! Storage layout
//! * `role_admin(role)`    -> the admin role of `role` (instance storage)
//! * `role_member(role, a)` -> whether `a` currently holds `role` (persistent storage)
//!
//! Tuple keys are used (rather than an enum) so that role bookkeeping can never collide with the storage keys
//! a contract defines for its own state.

use soroban_sdk::{panic_with_error, symbol_short, Address, Env, Symbol};

use crate::errors::Error;

fn admin_key(role: &Symbol) -> (Symbol, Symbol) {
    (symbol_short!("role_adm"), role.clone())
}

fn member_key(role: &Symbol, account: &Address) -> (Symbol, Symbol, Address) {
    (symbol_short!("role_mbr"), role.clone(), account.clone())
}

/// Sets the admin role of `role`. Mirrors `_setRoleAdmin(role, adminRole)`.
pub fn set_role_admin(env: &Env, role: &Symbol, admin_role: &Symbol) {
    env.storage().instance().set(&admin_key(role), admin_role);
}

/// Returns the admin role of `role`, defaulting to `role` itself (OpenZeppelin's default).
pub fn role_admin(env: &Env, role: &Symbol) -> Symbol {
    env.storage()
        .instance()
        .get(&admin_key(role))
        .unwrap_or_else(|| role.clone())
}

/// Grants `role` to `account`. Mirrors `_grantRole(role, account)`.
pub fn grant_role(env: &Env, role: &Symbol, account: &Address) {
    env.storage()
        .persistent()
        .set(&member_key(role, account), &true);
}

/// Revokes `role` from `account`. Mirrors `_revokeRole(role, account)`.
pub fn revoke_role(env: &Env, role: &Symbol, account: &Address) {
    env.storage()
        .persistent()
        .remove(&member_key(role, account));
}

/// Returns whether `account` holds `role`. Mirrors `hasRole(role, account)`.
pub fn has_role(env: &Env, role: &Symbol, account: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&member_key(role, account))
        .unwrap_or(false)
}

/// Reverts with [`Error::NotAuthorized`] unless `account` holds `role`. Mirrors the `onlyRole` modifier.
pub fn require_role(env: &Env, role: &Symbol, account: &Address) {
    if !has_role(env, role, account) {
        panic_with_error!(env, Error::NotAuthorized);
    }
}

/// Grants `role` to `account` after checking that `caller` holds the role's admin role.
/// Mirrors `grantRole` (the caller must be authorized) combined with the mandatory `requireAuth` step of
/// Soroban's authorization model.
pub fn grant_role_checked(env: &Env, role: &Symbol, caller: &Address, account: &Address) {
    caller.require_auth();
    let admin_role = role_admin(env, role);
    require_role(env, &admin_role, caller);
    grant_role(env, role, account);
}

/// Revokes `role` from `account` after checking that `caller` holds the role's admin role.
pub fn revoke_role_checked(env: &Env, role: &Symbol, caller: &Address, account: &Address) {
    caller.require_auth();
    let admin_role = role_admin(env, role);
    require_role(env, &admin_role, caller);
    revoke_role(env, role, account);
}
