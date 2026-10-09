//! Persistent storage helpers with archival (TTL) management.
//!
//! Soroban persistent entries expire unless their time-to-live is extended. The Solidity contracts have no
//! equivalent concern, so this module is the one place where the port adds behaviour rather than translating it:
//! every persistent write through [`set`] also bumps the entry's TTL. [`get`] is a thin typed wrapper.

use soroban_sdk::{Env, IntoVal, TryFromVal, Val};

/// Entries are extended when their remaining TTL drops below this many ledgers (~5.8 days at 5s/ledger).
pub const TTL_THRESHOLD: u32 = 100_000;
/// Entries are extended to this many ledgers (~29 days at 5s/ledger).
pub const TTL_EXTEND_TO: u32 = 500_000;

/// Writes a persistent entry and extends its TTL.
pub fn set<K, V>(env: &Env, key: &K, value: &V)
where
    K: IntoVal<Env, Val>,
    V: IntoVal<Env, Val>,
{
    let storage = env.storage().persistent();
    storage.set(key, value);
    storage.extend_ttl(key, TTL_THRESHOLD, TTL_EXTEND_TO);
}

/// Reads a persistent entry.
pub fn get<K, V>(env: &Env, key: &K) -> Option<V>
where
    K: IntoVal<Env, Val>,
    V: TryFromVal<Env, Val>,
{
    env.storage().persistent().get(key)
}

/// Removes a persistent entry.
pub fn remove<K>(env: &Env, key: &K)
where
    K: IntoVal<Env, Val>,
{
    env.storage().persistent().remove(key);
}
