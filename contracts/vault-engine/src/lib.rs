#![no_std]
//! # VaultEngine — the immutable core ledger
//!
//! Port of `contracts/core/VaultEngine.sol`.
//!
//! Master record of every piece of collateral and every unit of debt in the system. Enforces the fundamental
//! rule that no vault can mint more USDR than its collateral allows. Its rules can never be changed after
//! deployment: only risk parameters can be tuned, and only through the Governor.
//!
//! USDR charges no stability fee, so each ilk's `rate` is initialized to `RAY` (1.0) and never changes.
//! Internal USDR balances are tracked in `rad` (45 decimals).
//!
//! ## Soroban adaptations
//! * `move(from,to,rad)` is exported as `move_rad` (`move` is a Rust keyword).
//! * `ownerOf` returns `Option<Address>`; an unopened vault has no owner rather than `address(0)`.
//! * Every entrypoint that read `msg.sender` takes an explicit `sender` argument that is authenticated here.
//! * `paused()` / `isBreached()` / `isVolatile()` are cross-contract calls into the Governor and the Solvency
//!   Engine, exactly as in the Solidity original.

use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, panic_with_error, Address, BytesN, Env,
    Symbol, I256,
};

use shared::access::{grant_role, require_role, set_role_admin};
use shared::constants::{ray as ray_unit, WARD};
use shared::errors::Error;
use shared::events::Cage;
use shared::math::{self, zero};
use shared::storage;
use shared::types::{FileValue, Ilk, Urn};

use interfaces::igovernor::GovernorClient;
use interfaces::isolvency_engine::SolvencyEngineClient;

/// Storage keys.
#[contracttype]
pub enum DataKey {
    /// Total USDR issued [rad].
    Debt,
    /// Total bad debt [rad].
    Vice,
    /// Global debt ceiling [rad].
    GlobalLine,
    /// System liveness flag (`1` while live, `0` after shutdown).
    Live,
    /// The wired Solvency Engine.
    SolvencyEngine,
    /// The wired Governor.
    Governor,
    /// Total number of vaults ever opened.
    VaultCount,
    /// Whether `operator` may manage `owner`'s positions.
    Can(Address, Address),
    /// Per-collateral risk settings.
    Ilk(BytesN<32>),
    /// Owner of a vault.
    OwnerOf(u64),
    /// Collateral type a vault is bound to.
    IlkOf(u64),
    /// Locked collateral and normalized debt of a vault.
    Urn(u64),
    /// Free collateral balance of a user for an ilk [wad].
    Collateral(BytesN<32>, Address),
    /// Internal USDR balance of a user [rad].
    Usdr(Address),
    /// Bad debt attributed to a debt sink [rad].
    Sin(Address),
}

/* ========================== EVENTS ========================== */

/// Emitted when an owner permits an operator to manage its positions.
#[contractevent]
#[derive(Clone)]
pub struct Hope {
    #[topic]
    pub owner: Address,
    #[topic]
    pub operator: Address,
}

/// Emitted when an owner revokes an operator's management permission.
#[contractevent]
#[derive(Clone)]
pub struct Nope {
    #[topic]
    pub owner: Address,
    #[topic]
    pub operator: Address,
}

/// Emitted when a new collateral type is registered.
#[contractevent]
#[derive(Clone)]
pub struct Init {
    #[topic]
    pub ilk_id: BytesN<32>,
}

/// Emitted when a new vault is opened.
#[contractevent]
#[derive(Clone)]
pub struct Open {
    #[topic]
    pub ilk_id: BytesN<32>,
    #[topic]
    pub owner: Address,
    #[topic]
    pub vault_id: u64,
}

/// Emitted when a global numeric parameter is updated (Solidity `File(bytes32,uint256)`).
#[contractevent]
#[derive(Clone)]
pub struct FileNum {
    #[topic]
    pub what: Symbol,
    pub data: I256,
}

/// Emitted when a global address dependency is updated (Solidity `File(bytes32,address)`).
#[contractevent]
#[derive(Clone)]
pub struct FileAddr {
    #[topic]
    pub what: Symbol,
    pub addr: Address,
}

/// Emitted when a per-collateral parameter is updated (Solidity `File(bytes32,bytes32,uint256)`).
#[contractevent]
#[derive(Clone)]
pub struct FileIlk {
    #[topic]
    pub ilk_id: BytesN<32>,
    #[topic]
    pub what: Symbol,
    pub data: I256,
}

