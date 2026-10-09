//! Mirrors `contracts/interfaces/IOracleSecurityModule.sol`.
use soroban_sdk::{contractclient, Address, BytesN, Env, I256};

/// The delayed price feed.
#[contractclient(name = "OsmClient")]
pub trait IOracleSecurityModule {
    fn stop(env: Env, caller: Address, ilk_id: BytesN<32>);
    fn start(env: Env, caller: Address, ilk_id: BytesN<32>);
    fn void(env: Env, caller: Address, ilk_id: BytesN<32>);
    fn change(env: Env, caller: Address, ilk_id: BytesN<32>, new_src: Address);
    fn poke(env: Env, ilk_id: BytesN<32>);
    fn src(env: Env, ilk_id: BytesN<32>) -> Option<Address>;
    fn delay(env: Env, ilk_id: BytesN<32>) -> u64;
    fn stopped(env: Env, ilk_id: BytesN<32>) -> u32;
    /// `peek` is called by other contracts (the reader role is held by them), so the caller is explicit here.
    fn peek(env: Env, caller: Address, ilk_id: BytesN<32>) -> (I256, bool);
    fn peep(env: Env, caller: Address, ilk_id: BytesN<32>) -> (I256, bool);
    fn read(env: Env, caller: Address, ilk_id: BytesN<32>) -> I256;
    fn pass(env: Env, ilk_id: BytesN<32>) -> bool;
    fn hop(env: Env) -> u32;
}
