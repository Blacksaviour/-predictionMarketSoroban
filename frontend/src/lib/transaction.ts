/**
 * Transaction Builder Utilities
 *
 * Builds, simulates, and submits Soroban transactions via the Freighter
 * wallet extension.
 *
 * Standard flow:
 * 1. buildSorobanOperation - create a contract-call operation
 * 2. buildTransaction - create a TransactionBuilder configured with account,
 *    network, and operations
 * 3. simulateTx - dry-run the transaction to get Soroban auth data
 * 4. signAndSubmit - hand the XDR to Freighter for signing and submission
 * 5. pollTxResult - poll for the outcome of the submitted transaction
 */

import { Server as SorobanRpc } from "@stellar/stellar-sdk/rpc";
import {
  TransactionBuilder,
  SorobanDataBuilder,
  xdr,
  Account,
  Transaction,
  FeeBumpTransaction,
} from "@stellar/stellar-sdk/base";
import { rpc, PASSPHRASE } from "@/lib/soroban";
import { CONTRACT_IDS } from "@/lib/config";

/** Fee in stroops (1 stroop = 0.00001 XLM; default Freighter fee = 100 stroops). */
const DEFAULT_FEE = 100;

/** Convert a JS number or string to a hex string for BytesN<32> arguments. */
function toBytes32(val: string | number | Buffer): string {
  if (typeof val === "string") {
    if (val.startsWith("0x")) {
      return val.slice(2);
    }
    return Buffer.from(val, "utf8").toString("hex");
  }
  return val.toString(16).padStart(64, "0");
}

/**
 * Build a Soroban contract-invocation operation.
 *
 * Uses the Contract class from stellar-sdk/base to construct the
 * invokeContract operation with properly typed arguments.
 */
export function buildSorobanOperation(
  contractId: string,
  methodName: string,
  args: unknown[] = [],
): xdr.Operation {
  // Convert all JS args to ScVal using nativeToScVal
  const scArgs: any[] = args.map((arg) => {
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
        // Decimal: convert to big integer by scaling
        return xdr.ScVal.scvU128(BigInt(Math.floor(Math.abs(num) * 1e18)));
      }
      return xdr.ScVal.scvSymbol(xdr.Symbol.fromString(arg));
    }
    if (arg instanceof Buffer || arg instanceof Uint8Array) {
      return xdr.ScVal.scvBytes(new Uint8Array(arg));
    }
    // Default: string
    return xdr.ScVal.scvSymbol(xdr.Symbol.fromString(String(arg)));
  });

  // Build invokeContract operation
  const invoke = xdr.ContractFunctionArgs({
    contractId,
    function: methodName,
    args: scArgs,
  });

  return xdr.Operation.createInvokeContract(invoke);
}

/**
 * Build a Soroban transaction (unsigned) as a TransactionEnvelope XDR.
 *
 * Uses the TransactionBuilder from stellar-sdk/base.
 *
 * @param account - The account to build the transaction from
 * @param operations - Operations to add
 * @returns The XDR string of the unsigned transaction
 */
export function buildTransaction(
  account: Account,
  operations: xdr.Operation[],
): string {
  const builder = new TransactionBuilder(account, {
    fee: `${DEFAULT_FEE}`,
    networkPassphrase: PASSPHRASE,
    memos: [xdr.Memo.text("")],
  })
    .setTimeout(30)
    .addOperations(operations);

  const transaction = builder.build();
  return transaction.toXDR();
}

/**
 * Simulate a transaction (dry run) to get the SorobanData (fee, authorizations,
 * vlas) needed for the actual signed transaction.
 *
 * @param account - The account to build the transaction from
 * @param operations - Operations to add
 * @returns Transaction with SorobanData or error message
 */
export async function simulateTx(
  account: Account,
  operations: xdr.Operation[],
): Promise<{ error?: string; transaction?: any; result?: any[] }> {
  try {
    const builder = new TransactionBuilder(account, {
      fee: `${DEFAULT_FEE}`,
      networkPassphrase: PASSPHRASE,
      memos: [xdr.Memo.text("")],
    })
      .setTimeout(30)
      .addOperations(operations);

    const transaction = builder.build();

    const response = await rpc.simulateTransaction(transaction);

    // Check if simulation threw an exception
    if (response.exceptions && response.exceptions.length > 0) {
      return {
        error: response.exceptions.map((e: any) => e.description).join(", "),
      };
    }

    return {
      transaction: response.transaction,
      result: response.results,
    };
  } catch (err: any) {
    return {
      error: err?.message || String(err),
    };
  }
}

/**
 * Submit a signed Soroban transaction to the network.
 *
 * Freighter signs the transaction, then we submit the XDR to the RPC.
 */
export async function submitTransaction(signedXdr: string): Promise<string> {
  const response = await rpc.sendTransaction(signedXdr);
  return response.hash || (response as any).result?.hash || String(response);
}

/**
 * Poll for a transaction result by hash.
 *
 * @param hash - Transaction hash
 * @param timeoutMs - Max wait time in ms
 * @param intervalMs - Polling interval in ms
 */
export async function pollTxResult(
  hash: string,
  timeoutMs: number = 30000,
  intervalMs: number = 2000,
): Promise<{ successful: boolean; result?: any; error?: string }> {
  const start = Date.now();

  while (Date.now() - start < timeoutMs) {
    try {
      const result = await rpc.getTransaction(hash);
      const status = result?.status;

      if (status === "SUCCESS") {
        return { successful: true, result };
      }
      if (status === "FAILED") {
        return { successful: false, error: "Transaction failed on network" };
      }
      // Still pending
      await new Promise((resolve) => setTimeout(resolve, intervalMs));
    } catch (err: any) {
      // Transaction might not be found yet
      await new Promise((resolve) => setTimeout(resolve, intervalMs));
    }
  }

  return { successful: false, error: "Transaction polling timed out" };
}
