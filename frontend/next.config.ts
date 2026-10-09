import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  reactStrictMode: true,
  // Configure Turbopack (default bundler in Next.js 16) for browser polyfills.
  // Node core modules are aliased to their browser-compatible equivalents.
  turbopack: {
    resolveAlias: {
      buffer: "buffer/",
      stream: "stream-browserify",
      crypto: "crypto-browserify",
      path: "path-browserify",
      util: "util/",
      process: "process/browser",
      events: "events/",
    },
  },
};

export default nextConfig;

