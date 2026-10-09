//! Mirrors `contracts/interfaces/IDutchAuction.sol`.
use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env, Symbol, Vec, I256};

use shared::types::{FileValue, Sale};

/// The descending-price auction house.
#[contractclient(name = "DutchAuctionClient")]
pub trait IDutchAuction {
    fn file(env: Env, caller: Address, what: Symbol, value: FileValue);
    fn kick(
        env: Env,
        caller: Address,
        tab: I256,
        lot: I256,
        vault_id: u64,
        usr: Address,
        kpr: Address,
    ) -> u64;
    fn redo(env: Env, keeper: Address, id: u64);
    fn take(env: Env, keeper: Address, id: u64, amt: I256, max: I256, who: Address, data: Bytes);
    fn yank(env: Env, caller: Address, id: u64);
    fn cage(env: Env, caller: Address);
    fn upchost(env: Env);
    fn count(env: Env) -> u64;
    fn list(env: Env) -> Vec<u64>;
    fn get_status(env: Env, id: u64) -> (bool, I256, I256, I256);
    fn ilk_id(env: Env) -> BytesN<32>;
    fn vault_engine(env: Env) -> Address;
    fn chip(env: Env) -> u64;
    fn tip(env: Env) -> I256;
    fn buf(env: Env) -> I256;
    fn tail(env: Env) -> I256;
    fn cusp(env: Env) -> I256;
    fn kicks(env: Env) -> u64;
    fn chost(env: Env) -> I256;
    fn live(env: Env) -> u32;
    fn stopped(env: Env) -> I256;
    fn vow(env: Env) -> Option<Address>;
    fn governor(env: Env) -> Option<Address>;
    fn dog(env: Env) -> Option<Address>;
    fn pip(env: Env) -> Option<Address>;
    fn calc(env: Env) -> Option<Address>;
    fn active(env: Env, index: u64) -> u64;
    fn sales(env: Env, id: u64) -> Sale;
}