/// Emitted when a user's free collateral balance is adjusted.
#[contractevent]
#[derive(Clone)]
pub struct Slip {
    #[topic]
    pub ilk_id: BytesN<32>,
    #[topic]
    pub user: Address,
    pub wad: I256,
}

/// Emitted when collateral is moved between two free balances.
#[contractevent]
#[derive(Clone)]
pub struct Flux {
    #[topic]
    pub ilk_id: BytesN<32>,
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    pub wad: I256,
}

/// Emitted when internal USDR is moved between two accounts.
#[contractevent]
#[derive(Clone)]
pub struct Move {
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    pub rad: I256,
}

/// Emitted when a vault is modified.
#[contractevent]
#[derive(Clone)]
pub struct Frob {
    #[topic]
    pub ilk_id: BytesN<32>,
    #[topic]
    pub vault_id: u64,
    pub v: Address,
    pub w: Address,
    pub dink: I256,
    pub dart: I256,
}

/// Emitted when a vault is seized by privileged liquidation logic.
#[contractevent]
#[derive(Clone)]
pub struct Grab {
    #[topic]
    pub ilk_id: BytesN<32>,
    #[topic]
    pub vault_id: u64,
    pub v: Address,
    pub w: Address,
    pub dink: I256,
    pub dart: I256,
}

/// Emitted when bad debt is healed against internal USDR.
#[contractevent]
#[derive(Clone)]
pub struct Heal {
    #[topic]
    pub account: Address,
    pub rad: I256,
}

/// Emitted when a debt sink and a USDR account are adjusted simultaneously.
#[contractevent]
#[derive(Clone)]
pub struct Suck {
    #[topic]
    pub u: Address,
    #[topic]
    pub v: Address,
    pub rad: I256,
}

#[contract]
pub struct VaultEngine;

#[contractimpl]
impl VaultEngine {
    /// Authorizes the deployer and marks the ledger live.
    pub fn __constructor(env: Env, admin: Address) {
        admin.require_auth();

        set_role_admin(&env, &WARD, &WARD);
        grant_role(&env, &WARD, &admin);

        env.storage().instance().set(&DataKey::Live, &1u32);
    }

    /* ========================== VIEWS ========================== */

    /// Returns the total USDR issued [rad].
    pub fn debt(env: Env) -> I256 {
        instance_i256(&env, &DataKey::Debt)
    }

    /// Returns the total bad debt [rad].
    pub fn vice(env: Env) -> I256 {
        instance_i256(&env, &DataKey::Vice)
    }

    /// Returns the global debt ceiling [rad].
    pub fn global_line(env: Env) -> I256 {
        instance_i256(&env, &DataKey::GlobalLine)
    }

    /// Returns the system liveness flag. `1` while live, `0` after shutdown.
    pub fn live(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::Live).unwrap_or(0)
    }

    /// Returns the wired Solvency Engine, if any.
    pub fn solvency_engine(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::SolvencyEngine)
    }

    /// Returns the wired Governor, if any.
    pub fn governor(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Governor)
    }

    /// Returns whether an operator may manage an owner's positions.
    pub fn can(env: Env, owner: Address, operator: Address) -> u32 {
        storage::get(&env, &DataKey::Can(owner, operator)).unwrap_or(0)
    }

    /// Returns a collateral type's settings and totals.
    pub fn ilks(env: Env, ilk_id: BytesN<32>) -> Ilk {
        read_ilk(&env, &ilk_id)
    }

    /// Returns the total number of vaults ever opened.
    pub fn vault_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::VaultCount)
            .unwrap_or(0)
    }

    /// Returns the owner of a vault, or `None` when the vault has not been opened.
    pub fn owner_of(env: Env, vault_id: u64) -> Option<Address> {
        storage::get(&env, &DataKey::OwnerOf(vault_id))
    }

    /// Returns the collateral type a vault is bound to.
    pub fn ilk_of(env: Env, vault_id: u64) -> BytesN<32> {
        storage::get(&env, &DataKey::IlkOf(vault_id)).unwrap_or_else(|| empty_ilk_id(&env))
    }

    /// Returns a vault's locked collateral and normalized debt.
    pub fn urns(env: Env, vault_id: u64) -> Urn {
        read_urn(&env, vault_id)
    }

    /// Returns a user's free collateral balance [wad].
    pub fn collateral(env: Env, ilk_id: BytesN<32>, user: Address) -> I256 {
        storage::get(&env, &DataKey::Collateral(ilk_id, user)).unwrap_or_else(|| zero(&env))
    }

    /// Returns a user's internal USDR balance [rad].
    pub fn usdr(env: Env, user: Address) -> I256 {
        storage::get(&env, &DataKey::Usdr(user)).unwrap_or_else(|| zero(&env))
    }

    /// Returns a debt sink's bad debt balance [rad].
    pub fn sin(env: Env, debt_sink: Address) -> I256 {
        storage::get(&env, &DataKey::Sin(debt_sink)).unwrap_or_else(|| zero(&env))
    }
}

