#!/usr/bin/env -S bun run --

import assert from "node:assert";
import { semver } from "bun";
import { RUBY_VERSION_FILE, rustVersion, versionFileText } from "./sync";

const match = versionFileText.match(/VERSION = "(.*)"\n/);
assert(match, "Ruby version file does not match the expected format.");
const [_, rubyVersion, ...__] = match;

assert.equal(
  semver.order(rubyVersion, rustVersion),
  0,
  `Gem version (${rubyVersion}) at ${RUBY_VERSION_FILE} does not match the Rust library version (${rustVersion}).`,
);

console.log("✅ Ruby gem and Rust library versions match.");
