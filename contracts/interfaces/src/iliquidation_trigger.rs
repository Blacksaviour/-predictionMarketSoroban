//! Mirrors `contracts/interfaces/ILiquidationTrigger.sol`.
use soroban_sdk::{contractclient, Address, BytesN, Env, Symbol, I256};

use shared::types::{FileValue, IlkLiquidation};

/// Detects unsafe vaults and starts auctions.
#[contractclient(name = "LiquidationTriggerClient")]
pub trait ILiquidationTrigger {
    fn file(env: Env, caller: Address, what: Symbol, value: FileValue);
    fn file_ilk(env: Env, caller: Address, ilk_id: BytesN<32>, what: Symbol, value: FileValue);
    fn cage(env: Env, caller: Address);
    fn bark(env: Env, vault_id: u64, kpr: Address) -> u64;
    fn digs(env: Env, caller: Address, ilk_id: BytesN<32>, rad: I256);
    fn chop(env: Env, ilk_id: BytesN<32>) -> I256;
    fn vault_engine(env: Env) -> Address;
    fn global_hole(env: Env) -> I256;
    fn global_dirt(env: Env) -> I256;
    fn throttle(env: Env) -> I256;
    fn live(env: Env) -> u32;
    fn balance_sheet(env: Env) -> Option<Address>;
    fn circuit_breaker(env: Env) -> Option<Address>;
    fn governor(env: Env) -> Option<Address>;
    fn ilks(env: Env, ilk_id: BytesN<32>) -> IlkLiquidation;
}
