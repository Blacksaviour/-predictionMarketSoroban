/**
 * Freighter Wallet API Wrapper
 *
 * This module wraps the Freighter browser extension API that is injected
 * onto `window.stellarPubkey` (the legacy injected API surface).
 * It provides a clean, typed interface for checking installation status,
 * requesting a public key, and signing transactions.
 *
 * Key design decisions:
 * - All methods are safe to call before the extension is installed; they
 *   return `null` or throw a descriptive error instead of crashing.
 * - The `isInstalled` check uses `typeof window !== 'undefined'` to avoid
 *   SSR crashes (this module may be imported in server contexts).
 */

// Augment the global Window type for the injected Freighter API.
// The primary interface is `window.stellarPubkey` (the older injected API).
// Some installations also expose `window.freighter` as an alias.
declare global {
  interface Window {
    stellarPubkey?: FreighterWindowAPI;
    freighter?: FreighterWindowAPI;
  }
}

/** The subset of the Freighter injected API we need. */
interface FreighterWindowAPI {
  /** The extension's application name (e.g. "Freighter"). */
  appName?: () => string;
  /** The extension's version. */
  version?: () => string;
  /** True when the extension is unlocked and ready. */
  isUnlocked?: () => Promise<{ unlocked: boolean }>;
  /** Request the user's public key (G… address). */
  getPublicKey: () => Promise<string>;
  /** Enable access to the wallet (bring extension to front, prompt user). */
  enable?: () => Promise<string>;
  /** Sign and return the XDR for a Soroban transaction (no submission). */
  signAndSendTransaction?: (
    xdr: string,
    network?: string,
    opts?: {
      publickeyId?: string;
      memo?: string;
    },
  ) => Promise<string>;
  /** Sign an arbitrary XDR and return the signed XDR. */
  signTransaction?: (
    xdr: string,
    network?: string,
    opts?: { publickeyId?: string },
  ) => Promise<string>;
  /** Get the network info the wallet is currently configured for. */
  getNetwork?: () => Promise<{
    network: string;
    networkPassphrase: string;
  }>;
}

/** Check whether Freighter (or a compatible wallet) is installed. */
export function isFreighterInstalled(): boolean {
  if (typeof window === "undefined") return false;
  return (
    !!window.stellarPubkey ||
    !!window.freighter
  );
}

/** Get the injected API object (preferring `stellarPubkey`, then `freighter`). */
function getFreighterAPI(): FreighterWindowAPI | null {
  if (typeof window === "undefined") return null;
  return window.stellarPubkey || window.freighter || null;
}

/**
 * Retrieve the connected user's public key.
 * @throws Error with a descriptive message if Freighter is not installed,
 *        not unlocked, or the user rejects the connection.
 */
export async function getPublicKey(): Promise<string> {
  const api = getFreighterAPI();
  if (!api) {
    throw new Error(
      "Freighter wallet is not installed. Please install the Freighter browser extension.",
    );
  }

  // If the wallet exposes `isUnlocked`, check it first.
  if (api.isUnlocked) {
    const { unlocked } = await api.isUnlocked();
    if (!unlocked) {
      throw new Error("Freighter is not unlocked. Please unlock your wallet.");
    }
  }

  // Trigger a user interaction if `enable` is available (some versions require it).
  if (api.enable) {
    await api.enable();
  }

  return api.getPublicKey();
}

/**
 * Sign and submit a Soroban transaction via Freighter.
 *
 * Freighter signs the `xdr` with the user's key and (in newer versions)
 * also submits it to the network. We return the transaction hash so the
 * caller can poll for the result.
 *
 * @param xdr  — unsigned Soroban transaction envelope XDR
 * @param network — the network passphrase (e.g. "Test SDF Network ; September 2017")
 * @param account — optional account ID hint
 */
export async function signAndSendTransaction(
  xdr: string,
  network: string,
  account?: string,
): Promise<string> {
  const api = getFreighterAPI();
  if (!api) {
    throw new Error(
      "Freighter wallet is not installed. Please install the Freighter browser extension.",
    );
  }

  if (!api.signAndSendTransaction && !api.signTransaction) {
    throw new Error(
      "Your Freighter version does not support signing Soroban transactions.",
    );
  }

  // Prefer signAndSendTransaction (handles both signing and submission).
  if (api.signAndSendTransaction) {
    const opts = account ? { publickeyId: account } : undefined;
    return api.signAndSendTransaction(xdr, network, opts);
  }

  // Fallback: sign only and return the signed XDR (caller submits separately).
  if (api.signTransaction) {
    const opts = account ? { publickeyId: account } : undefined;
    return api.signTransaction(xdr, network, opts);
  }

  throw new Error("Unable to sign transaction: no signing method available.");
}

/**
 * Get the network passphrase the Freighter extension is currently configured for.
 * Falls back to the Stellar testnet passphrase if the API is unavailable.
 */
export async function getNetwork(): Promise<string> {
  const api = getFreighterAPI();
  if (!api || !api.getNetwork) {
    return "Test SDF Network ; September 2017";
  }
  try {
    const { networkPassphrase } = await api.getNetwork();
    return networkPassphrase;
  } catch {
    return "Test SDF Network ; September 2017";
  }
}
