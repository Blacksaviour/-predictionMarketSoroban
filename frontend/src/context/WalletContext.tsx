"use client";

/**
 * Wallet Context — Provides Freighter wallet state to the entire app.
 *
 * This context tracks:
 * - `publicKey` — The connected wallet's public key (null if not connected)
 * - `isConnected` — Whether a wallet is currently connected
 * - `isConnecting` — Whether a connection attempt is in progress
 * - `hasFreighter` — Whether the Freighter extension is installed
 * - `error` — Any error from the connection process
 *
 * The context also exposes a `connect()` function that checks for
 * Freighter installation and requests the user's public key.
 *
 * All wallet state is purely client-side — the context provider throws
 * an error if used during SSR.
 */

import { createContext, useContext, useState, useEffect, ReactNode } from "react";
import "../lib/polyfills";
import {
  detectFreighter,
  getPublicKey,
  getNetwork,
} from "@/lib/freighter";
import { CONSTANTS } from "@/lib/config";

export interface WalletProviderState {
  publicKey: string | null;
  isConnected: boolean;
  isConnecting: boolean;
  hasFreighter: boolean;
  error: string | null;
  connect: () => Promise<void>;
  disconnect: () => void;
}

const WalletContext = createContext<WalletProviderState | null>(null);

/**
 * Check if Freighter is available in the browser environment.
 *
 * This function is safe to call in both client and server contexts
 * (it guards with `typeof window !== "undefined"`).
 */
export function useFreighterCheck() {
  const [hasFreighter, setHasFreighter] = useState<boolean>(false);
  const [isChecking, setIsChecking] = useState<boolean>(false);

  useEffect(() => {
    let mounted = true;

    async function check() {
      if (typeof window === "undefined") {
        setHasFreighter(false);
        return;
      }
      setIsChecking(true);
      try {
        const installed = await detectFreighter();
        setHasFreighter(installed);
      } catch {
        setHasFreighter(false);
      } finally {
        setIsChecking(false);
      }
    }

    check();

    return () => {
      mounted = false;
    };
  }, []);

  return {
    hasFreighter,
    isChecking,
  };
}

/**
 * The WalletProvider component.
 *
 * Call this via the `<WalletProvider>` wrapper in your layout.
 *
 * @example
 * ```tsx
 * // layout.tsx
 * export default function RootLayout({ children }) {
 *   return <WalletProvider>{children}</WalletProvider>
 * }
 * ```
 */
export function WalletProvider({ children }: { children: ReactNode }) {
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [isConnected, setIsConnected] = useState<boolean>(false);
  const [isConnecting, setIsConnecting] = useState<boolean>(false);
  const [hasFreighter, setHasFreighter] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  // Initial check for Freighter installation on mount
  useEffect(() => {
    let mounted = true;

    async function initialCheck() {
      if (typeof window === "undefined") {
        setHasFreighter(false);
        return;
      }

      try {
        // Poll for the async-injected extension bridge.
        const installed = await detectFreighter();
        if (mounted) setHasFreighter(installed);

        if (installed) {
          // Try to get the public key if already authorized (no prompt).
          try {
            const key = await getPublicKey();
            if (mounted) {
              setPublicKey(key);
              setIsConnected(true);
            }
          } catch {
            // User not connected or wallet not unlocked — ignore.
          }
        }
      } catch {
        setHasFreighter(false);
      }
    }

    initialCheck();

    return () => {
      mounted = false;
    };
  }, []);

  /**
   * Connect to the Freighter wallet.
   *
   * - Checks if Freighter is installed
   * - Requests the user's public key (may prompt a popup)
   * - Updates context state
   */
  async function connect() {
    if (typeof window === "undefined") {
      throw new Error("Wallet connection not supported in SSR");
    }

    setError(null);
    setIsConnecting(true);

    try {
      // Verify Freighter is installed (poll for async injection).
      if (!(await detectFreighter())) {
        throw new Error(
          "Freighter wallet is not installed. Please install the Freighter browser extension.",
        );
      }
      setHasFreighter(true);

      // Get the public key (prompts user if needed)
      const key = await getPublicKey();

      setPublicKey(key);
      setIsConnected(true);
    } catch (err) {
      const message = err instanceof Error ? err.message : "Unknown error";
      setError(message);
      setPublicKey(null);
      setIsConnected(false);
      throw err;
    } finally {
      setIsConnecting(false);
    }
  }

  /**
   * Disconnect from the wallet (clear any stored state).
   */
  function disconnect() {
    setPublicKey(null);
    setIsConnected(false);
    setError(null);
  }

  const value: WalletProviderState = {
    publicKey,
    isConnected,
    isConnecting,
    hasFreighter,
    error,
    connect,
    disconnect,
  };

  return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
}

/**
 * Hook to access the wallet context.
 *
 * @throws {Error} If used outside a `<WalletProvider>`
 */
export function useWallet(): WalletProviderState {
  const context = useContext(WalletContext);
  if (!context) {
    throw new Error("useWallet must be used within a <WalletProvider>");
  }
  return context;
}

/**
 * Get the Freighter network string.
 * @returns The network passphrase (testnet in dev)
 */
export function getFreighterNetwork(): string {
  return "Test SDF Network ; September 2017";
}

/**
 * Helper to format a public key for display (first/last 4 chars).
 */
export function formatPublicKey(key: string): string {
  return `${key.slice(0, 6)}...${key.slice(-4)}`;
}
