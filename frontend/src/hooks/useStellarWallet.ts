/**
 * useStellarWallet — Focused hook for Freighter wallet operations.
 *
 * This hook provides a simple interface for:
 * - Checking if Freighter is installed
 * - Connecting to the wallet
 * - Disconnecting from the wallet
 * - Getting the current public key
 *
 * It wraps the WalletContext but provides a slimmer API for components
 * that only need basic wallet interaction.
 */

import { useState, useCallback } from "react";
import { useWallet } from "@/context/WalletContext";

/**
 * Hook for Freighter wallet interactions.
 *
 * @example
 * ```tsx
 * const { publicKey, isConnected, isConnecting, connect, disconnect } = useStellarWallet();
 * ```
 */
export function useStellarWallet() {
  const ctx = useWallet();

  const connect = useCallback(async () => {
    await ctx.connect();
  }, [ctx]);

  const disconnect = useCallback(() => {
    ctx.disconnect();
  }, [ctx]);

  return {
    // State
    publicKey: ctx.publicKey,
    isConnected: ctx.isConnected,
    isConnecting: ctx.isConnecting,
    hasFreighter: ctx.hasFreighter,
    error: ctx.error,

    // Actions
    connect,
    disconnect,

    // Utilities
    formatPublicKey: (key: string) => {
      return `${key.slice(0, 6)}...${key.slice(-4)}`;
    },
  };
}
