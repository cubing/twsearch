#!/usr/bin/env bun

// This script is a workaround to avoid triggering https://github.com/jj-vcs/jj/discussions/4709 for known dirs.

import { spawn } from "node:child_process";
import { mkdir } from "node:fs/promises";
import { join } from "node:path";
import { exit } from "node:process";
import { fileURLToPath } from "node:url";
import { argv, file, write } from "bun";

const args = argv.slice(2);

const folders = (() => {
  if (args[0] === "--pre-forked") {
    return args.slice(1);
  }
  // The fork doesn't seem to save a lot of time on my system, but it can be beneficial if the file system is acting slow.
  // Since we only need these `.gitignore` files to be in place when switching to an old commit, we're in no hurry.
  const childProcess = spawn(
    fileURLToPath(import.meta.url),
    ["--pre-forked", ...args],
    {
      detached: true,
      stdio: ["ignore", "ignore", "ignore"],
    },
  );
  childProcess.unref();
  exit(0);
})();

await Promise.all(
  folders.map(async (folder) => {
    await mkdir(folder, { recursive: true });
    await write(file(join(folder, ".gitignore")), "*\n");
  }),
);
