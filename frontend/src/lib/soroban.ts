/**
 * Soroban RPC Client
 *
 * Initializes the `@stellar/stellar-sdk` Soroban RPC server client and
 * provides utilities for transaction simulation, submission, and decoding.
 */

import { Server } from "@stellar/stellar-sdk/rpc";
import { TransactionBuilder, SorobanDataBuilder, Account, Networks, xdr, hash, scValToNative, nativeToScVal } from "@stellar/stellar-sdk/base";
import { NETWORK, CONTRACT_IDS } from "@/lib/config";

/** Cached RPC server instance for the configured network. */
export const rpc = new Server(
  NETWORK.rpcUrl as string,
  { timeoutMs: 60000, quickRw: true },
);

/** The standard network passphrase used across the SDK. */
export const PASSPHRASE = NETWORK.networkPassphrase;

/**
 * Decode a Soroban `I256` ScVal into a native JS value.
 *
 * The contract uses I256 for all fixed-point quantities:
 * - wad (18 decimals, token amounts)
 * - ray (27 decimals, rates and price factors)
 * - rad (45 decimals, internal balances)
 */
export function decodeScVal(val: any): unknown {
  if (!val) return null;
  try {
    return nativeToScVal(val, PASSPHRASE);
  } catch {
    return val;
  }
}

/**
 * Helper: convert a user-facing decimal string (e.g. "100.5") into the
 * corresponding I256 wad value used by the contract (18 decimals).
 */
export function toWad(value: string | number): xdr.ScVal {
  const factor = BigInt("1000000000000000000");
  const scaled = BigInt(Math.floor(Number(value) * 1e9)) * factor / BigInt(1000000000);
  return xdr.ScVal.scvU256(scaled);
}

/**
 * Helper: convert a wad I256 value (18 decimals) back to a human-readable
 * decimal string.
 */
export function fromWad(val: any): string {
  try {
    const nativeVal = nativeToScVal(val, PASSPHRASE);
    const big = BigInt(nativeVal.toString());
    const div = big / BigInt("1000000000000000000");
    const rem = big % BigInt("1000000000000000000");
    const sign = big < 0 ? "-" : "";
    const absRem = rem < 0 ? -rem : rem;
    const remStr = absRem.toString().padStart(18, "0");
    return `${sign}${div}.${remStr}`;
  } catch {
    return "0";
  }
}

/**
 * Read a contract method (read-only query).
 * Uses the SorobanRpc.queryContract method from the SDK.
 */
export async function readContract<T = any>(
  contractId: string,
  func: string,
  args: unknown[] = [],
): Promise<T> {
  const result = await rpc.queryContract<T>(contractId, func, args, PASSPHRASE);
  return result.result;
}

/**
 * Simulate a write transaction (dry-run) to check for errors before signing.
 * Uses the SorobanRpc.simulateTransaction method.
 */
export async function simulateWrite(
  contractId: string,
  func: string,
  args: unknown[] = [],
): Promise<{ results: any[]; simulationXdr: string }> {
  // Build the invokeContract operation
  const operation = buildSorobanOperation(contractId, func, args);

  // Build an unsigned transaction around the operation using a dummy account.
  const account = new Account(
    "GA3KLYW2MKR5DR4ZGS6AKRCHNIUJ3AXWHQRVU6KYE7F6HEH6XVYQYHV2",
    "1",
  );
  const builtTx = new TransactionBuilder(account, {
    fee: "100",
    networkPassphrase: PASSPHRASE,
  })
    .setTimeout(30)
    .addOperation(operation)
    .build();

  // Simulate the transaction (dry-run, no signing/submission).
  const sim = await rpc.simulateTransaction(builtTx);

  if (sim.error) {
    throw new Error(sim.error);
  }

  const results = (sim as any).results || [];
  const simulationXdr = (sim as any).transactionData
    ? (sim as any).transactionData.toXDR()
    : "";

  return {
    results,
    simulationXdr,
  };
}

/**
 * Build a Soroban operation from a contract call.
 */
function buildSorobanOperation(
  contractId: string,
  methodName: string,
  args: unknown[] = [],
): any {
  const invoke = {
    contractId,
    function: methodName,
    args: args.map((arg) => {
      if (arg === null || arg === undefined) {
        return xdr.ScVal.scvVoid();
      }
      if (typeof arg === "boolean") {
        return xdr.ScVal.scvBool(arg);
      }
      if (typeof arg === "number") {
        return xdr.ScVal.scvI64(BigInt(arg));
      }
      if (typeof arg === "string") {
        if (/^-?\d+(\.\d+)?$/.test(arg)) {
          const num = Number(arg);
          if (Number.isInteger(num)) {
            return xdr.ScVal.scvI64(BigInt(num));
          }
          return xdr.ScVal.scvU128(BigInt(Math.floor(Math.abs(num) * 1e18)));
        }
        return xdr.ScVal.scvSymbol(xdr.Symbol.fromString(arg));
      }
      if (arg instanceof Uint8Array || arg instanceof Buffer) {
        return xdr.ScVal.scvBytes(new Uint8Array(arg));
      }
      return xdr.ScVal.scvSymbol(xdr.Symbol.fromString(String(arg)));
    }),
  };

  return xdr.Operation.createInvokeContract(invoke);
}
