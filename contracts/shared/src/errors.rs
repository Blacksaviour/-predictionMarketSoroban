//! Free errors shared across contracts.
//!
//! Port of `contracts/shared/Errors.sol`, plus the per-contract custom errors that Solidity declares inside
//! interfaces/libraries. `#[contracterror]` requires every variant to live in a single enum with explicit
//! discriminants, so the full error surface of the system is collected here.

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /* ===================== shared/Errors.sol ===================== */
    /// An ilk has already been initialized.
    IlkAlreadyInitialized = 1,
    /// A failure with an address, for example `address(0)` / an unset dependency.
    InvalidAddress = 2,
    /// A failure with an amount, for example `0`.
    InvalidAmount = 3,
    /// A failure with an assignment, for example `state == newState`.
    InvalidAssignment = 4,
    /// A failure with a bytes value, for example `bytes32(0)`.
    InvalidBytes = 5,
    /// The caller lacks a required role.
    NotAuthorized = 6,
    /// The contract has been shut down and the operation is unavailable.
    NotLive = 7,
    /// Unrecognized parameter name in a `file` call.
    UnrecognizedParameter = 8,
    /// The Governor's emergency pause is active.
    SystemPaused = 9,
    /// The solvency invariant is breached and reserve-decreasing operations are gated.
    SolvencyGateActive = 10,

    /* ===================== libraries/Math.sol ===================== */
    AddOverflow = 20,
    AddUnderflow = 21,
    SubOverflow = 22,
    SubUnderflow = 23,
    MulOverflow = 24,

    /* ===================== IVaultEngine.sol ===================== */
    VaultNotFound = 30,
    IlkNotInitialized = 31,
    NotLiveVat = 32,
    CeilingExceeded = 33,
    NotSafe = 34,
    NotAllowed = 35,
    DustAmount = 36,

    /* ===================== ICollateralAdapter.sol ===================== */
    InvalidDecimals = 40,
    FeeOnTransferToken = 41,

    /* ===================== IPriceConverter.sol ===================== */
    MatBelowOne = 50,
    WouldOrphanIlk = 51,

    /* ===================== IOracleSecurityModule.sol ===================== */
    NotPassed = 60,
    NoCurrentValue = 61,

    /* ===================== IReserveAccounting.sol ===================== */
    ReserveBelowEscrow = 70,
    EscrowExceedsReserve = 71,

    /* ===================== IBalanceSheet.sol ===================== */
    InsufficientSurplus = 80,
    InsufficientDebt = 81,
    OutstandingBadDebt = 82,
    NoBuybackReceiver = 83,
    WaitNotElapsed = 84,

    /* ===================== ISolvencyEngine.sol ===================== */
    ParameterOutOfBounds = 90,
    ExposureCapNotSet = 91,

    /* ===================== IPegStabilityModule.sol ===================== */
    InsufficientFreeSlack = 100,

    /* ===================== ILiquidationTrigger.sol ===================== */
    InvalidThrottle = 110,
    ChopBelowOne = 111,
    InvalidBarkFactor = 112,
    NotUnsafe = 113,
    LiquidationLimitHit = 114,
    DustyAuction = 115,
    NullAuction = 116,
    Overflow = 117,

    /* ===================== IDutchAuction.sol ===================== */
    Stopped = 120,
    ZeroTab = 121,
    ZeroLot = 122,
    ZeroUser = 123,
    ZeroTopPrice = 124,
    AuctionNotRunning = 125,
    CannotReset = 126,
    NeedsReset = 127,
    TooExpensive = 128,
    NoPartialPurchase = 129,
    InvalidPrice = 130,

    /* ===================== IGovernor.sol ===================== */
    NotScheduled = 140,
    ChangeCancelled = 141,
    AlreadyExecuted = 142,
    DelayNotElapsed = 143,
    ExecutionFailed = 144,
    AlreadyPaused = 145,
    NotPaused = 146,

    /* ===================== IEnd.sol ===================== */
    AlreadyCaged = 150,
    StillLive = 151,
    TagAlreadyDefined = 152,
    TagNotDefined = 153,
    ArtNotZero = 154,
    DebtAlreadyFixed = 155,
    SurplusNotZero = 156,
    DebtNotFixed = 157,
    FixAlreadyDefined = 158,
    FixNotDefined = 159,
    InsufficientBag = 160,

    /* ===================== ERC-20 (OpenZeppelin v5) ===================== */
    InsufficientBalance = 170,
    InsufficientAllowance = 171,
    InvalidSender = 172,
    InvalidReceiver = 173,
    ExpiredAllowance = 174,
}
