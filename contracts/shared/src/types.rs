//! Data structures shared between contracts.
//!
//! Port of the `struct` declarations found in the Solidity `contracts/interfaces` directory. They live here so
//! that both a contract implementation and the generated cross-contract clients in the `interfaces` crate can
//! refer to the exact same Rust type.
//!
//! Unit conventions (unchanged from the Solidity source):
//! * `wad` values have 18 decimals,
//! * `ray` values have 27 decimals,
//! * `rad` values have 45 decimals.

use soroban_sdk::{contracttype, Address, BytesN, Symbol, Val, Vec, I256};

/// A collateral type and its risk settings. Mirrors `IVaultEngine.Ilk`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ilk {
    /// Total normalized debt issued against this collateral [wad].
    pub global_art: I256,
    /// Total collateral locked in vaults of this collateral type [wad].
    pub global_ink: I256,
    /// Debt multiplier. Fixed at RAY (1.0) since USDR charges no stability fee [ray].
    pub rate: I256,
    /// Maximum USDR mintable per unit of collateral (price factor) [ray].
    pub spot: I256,
    /// Debt ceiling for this collateral type [rad].
    pub line: I256,
    /// Minimum vault debt size [rad].
    pub dust: I256,
}

/// A single collateralized position. Mirrors `IVaultEngine.Urn`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Urn {
    /// Amount of collateral locked in the vault [wad].
    pub ink: I256,
    /// Normalized debt of the vault [wad].
    pub art: I256,
}

/// Oracle configuration for a collateral type. Mirrors `IPriceConverter.IlkOracle`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IlkOracle {
    /// The collateral's Oracle Security Module. `None` for fixed-price ilks.
    pub pip: Option<Address>,
    /// The required collateralization ratio [ray].
    pub mat: I256,
    /// Whether the ilk is a supported stablecoin pinned to $1 (no oracle).
    pub fixed_price: bool,
}

/// A delayed price entry. Mirrors `IOracleSecurityModule.Feed`.
///
/// The Solidity original stores the price as `uint128` inside a `bytes32`. Soroban has a native 256-bit integer,
/// so the value is kept as [`I256`] and the pointless packing into `bytes32` is dropped.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Feed {
    /// The price value [wad].
    pub val: I256,
    /// Whether the feed currently holds a value.
    pub has: bool,
}

/// Oracle state per collateral type. Mirrors `IOracleSecurityModule.Ilk`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OsmIlk {
    /// The per-ilk price source.
    pub src: Option<Address>,
    /// The current (delayed) price.
    pub cur: Feed,
    /// The next price, promoted to `cur` on the following poke.
    pub nxt: Feed,
    /// The block-aligned timestamp of the last poke.
    pub delay: u64,
    /// Whether the collateral's feed is stopped.
    pub stopped: bool,
}

/// Configuration and state of a registered adapter ilk. Mirrors `ICollateralAdapter.Ilk`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterIlk {
    /// The token this ilk bridges, held in custody, or minted and burned for USDR.
    pub token: Address,
    /// Decimals of the token.
    pub dec: u32,
    /// Whether this ilk is the USDR ilk (`move` plus mint and burn) or a collateral ilk (`slip` plus custody).
    pub is_usdr: bool,
    /// Ilk liveness flag (`1` while live, `0` after shutdown).
    pub live: u32,
}

/// Configuration of a registered stablecoin ilk. Mirrors `IPegStabilityModule.Ilk`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PsmIlk {
    /// The stablecoin (USDT or USDC).
    pub token: Address,
    /// Decimal conversion factor between the stablecoin and 18 decimals.
    pub to18_conversion_factor: I256,
    /// The PSM's dedicated vault for this ilk in the Vault Engine.
    pub vault_id: u64,
}

/// A live auction. Mirrors `IDutchAuction.Sale`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sale {
    /// Index in the active auctions array.
    pub pos: u64,
    /// Amount of USDR to recover, including the penalty [rad].
    pub tab: I256,
    /// Collateral for sale [wad].
    pub lot: I256,
    /// Identifier of the vault the collateral was seized from.
    pub vault_id: u64,
    /// Vault owner who receives any leftover collateral.
    pub usr: Address,
    /// Auction start time.
    pub tic: u64,
    /// Starting price [ray].
    pub top: I256,
}

/// Liquidation settings for a collateral type. Mirrors `ILiquidationTrigger.IlkLiquidation`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IlkLiquidation {
    /// The Dutch auction contract for this collateral.
    pub clip: Option<Address>,
    /// The liquidation penalty [wad].
    pub chop: I256,
    /// The maximum active liquidation size for this collateral [rad].
    pub hole: I256,
    /// The amount currently being auctioned for this collateral [rad].
    pub dirt: I256,
    /// Fraction of the required collateral ratio at which a vault becomes liquidatable [wad].
    pub bark_factor: I256,
}

/// A scheduled parameter change. Mirrors `IGovernor.Change`.
///
/// Soroban has no EVM-style `bytes calldata`; an arbitrary call is described by the target contract, the
/// function name and the encoded argument vector, which `Governor.execute` dispatches with
/// `Env::invoke_contract`.
#[contracttype]
#[derive(Clone)]
pub struct Change {
    /// Contract to call.
    pub target: Option<Address>,
    /// Name of the function to call on `target`.
    pub fn_name: Symbol,
    /// Encoded arguments of the call.
    pub args: Vec<Val>,
    /// Earliest execution time.
    pub eta: u64,
    /// Whether the change has already been executed.
    pub executed: bool,
    /// Whether the change has been cancelled.
    pub cancelled: bool,
}

/// A 32-byte collateral identifier, aliased for readability in signatures.
pub type IlkId = BytesN<32>;

/// The payload accepted by the `file` family of setters.
///
/// Solidity uses function overloading (`file(bytes32,uint256)` / `file(bytes32,address)`). Rust traits cannot be
/// overloaded and Soroban's exported ABI has no overloading either, so each setter is exposed with a fixed name
/// and a tagged value. `Num` corresponds to the `uint256` overload and `Addr` to the `address` overload.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FileValue {
    /// The `uint256` overload.
    Num(I256),
    /// The `address` overload.
    Addr(Address),
}
