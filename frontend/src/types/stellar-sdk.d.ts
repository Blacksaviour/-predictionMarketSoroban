/**
 * Ambient type declarations for `@stellar/stellar-sdk` subpath imports.
 *
 * The installed stellar-sdk (v17.2.1) does not ship `.d.ts` files for its
 * `./rpc` and `./base` subpath entrypoints (its `types` field points to
 * files that don't exist in the published package). TypeScript therefore
 * raises TS7016 ("Could not find a declaration file") for every import.
 *
 * These ambient shims provide the minimal surface the app uses so the
 * production build (`next build`, which type-checks strictly) succeeds.
 * Runtime behavior is unaffected — these are compile-time only.
 *
 * TODO: Remove this file once the SDK ships proper type declarations or
 * `@types/stellar__stellar-sdk` becomes available.
 */

declare module "@stellar/freighter-api" {
  /** Result wrapper returned by most freighter-api calls. */
  export interface FreighterResult<T = any> {
    data?: T;
    error?: { code?: number | string; message?: string };
  }

  /** Request access / authorization. Returns the authorized address. */
  export function requestAccess(): Promise<
    FreighterResult<{ address: string }> & { address: string; error: any }
  >;

  /** Get the currently authorized address (no prompt). */
  export function getAddress(): Promise<
    FreighterResult<{ address: string }> & { address: string; error: any }
  >;

  /** Sign an XDR. Returns the signed XDR (does NOT submit). */
  export function signTransaction(
    entryXdr: string,
    opts?: {
      networkPassphrase?: string;
      address?: string;
      addressToSign?: string;
      signWithSecureEnclave?: boolean;
    },
  ): Promise<
    FreighterResult<{ signedTxXdr: string; signerAddress: string }> & {
      signedTxXdr: string;
      signerAddress: string;
      error: any;
    }
  >;

  /** Get the network the extension is configured for. */
  export function getNetwork(): Promise<
    FreighterResult<{ network: string; networkPassphrase: string }> & {
      network: string;
      networkPassphrase: string;
      error: any;
    }
  >;
}

declare module "@stellar/stellar-sdk/rpc" {
  /** Minimal Soroban RPC server client surface used by the app. */
  export class Server {
    constructor(url: string, opts?: any);
    /** Read-only contract query. Returns the decoded result. */
    queryContract<T = any>(
      contractId: string,
      method: string,
      args?: any,
      networkPassphrase?: string,
    ): Promise<{ result: T; isReadCall?: boolean }>;
    /** Simulate (dry-run) a transaction. */
    simulateTransaction(tx: any): Promise<any>;
    /** Assemble Soroban resource/fee data into a signed-ready transaction. */
    prepareTransaction(tx: any): Promise<any>;
    /** Submit a signed transaction. */
    sendTransaction(tx: any): Promise<any>;
    /** Fetch a transaction by hash. */
    getTransaction(hash: string): Promise<any>;
    /** Fetch network info (passphrase, etc.). */
    getNetwork(): Promise<any>;
  }
  export const Api: any;
}

declare module "@stellar/stellar-sdk/base" {
  /** XDR helpers. Declared as a namespace so it works in type & value positions. */
  export namespace xdr {
    export type Operation = any;
    export type ScVal = any;
    export type TransactionEnvelope = any;
    export type Memo = any;
    export type Symbol = any;
    export const ScVal: any;
    export const Symbol: any;
    export const Memo: any;
    export const Operation: any;
    export const ContractFunctionArgs: any;
  }

  /** Stellar account (used as both a value via `new` and a type). */
  export class Account {
    constructor(accountId: string, sequence: string);
    accountId(): string;
    sequenceNumber(): string;
  }

  export const TransactionBuilder: any;
  export const SorobanDataBuilder: any;
  export const Networks: any;
  export const hash: any;
  export const scValToNative: any;
  export const nativeToScVal: any;
  export const Transaction: any;
  export const FeeBumpTransaction: any;
  export const Contract: any;
  export const Address: any;
  export const Keypair: any;
  export const StrKey: any;
}

