/**
 * WalletConnect — Freighter wallet connection button and status display.
 *
 * Shows:
 * - A "Connect Wallet" button when not connected
 * - The user's address (showing the last 4 characters) when connected
 * - Freighter installation status
 * - Connecting state with loading indicator
 * - Error message on connection failure
 *
 * The button also allows "hiding" the wallet (disconnect).
 */

"use client";

import { useEffect, useState } from "react";
import { useStellarWallet } from "@/hooks/useStellarWallet";

export function WalletConnect() {
  const { publicKey, isConnected, isConnecting, hasFreighter, error, connect, disconnect } =
    useStellarWallet();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const handleConnect = async () => {
    try {
      await connect();
    } catch (err) {
      console.error("Failed to connect:", err);
    }
  };

  const handleDisconnect = () => {
    disconnect();
  };

  if (!mounted) return null;

  return (
    <div className="flex items-center gap-3">
      {/* Freighter installation indicator */}
      <div className="flex items-center gap-2 text-xs text-muted-foreground">
        <span className={`h-2 w-2 rounded-full ${hasFreighter ? "bg-success" : "bg-muted-foreground"}`} />
        {hasFreighter ? (
          <span className="font-medium text-muted-foreground">Freighter installed</span>
        ) : (
          <span className="text-muted-foreground">Browser extension detected</span>
        )}
      </div>

      {!isConnected ? (
        <button
          onClick={handleConnect}
          disabled={isConnecting}
          className="flex items-center gap-2 rounded-lg bg-stellar-blue hover:bg-stellar-blue/90 px-4 py-2 text-sm font-medium text-stellar-dark transition-colors disabled:opacity-50"
        >
          {isConnecting ? (
            <>
              <span className="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
              Connecting...
            </>
          ) : (
            <>
              <svg
                className="h-4 w-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
                />
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
              Connect Wallet
            </>
          )}
        </button>
      ) : (
        <div className="flex items-center gap-2 rounded-lg bg-card border border-border px-4 py-2">
          <span className="h-2 w-2 rounded-full bg-success" />
          <span className="text-sm text-card-foreground">
            {publicKey ? `Connected — ${publicKey}` : "Connected"}
          </span>
          <button
            onClick={handleDisconnect}
            className="ml-2 text-muted-foreground hover:text-foreground transition-colors"
            title="Disconnect wallet"
          >
            <svg
              className="h-4 w-4"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
              />
            </svg>
          </button>
        </div>
      )}

      {error && (
        <div className="text-sm text-error flex items-center gap-1">
          <svg
            className="h-4 w-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4.5c-.77-.833-2.694-.833-3.464 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z"
            />
          </svg>
          {error}
        </div>
      )}
    </div>
  );
}

/**
 * Simple status message display component.
 */
export function StatusMessage({ message, type }: { message: string; type: "success" | "error" | "info" }) {
  const styles = {
    success: "bg-success/10 text-success border-success/20",
    error: "bg-error/10 text-error border-error/20",
    info: "bg-muted text-muted-foreground border-border",
  };

  return (
    <div className={`rounded-lg border p-3 ${styles[type]}`}>
      <p className="text-sm">{message}</p>
    </div>
  );
}
