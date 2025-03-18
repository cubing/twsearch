#!/usr/bin/env bun

import { exists } from "node:fs/promises";
import { $ } from "bun";

if (await exists("./.git")) {
  console.log((await $`git rev-parse HEAD`.text()).trim());
} else if (await exists("./.jj")) {
  console.log(
    (await $`jj log --limit 1 -r "@" --no-graph -T "commit_id"`.text()).trim(),
  );
} else {
  console.error("Could not get a tag.");
}
