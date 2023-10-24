#!/usr/bin/env bun

import { mkdir } from "node:fs/promises";
import { build } from "esbuild";

const distDir = new URL("../dist/wasm/", import.meta.url).pathname;

await mkdir(distDir, { recursive: true });

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
});