#[contractimpl]
impl VaultEngine {
    /* ========================== AUTHORITY ========================== */

    /// Permits `operator` to manage the caller's positions. Mirrors `hope`.
    pub fn hope(env: Env, sender: Address, operator: Address) {
        sender.require_auth();

        storage::set(&env, &DataKey::Can(sender.clone(), operator.clone()), &1u32);

        Hope {
            owner: sender,
            operator,
        }
        .publish(&env);
    }

    /// Revokes `operator`'s permission to manage the caller's positions. Mirrors `nope`.
    pub fn nope(env: Env, sender: Address, operator: Address) {
        sender.require_auth();

        storage::set(&env, &DataKey::Can(sender.clone(), operator.clone()), &0u32);

        Nope {
            owner: sender,
            operator,
        }
        .publish(&env);
    }

    /* ========================== ADMINISTRATION ========================== */

    /// Registers a collateral type by pinning its rate to `RAY`. Mirrors `init`.
    pub fn init(env: Env, caller: Address, ilk_id: BytesN<32>) {
        caller.require_auth();
        require_role(&env, &WARD, &caller);

        if read_ilk(&env, &ilk_id).rate != zero(&env) {
            panic_with_error!(&env, Error::IlkAlreadyInitialized);
        }

        let mut ilk = read_ilk(&env, &ilk_id);
        ilk.rate = ray_unit(&env);
        write_ilk(&env, &ilk_id, &ilk);

        Init { ilk_id }.publish(&env);
    }

    /// Updates a global parameter. Mirrors `file(bytes32,uint256)` / `file(bytes32,address)`.
    pub fn file(env: Env, caller: Address, what: Symbol, value: FileValue) {
        caller.require_auth();
        require_role(&env, &WARD, &caller);

        if VaultEngine::live(env.clone()) != 1 {
            panic_with_error!(&env, Error::NotLive);
        }

        if what == sym(&env, "globalLine") {
            match value {
                FileValue::Num(data) => {
                    env.storage().instance().set(&DataKey::GlobalLine, &data);
                    FileNum { what, data }.publish(&env);
                }
                FileValue::Addr(_) => panic_with_error!(&env, Error::UnrecognizedParameter),
            }
        } else if what == sym(&env, "solvencyEngine") {
            match value {
                FileValue::Addr(addr) => {
                    env.storage()
                        .instance()
                        .set(&DataKey::SolvencyEngine, &addr);
                    FileAddr { what, addr }.publish(&env);
                }
                FileValue::Num(_) => panic_with_error!(&env, Error::UnrecognizedParameter),
            }
        } else if what == sym(&env, "governor") {
            match value {
                FileValue::Addr(addr) => {
                    env.storage().instance().set(&DataKey::Governor, &addr);
                    FileAddr { what, addr }.publish(&env);
                }
                FileValue::Num(_) => panic_with_error!(&env, Error::UnrecognizedParameter),
            }
        } else {
            panic_with_error!(&env, Error::UnrecognizedParameter);
        }
    }

    /// Updates a per-collateral parameter. Mirrors `file(bytes32,bytes32,uint256)`.
    pub fn file_ilk(env: Env, caller: Address, ilk_id: BytesN<32>, what: Symbol, value: FileValue) {
        caller.require_auth();
        require_role(&env, &WARD, &caller);

        if VaultEngine::live(env.clone()) != 1 {
            panic_with_error!(&env, Error::NotLive);
        }

        let data = match value {
            FileValue::Num(data) => data,
            FileValue::Addr(_) => panic_with_error!(&env, Error::UnrecognizedParameter),
        };

        let mut ilk = read_ilk(&env, &ilk_id);

        if what == sym(&env, "spot") {
            ilk.spot = data.clone();
        } else if what == sym(&env, "line") {
            ilk.line = data.clone();
        } else if what == sym(&env, "dust") {
            ilk.dust = data.clone();
        } else {
            panic_with_error!(&env, Error::UnrecognizedParameter);
        }

        write_ilk(&env, &ilk_id, &ilk);

        FileIlk { ilk_id, what, data }.publish(&env);
    }

