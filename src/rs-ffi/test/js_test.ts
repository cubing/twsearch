#!/usr/bin/env bun 

// @ts-ignore
import { dlopen, FFIType, suffix } from "bun:ffi";

// @ts-ignore
const path = await import.meta.resolve(`../../../target/release/libtwsearch_ffi.${suffix}`);

const {
  symbols: {
    ffi_random_scramble_for_event,
  },
} = dlopen(
  path,
  {
    ffi_random_scramble_for_event: {
      args: [ FFIType.cstring],
      returns: FFIType.cstring,
    },
  },
);

console.log(`// Scrambles`);
for (let i = 0; i < 5; i++) {
  console.log(ffi_random_scramble_for_event(new TextEncoder().encode("222")));
}
