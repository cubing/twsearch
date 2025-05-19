#!/usr/bin/env bun

import { existsSync } from "node:fs";
import { $ } from "bun";

if (existsSync("./.git/")) {
  console.log((await $`git rev-parse HEAD`.text()).trim());
} else if (existsSync("./.jj/")) {
  console.log(
    (await $`jj log --limit 1 -r "@" --no-graph -T "commit_id"`.text()).trim(),
  );
} else {
  console.error("Could not get a tag.");
}
