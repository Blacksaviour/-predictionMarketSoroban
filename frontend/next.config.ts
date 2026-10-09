import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  reactStrictMode: true,
  // Note: We intentionally do NOT alias Node core modules here.
  //
  // @stellar/stellar-sdk v17 ships browser-compatible builds that Turbopack
  // resolves automatically via the package "browser"/"exports" fields.
  // Global `resolveAlias` shims (util, process/browser, etc.) previously
  // broke server-side prerendering because the browser `util` shim shadows
  // Node's real `TextDecoder`. The minimal `Buffer`/`process` globals the
  // SDK needs are installed client-side in `src/lib/polyfills.ts`.
};

export default nextConfig;

