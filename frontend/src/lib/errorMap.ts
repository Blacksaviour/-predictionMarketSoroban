/**
 * Human-readable mapping of Soroban contract error codes.
 *
 * The full error enum is defined in `contracts/shared/src/errors.rs`.
 * These are the codes used across the ported contracts. When the contract
 * reverts, the RPC returns an error that we try to decode to one of these
 * codes so the UI can show a user-friendly message.
 */

export interface ContractErrorInfo {
  code: number;
  name: string;
  description: string;
  userMessage: string;
}

export const CONTRACT_ERRORS: Record<number, ContractErrorInfo> = {
  /* ===================== shared/Errors.sol ===================== */
  1: {
    code: 1,
    name: "IlkAlreadyInitialized",
    description: "A collateral type has already been initialized.",
    userMessage: "This collateral type has already been set up in the Vault Engine.",
  },
  2: {
    code: 2,
    name: "InvalidAddress",
    description: "An address is zero or an unset dependency.",
    userMessage: "An address used in this operation is invalid or not set.",
  },
  3: {
    code: 3,
    name: "InvalidAmount",
    description: "An amount is zero or invalid.",
    userMessage: "The amount you entered is zero or invalid.",
  },
  4: {
    code: 4,
    name: "InvalidAssignment",
    description: "An assignment is invalid (e.g. state == newState).",
    userMessage: "The state assignment is invalid.",
  },
  5: {
    code: 5,
    name: "InvalidBytes",
    description: "A bytes value is zero (e.g. bytes32(0)).",
    userMessage: "A required value is empty.",
  },
  6: {
    code: 6,
    name: "NotAuthorized",
    description: "The caller lacks a required role.",
    userMessage: "You are not authorized to perform this action.",
  },
  7: {
    code: 7,
    name: "NotLive",
    description: "The contract has been shut down (cage).",
    userMessage: "The system has been shut down and this operation is unavailable.",
  },
  8: {
    code: 8,
    name: "UnrecognizedParameter",
    description: "An unrecognized parameter name in a file call.",
    userMessage: "The parameter name is not recognized.",
  },
  9: {
    code: 9,
    name: "SystemPaused",
    description: "The Governor's emergency pause is active.",
    userMessage: "The system is currently paused. Please try again later.",
  },
  10: {
    code: 10,
    name: "SolvencyGateActive",
    description: "The solvency invariant is breached and some operations are gated.",
    userMessage: "The protocol's solvency is currently compromised.",
    },

  /* ===================== libraries/Math.sol ===================== */
  20: { code: 20, name: "AddOverflow", description: "An addition overflowed the max I256 value.", userMessage: "An arithmetic overflow occurred during addition." },
  21: { code: 21, name: "AddUnderflow", description: "An addition underflowed the min I256 value.", userMessage: "An arithmetic underflow occurred during addition." },
  22: { code: 22, name: "SubOverflow", description: "A subtraction overflowed the max I256 value.", userMessage: "An arithmetic overflow occurred during subtraction." },
  23: { code: 23, name: "SubUnderflow", description: "A subtraction underflowed the min I256 value.", userMessage: "An arithmetic underflow occurred during subtraction." },
  24: { code: 24, name: "MulOverflow", description: "A multiplication overflowed the max I256 value.", userMessage: "An arithmetic overflow occurred during multiplication." },

  /* ===================== IVaultEngine.sol ===================== */
  30: {
    code: 30,
    name: "VaultNotFound",
    description: "The vault does not exist.",
    userMessage: "The vault you are trying to access does not exist.",
  },
  31: {
    code: 31,
    name: "IlkNotInitialized",
    description: "The collateral type has not been initialized.",
    userMessage: "This collateral type has not been initialized. You need to init it first.",
  },
  32: {
    code: 32,
    name: "NotLiveVat",
    description: "The Vault Engine is not live.",
    userMessage: "The Vault Engine is currently not live.",
  },
  33: {
    code: 33,
    name: "CeilingExceeded",
    description: "The debt ceiling for this collateral type has been exceeded.",
    userMessage: "Drawing this amount would exceed the collateral type's debt ceiling.",
  },
  34: {
    code: 34,
    name: "NotSafe",
    description: "The vault would be undercollateralized.",
    userMessage: "This action would leave the vault undercollateralized. Reduce the debt or add more collateral.",
  },
  35: {
    code: 35,
    name: "NotAllowed",
    description: "The caller is not allowed to act on behalf of the user.",
    userMessage: "You are not authorized to perform this action on this vault. Use 'Hope' to grant permission.",
  },
  36: {
    code: 36,
    name: "DustAmount",
    description: "The vault debt would be below the dust threshold.",
    userMessage: "The resulting vault debt is below the minimum (dust) threshold.",
  },

  /* ===================== ICollateralAdapter.sol ===================== */
  40: {
    code: 40,
    name: "InvalidDecimals",
    description: "The token has unsupported decimals.",
    userMessage: "The token decimals are not supported.",
  },
  41: {
    code: 41,
    name: "FeeOnTransferToken",
    description: "The token charges a fee on transfer.",
    userMessage: "This token charges fees on transfer and cannot be used directly.",
  },

  /* ===================== IPriceConverter.sol ===================== */
  50: {
    code: 50,
    name: "MatBelowOne",
    description: "The collateralization ratio is below 100%.",
    userMessage: "The collateralization ratio (mat) must be at least 1.0 (100%).",
  },
  51: {
    code: 51,
    name: "WouldOrphanIlk",
    description: "Clearing the fixed flag on an ilk with no oracle would freeze its price.",
    userMessage: "This ilk has no oracle assigned. Set an oracle via file_ilk before clearing the fixed flag.",
  },

  /* IOracleSecurityModule.sol */
  60: { code: 60, name: "NotPassed", description: "The oracle price has not passed the required delay.", userMessage: "The delayed oracle price is not yet available." },
  61: { code: 61, name: "NoCurrentValue", description: "The oracle has no current value.", userMessage: "The oracle has no current price." },

  /* IReserveAccounting.sol */
  70: { code: 70, name: "ReserveBelowEscrow", description: "The reserve would drop below the committed escrow.", userMessage: "The reserve cannot drop below the committed escrow amount." },
  71: { code: 71, name: "EscrowExceedsReserve", description: "The committed escrow exceeds the total reserve.", userMessage: "The committed escrow cannot exceed the total reserve." },

  /* IBalanceSheet.sol */
  80: { code: 80, name: "InsufficientSurplus", description: "Insufficient surplus funds.", userMessage: "There is not enough surplus for this operation." },
  81: { code: 81, name: "InsufficientDebt", description: "Insufficient debt funds.", userMessage: "There is not enough debt for this operation." },
  82: { code: 82, name: "OutstandingBadDebt", description: "There is outstanding bad debt.", userMessage: "There is outstanding bad debt that needs to be resolved first." },
  83: { code: 83, name: "NoBuybackReceiver", description: "No buyback receiver is configured.", userMessage: "No buyback receiver is configured for this operation." },
  84: { code: 84, name: "WaitNotElapsed", description: "The timelock wait period has not elapsed.", userMessage: "The timelock wait period has not elapsed yet." },

  /* ISolvencyEngine.sol */
  90: { code: 90, name: "ParameterOutOfBounds", description: "A parameter is out of bounds.", userMessage: "One of the parameters is out of bounds." },
  91: { code: 91, name: "ExposureCapNotSet", description: "The exposure cap has not been set.", userMessage: "The exposure cap has not been set." },

  /* IPegStabilityModule.sol */
  100: { code: 100, name: "InsufficientFreeSlack", description: "The free slack would go negative.", userMessage: "There is not enough free slack for this operation." },

  /* ILiquidationTrigger.sol */
  110: { code: 110, name: "InvalidThrottle", description: "An invalid liquidation throttle value.", userMessage: "The liquidation throttle value is invalid." },
  111: { code: 111, name: "ChopBelowOne", description: "The liquidation penalty is below 100%.", userMessage: "The liquidation penalty must be at least 100%." },
  112: { code: 112, name: "InvalidBarkFactor", description: "The bark factor is invalid.", userMessage: "The liquidation bark factor is invalid." },
  113: { code: 113, name: "NotUnsafe", description: "The vault is not unsafe (not liquidatable).", userMessage: "This vault is not unsafe and cannot be liquidated." },
  114: { code: 114, name: "LiquidationLimitHit", description: "The liquidation limit has been hit.", userMessage: "The liquidation limit for this collateral type has been reached." },
  115: { code: 115, name: "DustyAuction", description: "The auction is too small to be worthwhile.", userMessage: "The auction amount is too small (dusty)." },
  116: { code: 116, name: "NullAuction", description: "The auction has no collateral to sell.", userMessage: "The auction has no collateral lot." },
  117: { code: 117, name: "Overflow", description: "An arithmetic overflow occurred.", userMessage: "An arithmetic overflow occurred during the operation." },

  /* IDutchAuction.sol */
  120: { code: 120, name: "Stopped", description: "The auction contract is stopped.", userMessage: "Trading on this auction is currently stopped." },
  121: { code: 121, name: "ZeroTab", description: "The tab (USDR to recover) is zero.", userMessage: "The auction tab is zero." },
  122: { code: 122, name: "ZeroLot", description: "The lot (collateral for sale) is zero.", userMessage: "The auction lot is zero." },
  123: { code: 123, name: "ZeroUser", description: "The user is zero.", userMessage: "The user address is zero." },
  124: { code: 124, name: "ZeroTopPrice", description: "The starting price is zero.", userMessage: "The auction starting price is zero." },
  125: { code: 125, name: "AuctionNotRunning", description: "The auction is not currently running.", userMessage: "This auction is not currently running." },
  126: { code: 126, name: "CannotReset", description: "The auction cannot be reset.", userMessage: "This auction cannot be reset." },
  127: { code: 127, name: "NeedsReset", description: "The auction needs a reset first.", userMessage: "This auction needs to be reset first." },
  128: { code: 128, name: "TooExpensive", description: "The offered price is too high.", userMessage: "The offered bid is above the current auction price." },
  129: { code: 129, name: "NoPartialPurchase", description: "Partial purchase is not allowed.", userMessage: "Partial purchases are not allowed." },
  130: { code: 130, name: "InvalidPrice", description: "The provided price is invalid.", userMessage: "The provided price is invalid." },

  /* IGovernor.sol */
  140: { code: 140, name: "NotScheduled", description: "The change is not scheduled.", userMessage: "This change is not scheduled in the Governor." },
  141: { code: 141, name: "ChangeCancelled", description: "The scheduled change has been cancelled.", userMessage: "This change has been cancelled." },
  142: { code: 142, name: "AlreadyExecuted", description: "The change has already been executed.", userMessage: "This change has already been executed." },
  143: { code: 143, name: "DelayNotElapsed", description: "The timelock delay has not elapsed.", userMessage: "The timelock delay has not elapsed yet." },
  144: { code: 144, name: "ExecutionFailed", description: "The scheduled execution failed.", userMessage: "The scheduled change execution failed." },
  145: { code: 145, name: "AlreadyPaused", description: "The system is already paused.", userMessage: "The system is already paused." },
  146: { code: 146, name: "NotPaused", description: "The system is not paused.", userMessage: "The system is not currently paused." },

  /* IEnd.sol */
  150: { code: 150, name: "AlreadyCaged", description: "The system is already shut down.", userMessage: "The system has already been shut down (caged)." },
  151: { code: 151, name: "StillLive", description: "The system is still live.", userMessage: "The system is still live." },
  152: { code: 152, name: "TagAlreadyDefined", description: "The price feed tag already exists.", userMessage: "This price feed tag is already defined." },
  153: { code: 153, name: "TagNotDefined", description: "The price feed tag is not defined.", userMessage: "This price feed tag is not defined." },
  154: { code: 154, name: "ArtNotZero", description: "Vault normalized debt is not zero.", userMessage: "The vault's debt must be zero for this operation." },
  155: { code: 155, name: "DebtAlreadyFixed", description: "The debt has already been fixed.", userMessage: "The debt has already been fixed." },
  156: { code: 156, name: "SurplusNotZero", description: "Surplus is not zero.", userMessage: "Surplus must be zero for this operation." },
  157: { code: 157, name: "DebtNotFixed", description: "The debt has not been fixed.", userMessage: "The debt has not been fixed yet." },
  158: { code: 158, name: "FixAlreadyDefined", description: "The debt fix is already defined.", userMessage: "This debt fix is already defined." },
  159: { code: 159, name: "FixNotDefined", description: "The debt fix is not defined.", userMessage: "This debt fix is not defined." },
  160: { code: 160, name: "InsufficientBag", description: "Insufficient bag balance.", userMessage: "Insufficient bag balance for this operation." },

  /* ERC-20 */
  170: { code: 170, name: "InsufficientBalance", description: "Insufficient token balance.", userMessage: "You do not have enough balance for this operation." },
  171: { code: 171, name: "InsufficientAllowance", description: "Insufficient allowance.", userMessage: "You have not approved enough tokens for this operation." },
  172: { code: 172, name: "InvalidSender", description: "Invalid sender address.", userMessage: "The sender address is invalid." },
  173: { code: 173, name: "InvalidReceiver", description: "Invalid receiver address.", userMessage: "The receiver address is invalid." },
  174: { code: 174, name: "ExpiredAllowance", description: "The allowance has expired.", userMessage: "The token allowance has expired." },
};

export function parseSorobanError(error: unknown): string {
  if (error instanceof Error) {
    const msg = error.message;
    const codeMatch = msg.match(/(?:code|Code):?\s*(\d+)/i);
    if (codeMatch) {
      const code = parseInt(codeMatch[1], 10);
      const info = CONTRACT_ERRORS[code];
      if (info) return info.userMessage;
    }
    for (const entry of Object.values(CONTRACT_ERRORS)) {
      if (msg.includes(entry.name)) return entry.userMessage;
    }
    const lower = msg.toLowerCase();
    if (lower.includes("timeout")) return "The transaction timed out. Please try again.";
    if (lower.includes("insufficient")) return "Insufficient funds for the transaction fee or operation.";
    if (lower.includes("freighter")) return "Freighter wallet is not installed or not connected.";
    if (lower.includes("not unlocked")) return "Freighter is not unlocked. Please unlock your wallet.";
    if (lower.includes("simulation")) return "Transaction simulation failed. Check your inputs and try again.";
    return msg;
  }
  return "An unexpected error occurred. Please try again.";
}

export const errorMap = CONTRACT_ERRORS;

