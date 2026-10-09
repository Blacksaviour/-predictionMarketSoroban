/**
 * Freighter Wallet API Wrapper
 *
 * Uses the official `@stellar/freighter-api` package (v4), which is the
 * supported way to talk to the Freighter browser extension. It handles the
 * extension's asynchronous injection and API-version differences for us.
 *
 * Key methods used:
 * - requestAccess()  → prompts the user to authorize the site, returns address
 * - getAddress()     → returns the address if already authorized (no prompt)
 * - signTransaction()→ signs an XDR, returns { signedTxXdr, signerAddress }
 * - getNetwork()     → returns { network, networkPassphrase }
 *
 * NOTE: Freighter injects `window.freighter` asynchronously *after* the page
 * loads, so `detectFreighter()` polls for a short window rather than
 * checking exactly once on mount.
 */

import {
  requestAccess,
  getAddress,
  signTransaction as freighterSignTransaction,
  getNetwork as freighterGetNetwork,
} from "@stellar/freighter-api";

/** Default testnet passphrase used as a fallback. */
const TESTNET_PASSPHRASE = "Test SDF Network ; September 2017";

/** Shape of the injected Freighter bridge (used only for presence detection). */
declare global {
  interface Window {
    freighter?: unknown;
    stellarPubkey?: unknown;
  }
}

/** How long to wait (ms) for the extension to inject itself. */
const INJECTION_TIMEOUT_MS = 2000;
/** Poll interval (ms) while waiting for injection. */
const INJECTION_POLL_MS = 150;

/**
 * Check whether the Freighter extension has injected its bridge.
 *
 * Returns `true` immediately if present. Otherwise polls for up to
 * `INJECTION_TIMEOUT_MS`, because the extension injects asynchronously.
 */
export async function detectFreighter(): Promise<boolean> {
  if (typeof window === "undefined") return false;

  if (window.freighter || window.stellarPubkey) return true;

  const deadline = Date.now() + INJECTION_TIMEOUT_MS;
  while (Date.now() < deadline) {
    await new Promise((resolve) => setTimeout(resolve, INJECTION_POLL_MS));
    if (window.freighter || window.stellarPubkey) return true;
  }
  return false;
}

/**
 * Synchronous best-effort check for Freighter presence.
 *
 * Prefer {@link detectFreighter} (async, waits for injection) when possible.
 * This returns the current presence without waiting.
 */
export function isFreighterInstalled(): boolean {
  if (typeof window === "undefined") return false;
  return Boolean(window.freighter || window.stellarPubkey);
}

/** Extract a human-readable message from a Freighter API error object. */
function freighterErrorMessage(error: any, fallback: string): string {
  if (error && typeof error.message === "string" && error.message.length > 0) {
    return error.message;
  }
  return fallback;
}

/**
 * Connect to Freighter and return the user's public key (G… address).
 *
 * Uses `requestAccess()`, which prompts the user to authorize the site if
 * they haven't already, and falls back to `getAddress()` for an existing
 * authorization.
 *
 * @throws Error with a descriptive message if Freighter is not installed
 *        or the user rejects the connection.
 */
export async function getPublicKey(): Promise<string> {
  if (typeof window === "undefined") {
    throw new Error("Wallet connection is only available in the browser.");
  }

  const installed = await detectFreighter();
  if (!installed) {
    throw new Error(
      "Freighter wallet is not installed. Please install the Freighter browser extension.",
    );
  }

  // requestAccess() prompts the user to authorize the site.
  const access = await requestAccess();
  if (access.error) {
    throw new Error(
      freighterErrorMessage(
        access.error,
        "Could not connect to Freighter. Please approve the connection request.",
      ),
    );
  }

  // requestAccess may return an empty address if the site is already allowed
  // but not explicitly requested; fall back to getAddress().
  if (access.address) {
    return access.address;
  }

  const addr = await getAddress();
  if (addr.error) {
    throw new Error(
      freighterErrorMessage(addr.error, "Unable to retrieve your Freighter address."),
    );
  }
  return addr.address;
}

/**
 * Sign a Soroban transaction XDR with Freighter.
 *
 * NOTE: Freighter only *signs* — it does not submit. This returns the signed
 * XDR; the caller is responsible for submitting it to the network.
 *
 * @param xdr — unsigned Soroban transaction envelope XDR
 * @param networkPassphrase — the network passphrase to sign for
 * @param account — optional address hint (the signer)
 * @returns the signed transaction XDR
 */
export async function signTransactionXdr(
  xdr: string,
  networkPassphrase: string,
  account?: string,
): Promise<string> {
  const installed = await detectFreighter();
  if (!installed) {
    throw new Error(
      "Freighter wallet is not installed. Please install the Freighter browser extension.",
    );
  }

  const opts = account
    ? { networkPassphrase, address: account }
    : { networkPassphrase };
  const res = await freighterSignTransaction(xdr, opts);
  if (res.error) {
    throw new Error(
      freighterErrorMessage(res.error, "Freighter failed to sign the transaction."),
    );
  }
  return res.signedTxXdr;
}

/**
 * Sign and submit a Soroban transaction via Freighter.
 *
 * Kept for backward compatibility with callers that expect a single call.
 * This signs the XDR and returns the signed XDR (the caller submits it and
 * polls for the hash via the Soroban RPC client).
 *
 * @deprecated Prefer {@link signTransactionXdr} and submit via the RPC client.
 */
export async function signAndSendTransaction(
  xdr: string,
  network: string,
  account?: string,
): Promise<string> {
  return signTransactionXdr(xdr, network, account);
}

/**
 * Get the network passphrase the Freighter extension is currently configured
 * for. Falls back to the Stellar testnet passphrase if unavailable.
 */
export async function getNetwork(): Promise<string> {
  if (typeof window === "undefined") return TESTNET_PASSPHRASE;

  if (!(await detectFreighter())) {
    return TESTNET_PASSPHRASE;
  }

  try {
    const res = await freighterGetNetwork();
    if (res.error || !res.networkPassphrase) {
      return TESTNET_PASSPHRASE;
    }
    return res.networkPassphrase;
  } catch {
    return TESTNET_PASSPHRASE;
  }
}
