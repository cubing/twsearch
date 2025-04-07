#!/usr/bin/env bun

import { $, spawn } from "bun";

export async function getCurrentVersionDescription(): Promise<string> {
  const command = spawn(["git", "describe", "--tags"], {
    stdout: "pipe",
    stderr: "ignore",
  });
  if ((await command.exited) !== 0) {
    console.error("Using version from `jj`.");
    // From https://github.com/jj-vcs/jj/discussions/2563#discussioncomment-11885001
    return $`jj log -r 'latest(tags())::@- ~ empty()' --no-graph --reversed -T 'commit_id.short(8) ++ " " ++ tags ++ "\n"' \
  | awk '{latest = $1; count++}; $2 != "" && tag == "" { tag = $2 } END {print tag "-" count "-g" latest}'`.text();
  }
  console.error("Using version description from `git`.");
  return await new Response(command.stdout).text();
}
