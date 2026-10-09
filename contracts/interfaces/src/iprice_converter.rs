//! Mirrors `contracts/interfaces/IPriceConverter.sol`.
use soroban_sdk::{contractclient, Address, BytesN, Env, Symbol, I256};

use shared::types::{FileValue, IlkOracle};

/// Turns a raw price into the maximum USDR mintable per unit of collateral.
#[contractclient(name = "PriceConverterClient")]
pub trait IPriceConverter {
    fn file(env: Env, caller: Address, what: Symbol, value: FileValue);
    fn file_ilk(env: Env, caller: Address, ilk_id: BytesN<32>, what: Symbol, value: FileValue);
    fn cage(env: Env, caller: Address);
    fn poke(env: Env, ilk_id: BytesN<32>);
    fn vault_engine(env: Env) -> Address;
    fn par(env: Env) -> I256;
    fn live(env: Env) -> u32;
    fn ilks(env: Env, ilk_id: BytesN<32>) -> IlkOracle;
}
