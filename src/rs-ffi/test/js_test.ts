#!/usr/bin/env bun 

// @ts-ignore
import { dlopen, FFIType, suffix } from "bun:ffi";

// @ts-ignore
const path = await import.meta.resolve(
  `../../../target/release/libtwsearch_ffi.${suffix}`,
);

const {
  symbols: { ffi_random_scramble_for_event },
} = dlopen(path, {
  ffi_random_scramble_for_event: {
    args: [FFIType.cstring],
    returns: FFIType.cstring,
  },
});

for (const eventID of ["222", "pyram", ...new Array(10).fill("333")]) {
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