    /// Opens a new vault bound to `ilk_id` and owned by `usr`. Mirrors `open`.
    pub fn open(env: Env, ilk_id: BytesN<32>, usr: Address) -> u64 {
        // Vaults may only be opened while the system is live.
        if VaultEngine::live(env.clone()) != 1 {
            panic_with_error!(&env, Error::NotLive);
        }

        // The collateral type must have been initialized: junk vaults against unknown ilks are rejected here.
        if read_ilk(&env, &ilk_id).rate == zero(&env) {
            panic_with_error!(&env, Error::IlkNotInitialized);
        }

        // Vault ids are sequential and never reused. Ownership is immutable.
        let vault_id = env
            .storage()
            .instance()
            .get(&DataKey::VaultCount)
            .unwrap_or(0u64)
            + 1;
        env.storage()
            .instance()
            .set(&DataKey::VaultCount, &vault_id);

        storage::set(&env, &DataKey::OwnerOf(vault_id), &usr);
        storage::set(&env, &DataKey::IlkOf(vault_id), &ilk_id);

        Open {
            ilk_id,
            owner: usr,
            vault_id,
        }
        .publish(&env);

        vault_id
    }

    /// Shuts the ledger down. Mirrors `cage`.
    pub fn cage(env: Env, caller: Address) {
        caller.require_auth();
        require_role(&env, &WARD, &caller);

        env.storage().instance().set(&DataKey::Live, &0u32);

        Cage {}.publish(&env);
    }
}

#[contractimpl]
impl VaultEngine {
    /* ========================== BALANCES ========================== */

    /// Adjusts a user's free collateral balance. Mirrors `slip`. Restricted to Ward (the Collateral Adapter).
    pub fn slip(env: Env, caller: Address, ilk_id: BytesN<32>, user: Address, wad: I256) {
        caller.require_auth();
        require_role(&env, &WARD, &caller);

        let current = read_collateral(&env, &ilk_id, &user);
        storage::set(
            &env,
            &DataKey::Collateral(ilk_id.clone(), user.clone()),
            &math::add(&env, &current, &wad),
        );

        Slip { ilk_id, user, wad }.publish(&env);
    }

    /// Moves free collateral between two users. Mirrors `flux`.
    pub fn flux(
        env: Env,
        sender: Address,
        ilk_id: BytesN<32>,
        from: Address,
        to: Address,
        wad: I256,
    ) {
        sender.require_auth();

        if !wish(&env, &from, &sender) {
            panic_with_error!(&env, Error::NotAllowed);
        }

        let from_balance = read_collateral(&env, &ilk_id, &from);
        if from_balance < wad {
            panic_with_error!(&env, Error::SubUnderflow);
        }
        storage::set(
            &env,
            &DataKey::Collateral(ilk_id.clone(), from.clone()),
            &math::sub(&env, &from_balance, &wad),
        );
        storage::set(
            &env,
            &DataKey::Collateral(ilk_id.clone(), to.clone()),
            &math::add(&env, &read_collateral(&env, &ilk_id, &to), &wad),
        );

        Flux {
            ilk_id,
            from,
            to,
            wad,
        }
        .publish(&env);
    }

    /// Moves internal USDR between two accounts. Mirrors `move` (exported as `move_rad`).
    pub fn move_rad(env: Env, sender: Address, from: Address, to: Address, rad: I256) {
        sender.require_auth();

        if !wish(&env, &from, &sender) {
            panic_with_error!(&env, Error::NotAllowed);
        }

        let from_balance = read_usdr(&env, &from);
        if from_balance < rad {
            panic_with_error!(&env, Error::SubUnderflow);
        }
        storage::set(
            &env,
            &DataKey::Usdr(from.clone()),
            &math::sub(&env, &from_balance, &rad),
        );
        storage::set(
            &env,
            &DataKey::Usdr(to.clone()),
            &math::add(&env, &read_usdr(&env, &to), &rad),
        );

        Move { from, to, rad }.publish(&env);
    }

