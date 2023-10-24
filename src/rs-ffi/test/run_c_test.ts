#!/usr/bin/env bun

// @ts-ignore
import * as assert from "assert";
import { suffix } from "bun:ffi";
import { exit } from "process";

// @ts-ignore
const CDYLIB_PATH = await import.meta.resolve(
  `../../../target/release/libtwsearch_ffi.${suffix}`,
);

const C_SOURCE_PATH = new URL("./c_test.c", import.meta.url).pathname;
const BIN_PATH = new URL("./c_test.bin", import.meta.url).pathname;

async function runChecked(command: string[]): Promise<void> {
  console.log(command.map((v) => JSON.stringify(v)).join(" "));

  const statusCode = await Bun.spawn(command, {
    stdout: "inherit",
    stdin: "inherit",
  }).exited;
  assert.equal(statusCode, 0);
}

await runChecked(["gcc", "-o", BIN_PATH, CDYLIB_PATH, C_SOURCE_PATH]);
await runChecked([BIN_PATH]);
