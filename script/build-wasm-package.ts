#!/usr/bin/env bun

import assert from "node:assert";
import { mkdir } from "node:fs/promises";
import { env } from "node:process";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";

const distDir = fileURLToPath(new URL("../dist/wasm/", import.meta.url));

const BITS_PER_BYTE = 8;

const KiB = 2 ** 10;
// https://github.com/GoogleChrome/lighthouse/blob/b129c136bff66c6c74c17d92a61c8c245abca435/core/config/constants.js#L34
const mobile3GConnectionBytesPerSecond = 700 * (KiB / BITS_PER_BYTE); // ≈90 KiB
function secondsToDownloadUsing3G(numBytes: number): number {
  return numBytes / mobile3GConnectionBytesPerSecond;
}

const wasmSize = Bun.file(
  new URL("../.temp/rust-wasm/twsearch_wasm_bg.wasm", import.meta.url),
).size;
console.log(
  `WASM size: ${Math.round(wasmSize / KiB)} KiB (${
    Math.round(secondsToDownloadUsing3G(wasmSize) * 10) / 10
  } seconds over 3G).`,
);
assert(wasmSize > 32 * KiB); // Make sure the file exists and has some contents.

/**
 * It's okay to increase this a bit, but anything approaching half a MiB is
 * pretty slow for intermittent mobile connections. Note that the entire
 * download has to happen before:
 *
 * - The base64 source is turned into WASM.
 *   - We can't use streaming instantiation (
 *     https://developer.mozilla.org/en-US/docs/WebAssembly/JavaScript_interface/instantiateStreaming
 *     ) due to `esbuild` limitations for portable code:
 *     https://github.com/evanw/esbuild/issues/795
 * - The WASM is instantiated.
 * - The search code initializes.
 * - The actual search is performed.
 *
 * And that's not even counting potential steps beforehand:
 *
 * - HTTPS handshakes
 * - HTML download
 * - main script download and parsing
 * - module tree download and parsing
 * - JS code invocation for a search/scramble
 * - Worker instantiation (with half a dozen fallbacks to try first)
 * - Worker initialization and dynamic import of the base64 source.
 *
 * Since we are consolidating all scramble code in `twsearch`, this is a
 * bottleneck for *all* scrambles, even the trivial ones (e.g. Megaminx and
 * Clock).
 *
 * For more:
 *
 * - See https://github.com/cubing/cubing.js/issues/291 for an issue about
 *   performing more of these steps in parallel.
 * - See https://github.com/cubing/twsearch/issues/37 for an issue about
 *   decreasing the WASM build size directly.
 */
assert(secondsToDownloadUsing3G(wasmSize) < 4.5);

await mkdir(distDir, { recursive: true });

const version = (async () => {
  if (env.CI) {
    // Bun seems to segfault, so we need to avoid the `spawn` call completely.
    console.warn(
      "WARNING: We seem to be in CI, embedding unknown version number to avoid a segfault.",
    );
    return "vUNKNOWN";
  }
  return await new Response(
    Bun.spawn(["git", "describe", "--tags"], { stdout: "pipe" }).stdout,
  ).text();
})();

build({
  entryPoints: [
    fileURLToPath(new URL("../src/wasm-package/index.ts", import.meta.url)),
  ],
  format: "esm",
  bundle: true,
  splitting: true,
  loader: { ".wasm": "binary" },
  outdir: distDir,
  external: ["cubing", "getbuiltinmodule-ponyfill"],
  banner: {
    js: `// Generated from \`twsearch\` ${version}`,
  },
});
