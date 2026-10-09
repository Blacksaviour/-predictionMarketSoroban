#![no_std]
//! # PriceConverter
//!
//! Port of `contracts/oracle/PriceConverter.sol` (Maker's Spot).
//!
//! Takes the delayed price and divides it by the required collateralization ratio to produce the price factor,
//! the maximum USDR mintable per unit of collateral. For RAIN at $1 with a 400% ratio, the factor is $0.25.
//! Supported stablecoins skip the oracle entirely: they are marked fixed and always convert at $1.
//!
//! Every ilk is exactly one of two kinds. A fixed ilk has no oracle and its price is pinned to $1. An
//! oracle-backed ilk reads its price from the OSM. `file("fixed")` and `file("pip")` clear each other so an ilk
//! can never be both, and `poke` reverts for unconfigured ilks rather than writing a zero spot.

use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, panic_with_error, Address, BytesN, Env,
    Symbol, I256,
};

use shared::access::{grant_role, require_role, set_role_admin};
use shared::constants::{ray as ray_unit, wad as wad_unit, WARD};
use shared::errors::Error;
use shared::events::Cage;
use shared::math::zero;
use shared::storage;
use shared::types::{FileValue, IlkOracle};

use interfaces::ioracle_security_module::OsmClient;
use interfaces::ivault_engine::VaultEngineClient;

/// Storage keys.
#[contracttype]
pub enum DataKey {
    /// The wired Vault Engine.
    VaultEngine,
    /// Reference value of USDR [ray].
    Par,
    /// Liveness flag.
    Live,
    /// Per-collateral oracle configuration.
    Ilk(BytesN<32>),
}

/* ========================== EVENTS ========================== */

/// Emitted when the global `par` value is updated (Solidity `File(bytes32,uint256)`).
#[contractevent]
#[derive(Clone)]
pub struct FileNum {
    #[topic]
    pub what: Symbol,
    pub data: I256,
}

/// Emitted when a collateral type's oracle is assigned (Solidity `File(bytes32,bytes32,address)`).
#[contractevent]
#[derive(Clone)]
pub struct FileAddr {
    #[topic]
    pub ilk_id: BytesN<32>,
    #[topic]
    pub what: Symbol,
    pub addr: Address,
}

/// Emitted when a per-collateral numeric parameter is updated.
#[contractevent]
#[derive(Clone)]
pub struct FileIlk {
    #[topic]
    pub ilk_id: BytesN<32>,
    #[topic]
    pub what: Symbol,
    pub data: I256,
}

/// Emitted when a collateral type's price factor is recalculated.
#[contractevent]
#[derive(Clone)]
pub struct Poke {
    #[topic]
    pub ilk_id: BytesN<32>,
    pub val: I256,
    pub spot: I256,
}

#[contract]
pub struct PriceConverter;

#[contractimpl]
impl PriceConverter {
    /// Initializes the converter with the Vault Engine and a `par` value of 1.0.
    pub fn __constructor(env: Env, admin: Address, vault_engine: Address) {
        admin.require_auth();

        set_role_admin(&env, &WARD, &WARD);
        grant_role(&env, &WARD, &admin);

        env.storage()
            .instance()
            .set(&DataKey::VaultEngine, &vault_engine);
        env.storage().instance().set(&DataKey::Par, &ray_unit(&env));
        env.storage().instance().set(&DataKey::Live, &1u32);
    }

    /* ========================== VIEWS ========================== */

