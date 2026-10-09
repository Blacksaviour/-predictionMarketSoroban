/* eslint-disable @typescript-eslint/no-explicit-any */

/**
 * Core types for the Rain USDR Soroban contract suite.
 *
 * These mirror the Rust `contracttype` structs defined in
 * `contracts/shared/src/types.rs`, `contracts/vault-engine/src/lib.rs`
 * and the other contract crates.
 */

/** 18-decimal fixed-point scalar (token quantities). Mirrors `_WAD`. */
export type Wad = string; // I256 serialized as a { lo, hi } or as a hex string
/** 27-decimal fixed-point scalar (rates, price factors). Mirrors `_RAY`. */
export type Ray = string;
/** 45-decimal fixed-point scalar (internal USDR balances). Mirrors `_RAD`. */
export type Rad = string;

/** The tagged union accepted by the `file` family of setters. */
export enum FileValueKind {
  Num = "Num", // uint256 overload
  Addr = "Addr", // address overload
}

export interface FileValue {
  kind: FileValueKind;
  num?: string; // used when kind === Num
  addr?: string; // used when kind === Addr
}

/** A collateral type and its risk settings. Mirrors `Ilk` struct. */
export interface Ilk {
  global_art: string; // I256
  global_ink: string; // I256
  rate: string; // I256 — fixed at RAY (1.0)
  spot: string; // I256 — price factor [ray]
  line: string; // I256 — debt ceiling [rad]
  dust: string; // I256 — minimum vault debt [rad]
}

/** A single collateralized position. Mirrors `Urn` struct. */
export interface Urn {
  ink: string; // I256 — locked collateral [wad]
  art: string; // I256 — normalized debt [wad]
}

/** Oracle configuration for a collateral type. Mirrors `IlkOracle`. */
export interface IlkOracle {
  pip: string | null; // Option<Address>
  mat: string; // I256 — required collateral ratio [ray]
  fixed_price: boolean;
}

/** A scheduled parameter change. Mirrors `Change` struct. */
export interface Change {
  target: string | null; // Option<Address>
  fn_name: string; // Symbol
  args: any[]; // Vec<Val>
  eta: number;
  executed: boolean;
  cancelled: boolean;
}

/** A live auction. Mirrors `Sale` struct. */
export interface Sale {
  pos: number;
  tab: string; // I256
  lot: string; // I256
  vault_id: number;
  usr: string;
  tic: number;
  top: string; // I256
}

/** Liquidation settings for a collateral type. Mirrors `IlkLiquidation`. */
export interface IlkLiquidation {
  clip: string | null; // Option<Address>
  chop: string; // I256
  hole: string; // I256
  dirt: string; // I256
  bark_factor: string; // I256
}

/** A delayed price entry. Mirrors `Feed` struct. */
export interface Feed {
  val: string; // I256
  has: boolean;
}

/** Oracle state per collateral type. Mirrors `OsmIlk` struct. */
export interface OsmIlk {
  src: string | null; // Option<Address>
  cur: Feed;
  nxt: Feed;
  delay: number;
  stopped: boolean;
}

/** Configuration of a registered stablecoin ilk. Mirrors `PsmIlk` struct. */
export interface PsmIlk {
  token: string;
  to18_conversion_factor: string; // I256
  vault_id: number;
}

/** Configuration and state of a registered adapter ilk. Mirrors `AdapterIlk`. */
export interface AdapterIlk {
  token: string;
  dec: number;
  is_usdr: boolean;
  live: number;
}

/** A 32-byte collateral identifier (BytesN<32>). */
export type IlkId = string;

// --- Contract error response ---

/** Error returned by Soroban when a contract invocation fails. */
export interface SorobanContractError {
  type: "contract";
  code: number;
  message: string;
}

/** Parsed error from a Soroban transaction failure. */
export interface ParsedError {
  raw: string;
  contract_revert?: string;
  contract_error_code?: number;
  contract_error_message?: string;
  transaction_error?: string;
}

// --- Wallet ---

export interface StellarWalletState {
  publicKey: string | null;
  isConnected: boolean;
  isConnecting: boolean;
  hasFreighter: boolean;
  error: string | null;
}

// --- Contract operation result ---

export interface ContractOperationResult {
  success: boolean;
  loading: boolean;
  error: string | null;
  data: any | null;
}
