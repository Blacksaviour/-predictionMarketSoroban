//! Mirrors `contracts/interfaces/IEnd.sol`.
use soroban_sdk::{contractclient, Address, BytesN, Env, Symbol, I256};

use shared::types::FileValue;

/// The emergency settlement module.
#[contractclient(name = "EndClient")]
pub trait IEnd {
    fn file(env: Env, caller: Address, what: Symbol, value: FileValue);
    fn cage(env: Env, caller: Address);
    fn cage_ilk(env: Env, ilk_id: BytesN<32>);
    fn skip(env: Env, ilk_id: BytesN<32>, auction_id: u64);
    fn skim(env: Env, vault_id: u64);
    fn free(env: Env, vault_id: u64);
    fn thaw(env: Env);
    fn flow(env: Env, ilk_id: BytesN<32>);
    fn pack(env: Env, sender: Address, wad: I256);
    fn cash(env: Env, sender: Address, ilk_id: BytesN<32>, wad: I256);
    fn vault_engine(env: Env) -> Address;
    fn live(env: Env) -> u32;
    fn when(env: Env) -> u64;
    fn wait(env: Env) -> I256;
    fn debt(env: Env) -> I256;
    fn liquidation_trigger(env: Env) -> Option<Address>;
    fn balance_sheet(env: Env) -> Option<Address>;
    fn price_converter(env: Env) -> Option<Address>;
    fn tag(env: Env, ilk_id: BytesN<32>) -> I256;
    fn gap(env: Env, ilk_id: BytesN<32>) -> I256;
    fn art(env: Env, ilk_id: BytesN<32>) -> I256;
    fn fix(env: Env, ilk_id: BytesN<32>) -> I256;
    fn bag(env: Env, usr: Address) -> I256;
    fn out(env: Env, ilk_id: BytesN<32>, usr: Address) -> I256;
}
