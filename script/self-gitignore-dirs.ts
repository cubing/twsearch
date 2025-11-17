#!/usr/bin/env bun

// This script is a workaround to avoid triggering https://github.com/jj-vcs/jj/discussions/4709 for known dirs.

import { spawn } from "node:child_process";
import { exit } from "node:process";
import { fileURLToPath } from "node:url";
import { argv } from "bun";
import { Path } from "path-class";

const args = argv.slice(2);

const folders = (() => {
  if (args[0] === "--pre-forked") {
    return args.slice(1).map((path) => new Path(path));
  }
  // The fork doesn't seem to save a lot of time on my system, but it can be beneficial if the file system is acting slow.
  // Since we only need these `.gitignore` files to be in place when switching to an old commit, we're in no hurry.
  const childProcess = spawn(
    "bun",
    ["run", fileURLToPath(import.meta.url), "--pre-forked", ...args],
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
    await folder.mkdir();
    await folder.join(".gitignore").write("*\n");
  }),
);
