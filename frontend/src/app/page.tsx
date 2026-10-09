"use client";

import { useEffect, useState } from "react";
import { useStellarWallet } from "@/hooks/useStellarWallet";
import { useContract } from "@/hooks/useContract";
import { CONTRACT_IDS } from "@/lib/config";
import { WalletConnect } from "@/components/WalletConnect";

/**
 * USDR Token Information Card
 */
function USDRTokenCard() {
  const { publicKey, isConnected } = useStellarWallet();
  const { read } = useContract(CONTRACT_IDS.usdrToken);
  const [balance, setBalance] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchBalance = async () => {
    if (!isConnected) return;
    setLoading(true);
    try {
      const bal = await read("balance_of", [publicKey!]);
      setBalance(bal);
    } catch (err) {
      console.error("Failed to fetch balance:", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (isConnected) {
      fetchBalance();
    } else {
      setBalance(null);
    }
  }, [isConnected, publicKey, read]);

  return (
    <div className="rounded-xl bg-card border border-border p-5">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold text-card-foreground">USDR Token</h2>
        <span className="text-xs text-muted-foreground">Rain Dollar (USDR)</span>
      </div>

      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <span className="text-sm text-muted-foreground">Token Name</span>
          <span className="text-sm text-card-foreground">Rain Dollar</span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-sm text-muted-foreground">Symbol</span>
          <span className="text-sm text-card-foreground">USDR</span>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-sm text-muted-foreground">Decimals</span>
          <span className="text-sm text-card-foreground">18</span>
        </div>
        <div className="flex items-center justify-between border-t border-border pt-3">
          <span className="text-sm text-muted-foreground">Your Balance</span>
          <div className="flex items-center gap-2">
            {loading ? (
              <span className="h-4 w-4 animate-spin rounded-full border-2 border-stellar-blue border-t-transparent" />
            ) : (
              <span className="text-sm text-card-foreground font-mono">
                {balance ? Number(balance) / 1e18 : "—"} USDR
              </span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

/**
 * Vault Engine Overview Card
 */
function VaultEngineCard() {
  const { read, result } = useContract(CONTRACT_IDS.vaultEngine);
  const debt = result.data as string | null;
  const vice = result.data?.vice as string | null;
  const line = result.data?.line as string | null;

  return (
    <div className="rounded-xl bg-card border border-border p-5">
      <h2 className="text-lg font-semibold text-card-foreground mb-4">Vault Engine</h2>

      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div className="rounded-lg bg-muted/30 p-3">
          <div className="text-xs text-muted-foreground">Total Debt</div>
          <div className="text-lg font-semibold text-card-foreground">
            {debt ? Number(debt) / 1e45 : "—"} USDR
          </div>
        </div>
        <div className="rounded-lg bg-muted/30 p-3">
          <div className="text-xs text-muted-foreground">Bad Debt (Vice)</div>
          <div className="text-lg font-semibold text-card-foreground">
            {vice ? Number(vice) / 1e45 : "—"} USDR
          </div>
        </div>
        <div className="rounded-lg bg-muted/30 p-3">
          <div className="text-xs text-muted-foreground">Global Line Limit</div>
          <div className="text-lg font-semibold text-card-foreground">
            {line ? Number(line) / 1e45 : "—"} USDR
          </div>
        </div>
      </div>
    </div>
  );
}

/**
 * Price Converter Card
 */
function PriceConverterCard() {
  const { read, result } = useContract(CONTRACT_IDS.priceConverter);
  const par = result.data?.par as string | null;
  const live = result.data?.live as number | null;

  return (
    <div className="rounded-xl bg-card border border-border p-5">
      <h2 className="text-lg font-semibold text-card-foreground mb-4">Price Converter</h2>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <div className="rounded-lg bg-muted/30 p-3">
          <div className="text-xs text-muted-foreground">Reference Par Value</div>
          <div className="text-lg font-semibold text-card-foreground">
            {par ? Number(par) / 1e27 : "—"} USDR
          </div>
        </div>
        <div className="rounded-lg bg-muted/30 p-3">
          <div className="text-xs text-muted-foreground">System Liveness</div>
          <div className="text-lg font-semibold text-card-foreground">
            {live !== null ? (live === 1 ? "Live" : "Paused") : "—"}
          </div>
        </div>
      </div>
    </div>
  );
}

/**
 * Main Dashboard Page
 */
export default function DashboardPage() {
  const { isConnected, isConnecting, hasFreighter } = useStellarWallet();

  return (
    <div className="min-h-screen bg-background text-foreground">
      <header className="border-b border-border bg-card/50 backdrop-blur-sm">
        <div className="mx-auto max-w-5xl px-4 py-4 sm:px-6">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-stellar-blue/10">
                <span className="text-sm font-bold text-stellar-blue">R</span>
              </div>
              <div>
                <h1 className="text-lg font-bold text-card-foreground">Rain USDR Dashboard</h1>
                <p className="text-xs text-muted-foreground">
                  Soroban frontend for the Rain USDR stablecoin protocol
                </p>
              </div>
            </div>

            <div className="flex items-center gap-3">
              <div className="flex items-center gap-2 text-sm">
                <span className={`h-2 w-2 rounded-full ${isConnected ? "bg-success" : "bg-muted-foreground"}`} />
                <span className={isConnected ? "text-success" : "text-muted-foreground"}>
                  {isConnected ? "Connected" : "Not Connected"}
                </span>
                {hasFreighter && <span className="text-xs text-muted-foreground">(Freighter)</span>}
              </div>
              <WalletConnect />
            </div>
          </div>
        </div>
      </header>

      <main className="mx-auto max-w-5xl px-4 py-6 space-y-6">
        {!isConnected && (
          <div className="rounded-xl border border-border bg-muted/30 p-4">
            <p className="text-sm text-muted-foreground">
              Connect your Freighter wallet to view your USDR balance and interact with the contracts.
            </p>
          </div>
        )}

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <USDRTokenCard />
          <VaultEngineCard />
        </div>

        <PriceConverterCard />
      </main>

      <footer className="border-t border-border py-4">
        <div className="mx-auto max-w-5xl px-4 text-center text-xs text-muted-foreground">
          Rain USDR — Soroban port of the USDR stablecoin protocol.
          <br />
          Not audited. For development purposes only.
        </div>
      </footer>
    </div>
  );
}
