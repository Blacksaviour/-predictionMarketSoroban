//! Mirrors `contracts/interfaces/ICollateralAdapter.sol`.
use soroban_sdk::{contractclient, Address, BytesN, Env, I256};

use shared::types::AdapterIlk;

/// The doorway for tokens entering and leaving the system.
#[contractclient(name = "CollateralAdapterClient")]
pub trait ICollateralAdapter {
    fn init(env: Env, caller: Address, ilk_id: BytesN<32>, token: Address);
    fn cage(env: Env, caller: Address, ilk_id: BytesN<32>);
    fn join(env: Env, sender: Address, ilk_id: BytesN<32>, user: Address, amount: I256);
    fn exit(env: Env, sender: Address, ilk_id: BytesN<32>, user: Address, amount: I256);
    fn vault_engine(env: Env) -> Address;
    fn ilks(env: Env, ilk_id: BytesN<32>) -> AdapterIlk;
}
