//! Mirrors `contracts/interfaces/IPegStabilityModule.sol`.
use soroban_sdk::{contractclient, Address, BytesN, Env, Symbol, I256};

use shared::types::{FileValue, PsmIlk};

/// The on-ramp and off-ramp for stablecoins.
#[contractclient(name = "PegStabilityModuleClient")]
pub trait IPegStabilityModule {
    fn init(env: Env, caller: Address, ilk_id: BytesN<32>);
    fn file(env: Env, caller: Address, what: Symbol, value: FileValue);
    fn sell_stable(env: Env, sender: Address, ilk_id: BytesN<32>, user: Address, stable_amt: I256);
    fn buy_stable(env: Env, sender: Address, ilk_id: BytesN<32>, user: Address, stable_amt: I256);
    fn collateral_adapter(env: Env) -> Address;
    fn reserve_accounting(env: Env) -> Address;
    fn vault_engine(env: Env) -> Address;
    fn usdr(env: Env) -> Address;
    fn ilks(env: Env, ilk_id: BytesN<32>) -> PsmIlk;
    fn solvency_engine(env: Env) -> Option<Address>;
    fn governor(env: Env) -> Option<Address>;
}
