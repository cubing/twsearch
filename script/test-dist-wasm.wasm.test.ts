import { expect, test } from "bun:test";
import assert from "node:assert";
import { cube2x2x2 } from "cubing/puzzles";
import {
  wasmDeriveScrambleForEvent,
  wasmFreeMemoryForAllScrambleFinders,
  wasmRandomScrambleForEvent,
  wasmTwips,
} from "../dist/wasm/index";

test("wasmRandomScrambleForEvent(…)", async () => {
  for (const eventID of [
    "222",
    "pyram",
    "minx",
    "555",
    "666",
    "777",
    "sq1",
    ...new Array(10).fill("333"),
    "333oh",
    "333fm",
    "skewb",
  ]) {
    const startTime = performance.now();
    const scramble = await wasmRandomScrambleForEvent(eventID);
    console.log(
      `${scramble} // ${eventID} (${Math.floor(
        performance.now() - startTime,
      )}ms)`,
    );
  }
});

test("wasmTwips(…)", async () => {
  console.log("----------------");
  console.log("Performing a basic 2x2x2 search:");
  const kpuzzle = await cube2x2x2.kpuzzle();
  const scramble = "L' U' L U F U F'";
  console.log("Scramble:", scramble);
  const pattern = kpuzzle.defaultPattern().applyAlg(scramble);
  const solution = await wasmTwips(kpuzzle.definition, pattern, {
    generatorMoves: ["R", "U", "F"],
  });
  console.log("Solution:", solution.toString());
  assert(pattern.applyAlg(solution).isIdentical(kpuzzle.defaultPattern()));
});

test("wasmDeriveScrambleForEvent(…)", async () => {
  console.log("----------------");
  console.log("Deriving scrambles.");
  const scramble1 = await wasmDeriveScrambleForEvent(
    "67002dfc95e6d4288f418fbaa9150aa65b239fd5581f2d067d0293b9321a8b67",
    [
      "EBNLEND@MABLNHJFHGFEKFIA@DNBKABHHNANA@FD@KKADJAKNFCIJNJGIFCBLEDF",
      "scrambles",
      "333",
      "r1",
      "g1",
      "a1",
      "333",
      "sub1",
    ],
    "333",
  );
  const scramble2 = await wasmDeriveScrambleForEvent(
    "67002dfc95e6d4288f418fbaa9150aa65b239fd5581f2d067d0293b9321a8b67",
    [
      "EBNLEND@MABLNHJFHGFEKFIA@DNBKABHHNANA@FD@KKADJAKNFCIJNJGIFCBLEDF",
      "scrambles",
      "333",
      "r1",
      "g1",
      "a1",
      "333",
      "sub1",
    ],
    "333",
  );
  const scramble3 = await wasmDeriveScrambleForEvent(
    "6700abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdef",
    [
      "EBNLEND@MABLNHJFHGFEKFIA@DNBKABHHNANA@FD@KKADJAKNFCIJNJGIFCBLEDF",
      "scrambles",
      "333",
      "r1",
      "g1",
      "a1",
      "333",
      "sub1",
    ],
    "333",
  );

  expect(scramble1.isIdentical(scramble2)).toBe(true);
  expect(!scramble1.isIdentical(scramble3)).toBe(true);

  // TODO: use `bun` to write an expectation for this.
  expect(() =>
    wasmDeriveScrambleForEvent(
      "6700abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdef",
      [
        "EBNLEND@MABLNHJFHGFEKFIA@DNBKABHHNANA@FD@KKADJAKNFCIJNJGIFCBLEDF",
        "scrambles",
        "333",
        "r1",
        "g1",
        "a1",
        "333",
        "sub1",
      ],
      "222",
    ),
  ).toThrow("Mismatched subevent in second-to-last level of hierarchy");
});

test("wasmFreeMemoryForAllScrambleFinders()", async () => {
  console.log("----------------");
  console.log(
    `Freed ${wasmFreeMemoryForAllScrambleFinders()} scramble finders.`,
  );
});
