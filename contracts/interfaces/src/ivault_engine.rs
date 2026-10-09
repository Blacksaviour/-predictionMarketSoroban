//! Mirrors `contracts/interfaces/IVaultEngine.sol`.
use soroban_sdk::{contractclient, Address, BytesN, Env, Symbol, I256};

use shared::types::{FileValue, Ilk, Urn};

/// The immutable core ledger of the USDR system.
///
/// Soroban adaptations:
/// * `move(from,to,rad)` is exported as `move_rad` because `move` is a Rust keyword.
/// * `ownerOf` returns `Option<Address>` (an unopened vault has no owner rather than `address(0)`).
/// * Functions that read `msg.sender` take an explicit `sender` parameter that is authenticated in the callee.
#[contractclient(name = "VaultEngineClient")]
pub trait IVaultEngine {
    /* ========================== MUTATIONS ========================== */
    fn hope(env: Env, sender: Address, operator: Address);
    fn nope(env: Env, sender: Address, operator: Address);
    fn init(env: Env, caller: Address, ilk_id: BytesN<32>);
    fn file(env: Env, caller: Address, what: Symbol, value: FileValue);
    fn file_ilk(env: Env, caller: Address, ilk_id: BytesN<32>, what: Symbol, value: FileValue);
    fn open(env: Env, ilk_id: BytesN<32>, usr: Address) -> u64;
    fn cage(env: Env, caller: Address);
    fn slip(env: Env, caller: Address, ilk_id: BytesN<32>, user: Address, wad: I256);
    fn flux(env: Env, sender: Address, ilk_id: BytesN<32>, from: Address, to: Address, wad: I256);
    fn move_rad(env: Env, sender: Address, from: Address, to: Address, rad: I256);
    fn frob(
        env: Env,
        sender: Address,
        vault_id: u64,
        v: Address,
        w: Address,
        dink: I256,
        dart: I256,
    );
    fn grab(
        env: Env,
        caller: Address,
        vault_id: u64,
        v: Address,
        w: Address,
        dink: I256,
        dart: I256,
    );
    fn heal(env: Env, sender: Address, rad: I256);
    fn suck(env: Env, caller: Address, u: Address, v: Address, rad: I256);

    /* ========================== VIEWS ========================== */
    fn debt(env: Env) -> I256;
    fn vice(env: Env) -> I256;
    fn global_line(env: Env) -> I256;
    fn live(env: Env) -> u32;
    fn solvency_engine(env: Env) -> Option<Address>;
    fn governor(env: Env) -> Option<Address>;
    fn can(env: Env, owner: Address, operator: Address) -> u32;
    fn ilks(env: Env, ilk_id: BytesN<32>) -> Ilk;
    fn vault_count(env: Env) -> u64;
    fn owner_of(env: Env, vault_id: u64) -> Option<Address>;
    fn ilk_of(env: Env, vault_id: u64) -> BytesN<32>;
    fn urns(env: Env, vault_id: u64) -> Urn;
    fn collateral(env: Env, ilk_id: BytesN<32>, user: Address) -> I256;
    fn usdr(env: Env, user: Address) -> I256;
    fn sin(env: Env, debt_sink: Address) -> I256;
}