    /// Returns the Vault Engine this converter reports to.
    pub fn vault_engine(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::VaultEngine)
            .unwrap_or_else(|| panic_with_error!(&env, Error::InvalidAddress))
    }

    /// Returns the reference value of USDR [ray].
    pub fn par(env: Env) -> I256 {
        env.storage()
            .instance()
            .get(&DataKey::Par)
            .unwrap_or_else(|| ray_unit(&env))
    }

    /// Returns the liveness flag. `1` while live, `0` after shutdown.
    pub fn live(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::Live).unwrap_or(0)
    }

    /// Returns a collateral type's oracle configuration.
    pub fn ilks(env: Env, ilk_id: BytesN<32>) -> IlkOracle {
        read_ilk(&env, &ilk_id)
    }

    /* ========================== ADMINISTRATION ========================== */

    /// Updates the global `par` parameter. Mirrors `file(bytes32,uint256)`.
    pub fn file(env: Env, caller: Address, what: Symbol, value: FileValue) {
        caller.require_auth();
        require_role(&env, &WARD, &caller);

        if PriceConverter::live(env.clone()) != 1 {
            panic_with_error!(&env, Error::NotLive);
        }

        if what != sym(&env, "par") {
            panic_with_error!(&env, Error::UnrecognizedParameter);
        }

        let data = match value {
            FileValue::Num(data) => data,
            FileValue::Addr(_) => panic_with_error!(&env, Error::UnrecognizedParameter),
        };

        env.storage().instance().set(&DataKey::Par, &data);

        FileNum { what, data }.publish(&env);
    }

    /// Updates a per-collateral parameter (`mat`, `fixed` or `pip`).
    pub fn file_ilk(env: Env, caller: Address, ilk_id: BytesN<32>, what: Symbol, value: FileValue) {
        caller.require_auth();
        require_role(&env, &WARD, &caller);

        if PriceConverter::live(env.clone()) != 1 {
            panic_with_error!(&env, Error::NotLive);
        }

        let mut ilk = read_ilk(&env, &ilk_id);

        match value {
            FileValue::Addr(pip) => {
                if what != sym(&env, "pip") {
                    panic_with_error!(&env, Error::UnrecognizedParameter);
                }
                // Assigning an oracle makes the ilk oracle-backed. The kinds are mutually exclusive.
                ilk.pip = Some(pip.clone());
                ilk.fixed_price = false;
                write_ilk(&env, &ilk_id, &ilk);
                FileAddr {
                    ilk_id,
                    what,
                    addr: pip,
                }
                .publish(&env);
            }
            FileValue::Num(data) => {
                if what == sym(&env, "mat") {
                    // A collateralization ratio below 100% would authorize minting more than a dollar of USDR
                    // per dollar of collateral at origination. No legitimate configuration wants that.
                    if data < ray_unit(&env) {
                        panic_with_error!(&env, Error::MatBelowOne);
                    }
                    ilk.mat = data.clone();
                } else if what == sym(&env, "fixed") {
                    if data == I256::from_i32(&env, 1) {
                        // Marking an ilk fixed pins it to $1 and detaches any oracle.
                        ilk.fixed_price = true;
                        ilk.pip = None;
                    } else {
                        // Clearing the fixed flag on an ilk with no oracle would silently brick its price
                        // updates and freeze spot at its last value (a stale price keeps authorizing mints).
                        // The flag is only clearable by assigning an oracle via `file_ilk(..., "pip")`.
                        panic_with_error!(&env, Error::WouldOrphanIlk);
                    }
                } else {
                    panic_with_error!(&env, Error::UnrecognizedParameter);
                }
                write_ilk(&env, &ilk_id, &ilk);
                FileIlk { ilk_id, what, data }.publish(&env);
            }
        }
    }

    /// Shuts the converter down. Mirrors `cage`.
    pub fn cage(env: Env, caller: Address) {
        caller.require_auth();
        require_role(&env, &WARD, &caller);

        env.storage().instance().set(&DataKey::Live, &0u32);

        Cage {}.publish(&env);
    }
}

#[contractimpl]
impl PriceConverter {
    /* ========================== POKING ========================== */

    /// Recalculates a collateral type's price factor and writes it to the Vault Engine. Mirrors `poke`.
    pub fn poke(env: Env, ilk_id: BytesN<32>) {
        let ilk = read_ilk(&env, &ilk_id);

        // In Solidity `msg.sender` here is this contract; Soroban exposes it as the contract's own address, and
        // a direct sub-invocation is automatically authorized by the calling contract.
        let this = env.current_contract_address();

        let (val, has) = if ilk.fixed_price {
            // Supported stablecoin: the price is pinned to $1, no oracle lookup.
            (wad_unit(&env), true)
        } else {
            // Oracle-backed collateral: the ilk must have an oracle assigned.
            match ilk.pip {
                None => panic_with_error!(&env, Error::InvalidAddress),
                Some(pip) => OsmClient::new(&env, &pip).peek(&this, &ilk_id),
            }
        };

        // If the price is invalid, the price factor is set to ZERO, freezing new minting against this
        // collateral until a valid price returns (a zero spot makes every mint/withdraw fail the safety check).
        let spot = if has {
            let scaled = val
                .mul(&I256::from_i128(&env, 1_000_000_000))
                .mul(&ray_unit(&env));
            let after_par = scaled.div(&PriceConverter::par(env.clone()));
            after_par.mul(&ray_unit(&env)).div(&ilk.mat)
        } else {
            zero(&env)
        };

        let vault_engine = PriceConverter::vault_engine(env.clone());
        VaultEngineClient::new(&env, &vault_engine).file_ilk(
            &this,
            &ilk_id,
            &sym(&env, "spot"),
            &FileValue::Num(spot.clone()),
        );

        Poke { ilk_id, val, spot }.publish(&env);
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

/// Reads a collateral type's oracle configuration, defaulting to unconfigured.
fn read_ilk(env: &Env, ilk_id: &BytesN<32>) -> IlkOracle {
    storage::get(env, &DataKey::Ilk(ilk_id.clone())).unwrap_or_else(|| IlkOracle {
        pip: None,
        mat: zero(env),
        fixed_price: false,
    })
}

/// Writes a collateral type's oracle configuration.
fn write_ilk(env: &Env, ilk_id: &BytesN<32>, ilk: &IlkOracle) {
    storage::set(env, &DataKey::Ilk(ilk_id.clone()), ilk);
}
