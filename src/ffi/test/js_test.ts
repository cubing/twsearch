#!/usr/bin/env bun

import { dlopen, FFIType, suffix } from "bun:ffi";
import { default as assert } from "node:assert";
import { Alg } from "cubing/alg";
import { cube3x3x3 } from "cubing/puzzles";

const {
  symbols: {
    ffi_random_scramble_for_event,
    ffi_free_memory_for_all_scramble_finders,
    ffi_derive_scramble_for_event,
  },
} = dlopen(
  import.meta.resolve(`../../../target/release/libtwips_ffi.${suffix}`),
  {
    ffi_random_scramble_for_event: {
      args: [FFIType.cstring],
      returns: FFIType.cstring,
    },
    ffi_free_memory_for_all_scramble_finders: {
      args: [],
      returns: FFIType.u32,
    },
    ffi_derive_scramble_for_event: {
      args: [FFIType.cstring, FFIType.cstring, FFIType.cstring],
      returns: FFIType.cstring,
    },
  },
);

const derivedScramble = ffi_derive_scramble_for_event(
  new TextEncoder().encode(
    "67002dfc95e6d4288f418fbaa9150aa65b239fd5581f2d067d0293b9321a8b67",
  ),
  new TextEncoder().encode(
    "EBNLEND@MABLNHJFHGFEKFIA@DNBKABHHNANA@FD@KKADJAKNFCIJNJGIFCBLEDF/scrambles/333/r1/g1/a1/333/sub1",
  ),
  new TextEncoder().encode("333"),
).toString();
const SOLUTION = new Alg(
  "R F U D' R L2 D' L B U' B2 D L2 U R2 B2 R2 U2 F2 U' B2",
);
const scrambled = (await cube3x3x3.kpuzzle())
  .defaultPattern()
  .applyAlg(derivedScramble);
assert(
  !scrambled.experimentalIsSolved({
    ignorePuzzleOrientation: true,
    ignoreCenterOrientation: true,
  }),
);
const solved = scrambled.applyAlg(SOLUTION);
assert(
  solved.experimentalIsSolved({
    ignorePuzzleOrientation: true,
    ignoreCenterOrientation: true,
  }),
);
console.log("✅ Derived scramble is okay.");

{
  const numScrambleFindersFreed = ffi_free_memory_for_all_scramble_finders();
  assert.equal(numScrambleFindersFreed, 1);
  console.log(
    `Freed ${numScrambleFindersFreed} scramble finder${numScrambleFindersFreed === 1 ? "" : "s"}.`,
  );
}

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

{
  const numScrambleFindersFreed = ffi_free_memory_for_all_scramble_finders();
  assert.equal(numScrambleFindersFreed, 9);
  console.log(
    `Freed ${numScrambleFindersFreed} scramble finder${numScrambleFindersFreed === 1 ? "" : "s"}.`,
  );
}
