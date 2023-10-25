import assert from "node:assert";
import { cube2x2x2 } from "cubing/puzzles";
import {
  wasmRandomScrambleForEvent,
  wasmTwsearch,
} from "../dist/wasm/index.js";

for (const eventID of ["222", "pyram", "minx", ...new Array(10).fill("333")]) {
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
