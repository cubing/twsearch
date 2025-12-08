#!/usr/bin/env -S bun run --

import assert from "node:assert";
import { stdin } from "node:process";
import { Readable } from "node:stream";
import type { PostVersionInfo } from "@lgarron-bin/repo/types/postVersion";
import { semver } from "bun";
import { Path } from "path-class";
import { PrintableShellCommand } from "printable-shell-command";

export const RUBY_VERSION_FILE = new Path(
  "./src/ruby-gem/lib/twips/version.rb",
);
export const RUBY_VERSION_ASSIGNMENT_REGEX = /VERSION = "(.*)"\n/;

export const rustVersion = await new PrintableShellCommand("bun", [
  ["x", "@lgarron-bin/repo"],
  "version",
  ["--ecosystem", "rust"],
  "get",
]).text();
console.log(rustVersion);

// Do a basic sense check on the Rust version that exercises parsing it.
assert.equal(semver.order(rustVersion, rustVersion), 0);
assert(rustVersion.startsWith("v"));

export let versionFileText = await RUBY_VERSION_FILE.readText();
if (import.meta.main) {
  const stdinJSON: PostVersionInfo = await new Response(
    Readable.from(stdin) as any,
  ).json();
  if (stdinJSON) {
    assert(versionFileText.match(RUBY_VERSION_ASSIGNMENT_REGEX));
    assert.equal(semver.order(rustVersion, stdinJSON.version), 0);
  }

  assert(versionFileText.match(RUBY_VERSION_ASSIGNMENT_REGEX));
  const bareVersion = rustVersion.slice(1);
  versionFileText = versionFileText.replace(
    RUBY_VERSION_ASSIGNMENT_REGEX,
    `VERSION = ${JSON.stringify(bareVersion)}\n`,
  );
  await RUBY_VERSION_FILE.write(versionFileText);
}
