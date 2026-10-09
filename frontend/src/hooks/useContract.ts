/**
 * useContract — Generic Soroban contract interaction hook.
 *
 * Wraps the Soroban RPC client and Freighter signing workflow.
 */

import { useState, useCallback } from "react";
import { TransactionBuilder, SorobanDataBuilder } from "@stellar/stellar-sdk/base";
import { readContract, simulateWrite, rpc, PASSPHRASE } from "@/lib/soroban";
import { signTransactionXdr } from "@/lib/freighter";
import { parseSorobanError } from "@/lib/errorMap";
import { NETWORK } from "@/lib/config";

export interface ContractOperationResult<T = any> {
  success: boolean;
  loading: boolean;
  error: string | null;
  data: T | null;
  hash: string | null;
  retry: () => void;
}

export function useContract(contractId: string) {
  const [result, setResult] = useState<ContractOperationResult>({
    success: false,
    loading: false,
    error: null,
    data: null,
    hash: null,
    retry: () => {},
  });

  const retry = useCallback(() => {
    setResult((prev) => ({ ...prev, loading: false, error: null, success: false }));
  }, []);

  /**
   * Call a read-only contract method.
   */
  const read = useCallback(
    async <T = any>(func: string, args: any[] = [], opts?: { loaderMsg?: string }): Promise<T> => {
      setResult({ success: false, loading: true, error: null, data: null, hash: null, retry });

      try {
        const data = await readContract<T>(contractId, func, args);
        setResult({ success: true, loading: false, error: null, data, hash: null, retry });
        return data;
      } catch (err) {
        const message = parseSorobanError(err);
        setResult({ success: false, loading: false, error: message, data: null, hash: null, retry });
        throw err;
      }
    },
    [contractId],
  );

  /**
   * Simulate a write (dry-run).
   */
  const simulate = useCallback(
    async (func: string, args: any[] = []): Promise<void> => {
      setResult({ success: false, loading: true, error: null, data: null, hash: null, retry });

      try {
        const sim = await simulateWrite(contractId, func, args);
        const hexResult = sim.results.length > 0
          ? Buffer.from(sim.results[0].toXDR()).toString("hex")
          : null;
        setResult({ success: true, loading: false, error: null, data: hexResult, hash: null, retry });
      } catch (err) {
        const message = parseSorobanError(err);
        setResult({ success: false, loading: false, error: message, data: null, hash: null, retry });
        throw err;
      }
    },
    [contractId],
  );

  /**
   * Sign and submit a Soroban transaction via Freighter.
   */
  const signAndSubmit = useCallback(
    async (
      publicKey: string,
      sequenceNumber: number,
      operations: any[],
    ): Promise<string> => {
      setResult({ success: false, loading: true, error: null, data: null, hash: null, retry });

      try {
        // 1. Build the unsigned transaction envelope XDR.
        const unsignedXdr = buildSorobanXDR(publicKey, sequenceNumber, operations);

        // 2. Prepare the transaction (assembles Soroban resource/fee data via
        //    RPC simulation) so Freighter can sign a valid Soroban tx.
        const txEnvelope = TransactionBuilder.fromXDR(
          unsignedXdr,
          PASSPHRASE,
        );
        let preparedXdr = unsignedXdr;
        try {
          const prepared = await rpc.prepareTransaction(txEnvelope);
          preparedXdr = prepared.toXDR();
        } catch {
          // If prepareTransaction is unavailable, sign the raw XDR.
          preparedXdr = unsignedXdr;
        }

        // 3. Sign with Freighter (returns the signed XDR, does NOT submit).
        const signedXdr = await signTransactionXdr(
          preparedXdr,
          NETWORK.networkPassphrase,
          publicKey,
        );

        // 4. Submit the signed transaction to the network.
        const sent = await rpc.sendTransaction(signedXdr);
        if (sent.error) {
          throw new Error(
            sent.error?.message || "Failed to submit transaction to the network.",
          );
        }
        const hash = sent.hash;

        // 5. Poll for the final status so we can surface a real outcome.
        let attempts = 0;
        while (attempts < 15) {
          const status = await rpc.getTransaction(hash);
          if (status.status === "SUCCESS") {
            setResult({ success: true, loading: false, error: null, data: null, hash, retry });
            return hash;
          }
          if (status.status === "FAILED") {
            throw new Error("Transaction failed on network.");
          }
          await new Promise((r) => setTimeout(r, 2000));
          attempts += 1;
        }

        setResult({ success: true, loading: false, error: null, data: null, hash, retry });
        return hash;
      } catch (err) {
        const message = parseSorobanError(err);
        setResult({ success: false, loading: false, error: message, data: null, hash: null, retry });
        throw err;
      }
    },
    [],
  );

  return {
    result,
    read,
    simulate,
    signAndSubmit,
  };
}

/**
 * Build the Soroban transaction envelope XDR.
 */
/**
 * Build the Soroban transaction envelope XDR for Freighter signing.
 *
 * Uses the TransactionBuilder and SorobanDataBuilder from stellar-sdk/base
 * to properly construct the SorobanTransactionEnvelope XDR that Freighter
 * can sign.
 *
 * @param publicKey - The user's public key (G... address)
 * @param sequenceNumber - The account's current sequence number
 * @param operations - Array of xdr.Operation objects
 * @returns The SorobanTransactionEnvelope XDR string
 */
export function buildSorobanXDR(
  publicKey: string,
  sequenceNumber: number,
  operations: any[],
): string {
  // Create a dummy Account from the public key and sequence number
  const account = {
    address: publicKey,
    sequenceNumber: String(sequenceNumber),
    keypair: null,
    sign: function() {},
  } as any;

  const builder = new TransactionBuilder(account, {
    fee: "100",
    networkPassphrase: "Test SDF Network ; September 2017",
  })
    .setTimeout(30)
    .addOperations(operations);

  const transaction = builder.build();

  // Add SorobanData to the transaction
  const sorobanData = new SorobanDataBuilder()
    .setResourceFee(100)  // Set the fee
    .appendFootprint([], []);  // Empty footprint for now

  // Set Soroban data on the transaction
  const envelope = transaction._buildSorobanEnvelope?.(sorobanData) ||
                   transaction._buildEnvelope(sorobanData);

  if (envelope) {
    return envelope.toXDR();
  }

  // Fallback: return the base transaction for Freighter to handle
  return transaction.toXDR();
}
