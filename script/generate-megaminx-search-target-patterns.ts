#!/usr/bin/env bun

import { writeFile } from "node:fs/promises";
import { $ } from "bun";
import type { Move } from "cubing/alg";
import { KPattern } from "cubing/kpuzzle";
import { puzzles } from "cubing/puzzles";

// biome-ignore lint/complexity/useLiteralKeys: Record access
const kpuzzle = await puzzles["megaminx"].kpuzzle();

const UFRtransformations = ["U", "F", "R"].map((move) =>
  kpuzzle.moveToTransformation(move),
);

function maskFromBlockingGenerators(
  blockingGenerators: (string | Move)[],
): KPattern {
  const pattern = new KPattern(
    kpuzzle,
    structuredClone(kpuzzle.defaultPattern().patternData),
  );
  const transformations = blockingGenerators.map((move) =>
    kpuzzle.moveToTransformation(move),
  );
  for (const orbitName of ["EDGES", "CORNERS"]) {
    const orbitDef = kpuzzle.lookupOrbitDefinition(orbitName);
    for (let i = 0; i < orbitDef.numPieces; i++) {
      let numAffectedUFR = 0;
      let ignoreThisPiece = false;
      for (const UFRtransformation of UFRtransformations) {
        if (
          UFRtransformation.transformationData[orbitName].permutation[i] !== i
        ) {
          numAffectedUFR++;
        }
      }
      if (numAffectedUFR >= 2) {
        ignoreThisPiece = true;
      }
      for (const transformation of transformations) {
        if (transformation.transformationData[orbitName].permutation[i] !== i) {
          ignoreThisPiece = true;
        }
      }
      if (ignoreThisPiece) {
        pattern.patternData[orbitName].pieces[i] = 0;
        pattern.patternData[orbitName].orientationMod ??= new Array(
          orbitDef.numPieces,
        ).fill(0);
        pattern.patternData[orbitName].orientationMod[i] = 1;
      }
    }
  }
  // biome-ignore lint/complexity/useLiteralKeys: Record access
  pattern.patternData["CENTERS"].orientationMod = new Array(12).fill(1);
  return pattern;
}

async function writeTargetPattern(
  phaseNumber: number,
  blockingGenerators: (string | Move)[],
) {
  await writeFile(
    `./src/rs/scramble/puzzles/definitions/megaminx-phase${phaseNumber}.target-pattern.json`,
    JSON.stringify(
      maskFromBlockingGenerators(blockingGenerators).patternData,
      null,
      "  ",
    ),
  );
}

await writeTargetPattern(1, [
  "U",
  "L",
  "F",
  "R",
  "BR",
  "BL",
  "FL",
  "FR",
  "DL",
  "DR",
  "B",
]);
await writeTargetPattern(2, ["U", "L", "F", "R", "BR", "BL", "FL", "FR", "DR"]);
await writeTargetPattern(3, ["U", "L", "F", "R", "BR", "BL", "FL", "FR"]);
await writeTargetPattern(4, ["U", "L", "F", "R", "BR", "BL", "FR"]);
await writeTargetPattern(5, ["U", "L", "F", "R", "BR", "BL"]);
await writeTargetPattern(6, ["U", "L", "F", "R", "BR"]);
await writeTargetPattern(7, ["U", "L", "F", "R"]);
await writeTargetPattern(8, ["U", "F", "R"]);
await writeTargetPattern(9, ["U", "R"]);
await writeTargetPattern(10, ["U"]);
await writeTargetPattern(11, []);

await $`bun x @biomejs/biome check --write "./src/rs/scramble/puzzles/definitions/"`;
