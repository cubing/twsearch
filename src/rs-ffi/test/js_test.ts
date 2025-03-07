#!/usr/bin/env bun

// @ts-ignore
import { FFIType, dlopen, suffix } from "bun:ffi";

// @ts-ignore
const {
  symbols: {
    ffi_random_scramble_for_event,
    ffi_free_memory_for_all_scramble_finders,
  },
} = dlopen(
  import.meta.resolve(`../../../target/release/libtwsearch_ffi.${suffix}`),
  {
    ffi_random_scramble_for_event: {
      args: [FFIType.cstring],
      returns: FFIType.cstring,
    },
    ffi_free_memory_for_all_scramble_finders: {
      args: [],
    },
  },
);

for (const eventID of [
  "222",
  "pyram",
  "minx",
  "555",
  "666",
  "777",
  ...new Array(10).fill("333"),
  "333fm",
  "333bf",
]) {
  const startTime = performance.now();
  const scramble = ffi_random_scramble_for_event(
    new TextEncoder().encode(eventID),
  );
  console.log(
    `${scramble} // ${eventID} (${Math.floor(
      performance.now() - startTime,
    )}ms)`,
  );
}

ffi_free_memory_for_all_scramble_finders();