    /// Heals the caller's bad debt against its internal USDR. Mirrors `heal`.
    /// Deliberately callable after shutdown so that emergency settlement can thaw.
    pub fn heal(env: Env, sender: Address, rad: I256) {
        sender.require_auth();

        storage::set(
            &env,
            &DataKey::Sin(sender.clone()),
            &math::sub(&env, &read_sin(&env, &sender), &rad),
        );
        storage::set(
            &env,
            &DataKey::Usdr(sender.clone()),
            &math::sub(&env, &read_usdr(&env, &sender), &rad),
        );
        write_instance_i256(
            &env,
            &DataKey::Vice,
            &math::sub(&env, &instance_i256(&env, &DataKey::Vice), &rad),
        );
        write_instance_i256(
            &env,
            &DataKey::Debt,
            &math::sub(&env, &instance_i256(&env, &DataKey::Debt), &rad),
        );

        Heal {
            account: sender,
            rad,
        }
        .publish(&env);
    }

    /// Creates bad debt for `u` and matching internal USDR for `v`. Mirrors `suck`. Restricted to Ward.
    /// Deliberately callable after shutdown.
    pub fn suck(env: Env, caller: Address, u: Address, v: Address, rad: I256) {
        caller.require_auth();
        require_role(&env, &WARD, &caller);

        storage::set(
            &env,
            &DataKey::Sin(u.clone()),
            &math::add(&env, &read_sin(&env, &u), &rad),
        );
        storage::set(
            &env,
            &DataKey::Usdr(v.clone()),
            &math::add(&env, &read_usdr(&env, &v), &rad),
        );
        write_instance_i256(
            &env,
            &DataKey::Vice,
            &math::add(&env, &instance_i256(&env, &DataKey::Vice), &rad),
        );
        write_instance_i256(
            &env,
            &DataKey::Debt,
            &math::add(&env, &instance_i256(&env, &DataKey::Debt), &rad),
        );

        Suck { u, v, rad }.publish(&env);
    }
}

#[contractimpl]
impl VaultEngine {
    /* ========================== VAULT OPERATIONS ========================== */

