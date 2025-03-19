import { heapStats } from "bun:jsc";
import assert from "node:assert";
import { cube2x2x2 } from "cubing/puzzles";
import {
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
  console.log(heapStats());
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

console.log(`Freed ${wasmFreeMemoryForAllScrambleFinders()} scramble finders.`);

for (let i = 0; i < 10; i++) {
  await new Promise((resolve) => setTimeout(resolve, 5000));
  console.log(heapStats());
}
