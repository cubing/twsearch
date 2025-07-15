import { existsSync } from "node:fs";
import { $ } from "bun";

export async function getCurrentVersionDescription(): Promise<string> {
  if (existsSync("./.git/")) {
    console.error("ℹ️ Using version description from `git`.");
    return (await $`git describe --tags`.text()).trim();
  }

  if (existsSync("./.jj/")) {
    console.error("ℹ️ Using version from `jj`.");
    // From https://github.com/jj-vcs/jj/discussions/2563#discussioncomment-11885001
    return (
      await $`jj log -r 'latest(tags())::@- ~ empty()' --no-graph --reversed -T 'commit_id.short(8) ++ " " ++ tags ++ "\n"' \
  | awk '{latest = $1; count++}; $2 != "" && tag == "" { tag = $2 } END {print tag "-" count "-g" latest}'`.text()
    ).trim();
  }

  throw new Error("Could not get a current version description.");
}