    /// Modifies a vault: `dink` collateral and `dart` normalized debt.
    ///
    /// Permissions mirror `wish`: a caller may act on a vault it owns or on an account that `hope`d it.
    pub fn frob(
        env: Env,
        sender: Address,
        vault_id: u64,
        v: Address,
        w: Address,
        dink: I256,
        dart: I256,
    ) {
        sender.require_auth();

        // System must be live.
        if VaultEngine::live(env.clone()) != 1 {
            panic_with_error!(&env, Error::NotLive);
        }

        // The vault must have been opened. Its owner and collateral type are fixed at open time.
        let owner: Option<Address> = storage::get(&env, &DataKey::OwnerOf(vault_id));
        let owner = match owner {
            Some(o) => o,
            None => panic_with_error!(&env, Error::VaultNotFound),
        };

        let ilk_id: BytesN<32> =
            storage::get(&env, &DataKey::IlkOf(vault_id)).unwrap_or_else(|| empty_ilk_id(&env));

        let mut urn = read_urn(&env, vault_id);
        let mut ilk = read_ilk(&env, &ilk_id);

        // The collateral type must have been initialized.
        if ilk.rate == zero(&env) {
            panic_with_error!(&env, Error::IlkNotInitialized);
        }

        // Emergency pause check (full stop): when the Governor is wired and paused, all vault modifications are
        // blocked, including risk-decreasing operations.
        let governor: Option<Address> = env.storage().instance().get(&DataKey::Governor);
        if let Some(gov) = governor {
            if GovernorClient::new(&env, &gov).paused() {
                panic_with_error!(&env, Error::SystemPaused);
            }
        }

        // Solvency gate: risk-increasing changes (drawing debt or withdrawing collateral) against VOLATILE
        // collateral are blocked while the invariant is breached. Stable (PSM) ilks are exempt here.
        let zero_ = zero(&env);
        let risky = dart > zero_ || dink < zero_;
        if risky {
            let engine: Option<Address> = env.storage().instance().get(&DataKey::SolvencyEngine);
            if let Some(engine_addr) = engine {
                let solvency = SolvencyEngineClient::new(&env, &engine_addr);
                if solvency.is_breached() && solvency.is_volatile(&ilk_id) {
                    panic_with_error!(&env, Error::SolvencyGateActive);
                }
            }
        }

        urn.ink = math::add(&env, &urn.ink, &dink);
        urn.art = math::add(&env, &urn.art, &dart);
        ilk.global_art = math::add(&env, &ilk.global_art, &dart);
        ilk.global_ink = math::add(&env, &ilk.global_ink, &dink);

        // `rate` is fixed at RAY forever, so `dtab`/`tab` are exact rad values and the dust/tab comparisons below
        // remain correct only under that assumption.
        let dtab = math::mul(&env, &ilk.rate, &dart);
        let tab = math::umul(&env, &ilk.rate, &urn.art);

        let new_debt = math::add(&env, &instance_i256(&env, &DataKey::Debt), &dtab);
        write_instance_i256(&env, &DataKey::Debt, &new_debt);

        let dart_le_zero = dart <= zero_;

        // Ceiling check: either debt is being repaid, or both the ilk ceiling and the global ceiling must hold.
        let ceiling_ok = math::umul(&env, &ilk.global_art, &ilk.rate) <= ilk.line
            && new_debt <= instance_i256(&env, &DataKey::GlobalLine);
        if !(dart_le_zero || ceiling_ok) {
            panic_with_error!(&env, Error::CeilingExceeded);
        }

        // Safety check: the urn is either less risky than before, or it is safe after the change.
        let less_risky = dart_le_zero && dink >= zero_;
        if !(less_risky || tab <= math::umul(&env, &urn.ink, &ilk.spot)) {
            panic_with_error!(&env, Error::NotSafe);
        }

        // Permission checks: the vault is either less risky than before, or its owner consents; collateral is
        // either not being taken, or its source consents; internal USDR is either not being drawn down, or the
        // destination consents.
        if !(less_risky || wish(&env, &owner, &sender)) {
            panic_with_error!(&env, Error::NotAllowed);
        }
        if !(dink <= zero_ || wish(&env, &v, &sender)) {
            panic_with_error!(&env, Error::NotAllowed);
        }
        if !(dart >= zero_ || wish(&env, &w, &sender)) {
            panic_with_error!(&env, Error::NotAllowed);
        }

        // Minimum size check: the urn either has no debt, or a non-dusty amount.
        if !(urn.art == zero_ || tab >= ilk.dust) {
            panic_with_error!(&env, Error::DustAmount);
        }

        let col_v = read_collateral(&env, &ilk_id, &v);
        storage::set(
            &env,
            &DataKey::Collateral(ilk_id.clone(), v.clone()),
            &math::sub(&env, &col_v, &dink),
        );
        storage::set(
            &env,
            &DataKey::Usdr(w.clone()),
            &math::add(&env, &read_usdr(&env, &w), &dtab),
        );

        write_urn(&env, vault_id, &urn);
        write_ilk(&env, &ilk_id, &ilk);

        Frob {
            ilk_id,
            vault_id,
            v,
            w,
            dink,
            dart,
        }
        .publish(&env);
    }

    /// Seizes a vault. Mirrors `grab`. Restricted to Ward and deliberately callable after shutdown: the
    /// emergency settlement module seizes positions through this function after `cage`.
    pub fn grab(
        env: Env,
        caller: Address,
        vault_id: u64,
        v: Address,
        w: Address,
        dink: I256,
        dart: I256,
    ) {
        caller.require_auth();
        require_role(&env, &WARD, &caller);

        let owner: Option<Address> = storage::get(&env, &DataKey::OwnerOf(vault_id));
        if owner.is_none() {
            panic_with_error!(&env, Error::VaultNotFound);
        }

        let ilk_id: BytesN<32> =
            storage::get(&env, &DataKey::IlkOf(vault_id)).unwrap_or_else(|| empty_ilk_id(&env));

        let mut urn = read_urn(&env, vault_id);
        let mut ilk = read_ilk(&env, &ilk_id);

        urn.ink = math::add(&env, &urn.ink, &dink);
        urn.art = math::add(&env, &urn.art, &dart);
        ilk.global_art = math::add(&env, &ilk.global_art, &dart);
        ilk.global_ink = math::add(&env, &ilk.global_ink, &dink);

        let dtab = math::mul(&env, &ilk.rate, &dart);

        let col_v = read_collateral(&env, &ilk_id, &v);
        storage::set(
            &env,
            &DataKey::Collateral(ilk_id.clone(), v.clone()),
            &math::sub(&env, &col_v, &dink),
        );
        storage::set(
            &env,
            &DataKey::Sin(w.clone()),
            &math::sub(&env, &read_sin(&env, &w), &dtab),
        );
        write_instance_i256(
            &env,
            &DataKey::Vice,
            &math::sub(&env, &instance_i256(&env, &DataKey::Vice), &dtab),
        );

        write_urn(&env, vault_id, &urn);
        write_ilk(&env, &ilk_id, &ilk);

        Grab {
            ilk_id,
            vault_id,
            v,
            w,
            dink,
            dart,
        }
        .publish(&env);
    }

