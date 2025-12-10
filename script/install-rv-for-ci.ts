#!/usr/bin/env -S bun run --

import { platform } from "node:os";
import { env } from "node:process";
import { $ } from "bun";

// biome-ignore lint/complexity/useLiteralKeys: https://github.com/biomejs/biome/discussions/7404
if (env["CI"] !== "true") {
  throw new Error("Not in CI.");
}

if (platform() === "win32") {
  console.info("Skipping `rv` installation for CI.");
} else {
  await $`curl --proto '=https' --tlsv1.2 -LsSf https://github.com/spinel-coop/rv/releases/download/v0.3.0/rv-installer.sh | sh`;
}
