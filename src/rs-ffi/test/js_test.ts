#!/usr/bin/env bun

import { dlopen, FFIType, suffix } from "bun:ffi";

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
      returns: FFIType.u32,
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
  "skewb",
  ...new Array(10).fill("333"),
  "333fm",
  "333bf",
  "sq1",
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

const numScrambleFindersFreed = ffi_free_memory_for_all_scramble_finders();
console.log(
  `Freed ${numScrambleFindersFreed} scramble finder${numScrambleFindersFreed === 1 ? "" : "s"}.`,
);
