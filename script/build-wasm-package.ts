#!/usr/bin/env bun

import { mkdir } from "node:fs/promises";
import { env } from "node:process";
import { build } from "esbuild";

const distDir = new URL("../dist/wasm/", import.meta.url).pathname;

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
    new URL("../src/wasm-package/index.ts", import.meta.url).pathname,
  ],
  format: "esm",
  bundle: true,
  splitting: true,
  loader: { ".wasm": "binary" },
  outdir: distDir,
  external: ["cubing"],
  banner: {
    js: `// Generated from \`twsearch\` ${version}`,
  },
});
