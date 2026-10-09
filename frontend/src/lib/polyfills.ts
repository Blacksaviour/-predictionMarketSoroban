// Client-side polyfills required by @stellar/stellar-sdk in the browser.
// Next.js 16 uses Turbopack by default, which does not support webpack's
// ProvidePlugin, so we set the Node globals manually on the client.
import { Buffer } from "buffer/";

if (typeof globalThis.Buffer === "undefined") {
  globalThis.Buffer = Buffer;
}

if (typeof globalThis.process === "undefined") {
  // A minimal `process` shim is enough for the stellar-sdk browser build.
  globalThis.process = { env: {}, browser: true } as unknown as NodeJS.Process;
}

export {};