    /* ========================== ROLE MANAGEMENT ========================== */

    /// Grants `role` to `account`. Mirrors OpenZeppelin `AccessControl.grantRole`.
    pub fn grant(env: Env, caller: Address, role: Symbol, account: Address) {
        shared::access::grant_role_checked(&env, &role, &caller, &account);
    }

    /// Revokes `role` from `account`. Mirrors OpenZeppelin `AccessControl.revokeRole`.
    pub fn revoke(env: Env, caller: Address, role: Symbol, account: Address) {
        shared::access::revoke_role_checked(&env, &role, &caller, &account);
    }

    /// Returns whether `account` holds `role`. Mirrors OpenZeppelin `AccessControl.hasRole`.
    pub fn has(env: Env, role: Symbol, account: Address) -> bool {
        shared::access::has_role(&env, &role, &account)
    }
}

/* ========================== INTERNAL HELPERS ========================== */

/// Builds a `Symbol` from a short ASCII string (used for the `what` parameter names).
fn sym(env: &Env, s: &str) -> Symbol {
    Symbol::new(env, s)
}

/// The zero collateral identifier, returned for unopened vaults.
fn empty_ilk_id(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[0u8; 32])
}

/// Reads an instance-stored `I256`, defaulting to zero.
fn instance_i256(env: &Env, key: &DataKey) -> I256 {
    env.storage()
        .instance()
        .get(key)
        .unwrap_or_else(|| zero(env))
}

/// Writes an instance-stored `I256`.
fn write_instance_i256(env: &Env, key: &DataKey, value: &I256) {
    env.storage().instance().set(key, value);
}

/// Reads a collateral type, defaulting to all-zero (an uninitialized ilk has `rate == 0`).
fn read_ilk(env: &Env, ilk_id: &BytesN<32>) -> Ilk {
    storage::get(env, &DataKey::Ilk(ilk_id.clone())).unwrap_or_else(|| Ilk {
        global_art: zero(env),
        global_ink: zero(env),
        rate: zero(env),
        spot: zero(env),
        line: zero(env),
        dust: zero(env),
    })
}

/// Writes a collateral type.
fn write_ilk(env: &Env, ilk_id: &BytesN<32>, ilk: &Ilk) {
    storage::set(env, &DataKey::Ilk(ilk_id.clone()), ilk);
}

/// Reads a vault, defaulting to empty.
fn read_urn(env: &Env, vault_id: u64) -> Urn {
    storage::get(env, &DataKey::Urn(vault_id)).unwrap_or_else(|| Urn {
        ink: zero(env),
        art: zero(env),
    })
}

/// Writes a vault.
fn write_urn(env: &Env, vault_id: u64, urn: &Urn) {
    storage::set(env, &DataKey::Urn(vault_id), urn);
}

/// Reads a user's free collateral balance.
fn read_collateral(env: &Env, ilk_id: &BytesN<32>, user: &Address) -> I256 {
    storage::get(env, &DataKey::Collateral(ilk_id.clone(), user.clone()))
        .unwrap_or_else(|| zero(env))
}

/// Reads a user's internal USDR balance.
fn read_usdr(env: &Env, user: &Address) -> I256 {
    storage::get(env, &DataKey::Usdr(user.clone())).unwrap_or_else(|| zero(env))
}

/// Reads a debt sink's bad debt balance.
fn read_sin(env: &Env, sink: &Address) -> I256 {
    storage::get(env, &DataKey::Sin(sink.clone())).unwrap_or_else(|| zero(env))
}

/// Returns whether `operator` may manage the positions of `owner`. Mirrors `_wish`.
fn wish(env: &Env, owner: &Address, operator: &Address) -> bool {
    owner == operator
        || storage::get::<_, u32>(env, &DataKey::Can(owner.clone(), operator.clone())).unwrap_or(0)
            == 1
}

#[cfg(test)]
mod test;
