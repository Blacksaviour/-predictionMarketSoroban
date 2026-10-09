/**
 * Network configuration for the Rain USDR frontend.
 *
 * The RPC endpoint and network passphrase target the Stellar testnet,
 * where the USDR contracts are deployed during development.
 *
 * Override these at build/runtime via a `.env.local` file:
 *
 *   NEXT_PUBLIC_SOROBAN_RPC_URL=https://soroban-testnet.stellar.org
 *   NEXT_PUBLIC_NETWORK_PASSPHRASE="Standalone Network ; February 2017"
 *   NEXT_PUBLIC_VAULT_ENGINE_ID=…
 *   NEXT_PUBLIC_USDR_TOKEN_ID=…
 *   NEXT_PUBLIC_PRICE_CONVERTER_ID=…
 *   NEXT_PUBLIC_RESERVE_ACCOUNTING_ID=…
 *   NEXT_PUBLIC_PRICE_CURVE_ID=…
 */

export const NETWORK = {
  rpcUrl:
    process.env.NEXT_PUBLIC_SOROBAN_RPC_URL ||
    "https://soroban-testnet.stellar.org",
  networkPassphrase:
    process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE ||
    "Test SDF Network ; September 2017",
} as const;

/**
 * Deployed contract IDs on the Stellar testnet.
 *
 * These are placeholders — replace them with the actual deployed IDs
 * from `stellar contract deploy` output after deploying the contracts.
 * The README in the Rust project contains deployment instructions.
 */
export const CONTRACT_IDS = {
  vaultEngine:
    process.env.NEXT_PUBLIC_VAULT_ENGINE_ID ||
    "CAYCLN5JVRC5RFPDFSFPJZAWJG3KJ5J5J5J5J5J5J5J5J5J5J5J5J5J5",
  usdrToken:
    process.env.NEXT_PUBLIC_USDR_TOKEN_ID ||
    "CAYCLN5JVRC5RFPDFSFPJZAWJG3KJ5J5J5J5J5J5J5J5J5J5J5J5J5",
  priceConverter:
    process.env.NEXT_PUBLIC_PRICE_CONVERTER_ID ||
    "CAYCLN5JVRC5RFPDFSFPJZAWJG3KJ5J5J5J5J5J5J5J5J5J5J5J5J5",
  reserveAccounting:
    process.env.NEXT_PUBLIC_RESERVE_ACCOUNTING_ID ||
    "CAYCLN5JVRC5RFPDFSFPJZAWJG3KJ5J5J5J5J5J5J5J5J5J5J5J5",
  priceCurve:
    process.env.NEXT_PUBLIC_PRICE_CURVE_ID ||
    "CAYCLN5JVRC5RFPDFSFPJZAWJG3KJ5J5J5J5J5J5J5J5J5J5J5J5",
} as const;

/** Common token/asset identifiers used in the USDR system. */
export const CONSTANTS = {
  WAD: "1000000000000000000", // 10^18
  RAY: "1000000000000000000000000000", // 10^27
  RAD: "100000000000000000000000000000000000000000000000", // 10^45

  // Common ilk names
  RAIN_ILK: "RAIN",
  USDT_ILK: "USDT",
  USDC_ILK: "USDC",
  USDR_ILK: "USDR",
} as const;
