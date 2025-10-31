import assert from "node:assert";
import { cube2x2x2 } from "cubing/puzzles";
import {
  wasmDeriveScrambleForEvent,
  wasmFreeMemoryForAllScrambleFinders,
  wasmRandomScrambleForEvent,
  wasmTwsearch,
} from "../dist/wasm/index";

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

{
  console.log("----------------");
  console.log("Performing a basic 2x2x2 search:");
  const kpuzzle = await cube2x2x2.kpuzzle();
  const scramble = "L' U' L U F U F'";
  console.log("Scramble:", scramble);
  const pattern = kpuzzle.defaultPattern().applyAlg(scramble);
  const solution = await wasmTwsearch(kpuzzle.definition, pattern, {
    generatorMoves: ["R", "U", "F"],
  });
  console.log("Solution:", solution.toString());
  assert(pattern.applyAlg(solution).isIdentical(kpuzzle.defaultPattern()));
}

{
  console.log("----------------");
  console.log("Deriving scrambles.");
  const scramble1 = await wasmDeriveScrambleForEvent(
    "67002dfc95e6d4288f418fbaa9150aa65b239fd5581f2d067d0293b9321a8b67",
    ["222", "222-r4", "set1", "attempt3-extra2", "scramble1"],
    "222",
  );
  const scramble2 = await wasmDeriveScrambleForEvent(
    "67002dfc95e6d4288f418fbaa9150aa65b239fd5581f2d067d0293b9321a8b67",
    ["222", "222-r4", "set1", "attempt3-extra2", "scramble1"],
    "222",
  );
  const scramble3 = await wasmDeriveScrambleForEvent(
    "6700abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdef",
    ["222", "222-r4", "set1", "attempt3-extra2", "scramble1"],
    "222",
  );

  assert(scramble1.isIdentical(scramble2));
  assert(!scramble1.isIdentical(scramble3));
}

console.log("----------------");
console.log(`Freed ${wasmFreeMemoryForAllScrambleFinders()} scramble finders.`);
