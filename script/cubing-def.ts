#!/usr/bin/env bun

import { exit } from "node:process";
import { puzzles } from "cubing/puzzles";

const {
  binary,
  string: cmdString,
  command,
  positional,
  run,
} = await import("cmd-ts-too");

const app = command({
  name: "cubing-def",
  description: "Example: cubing-def 3x3x3",
  args: {
    puzzleID: positional({
      type: cmdString,
      displayName: "puzzle ID",
    }),
  },
  handler: async ({ puzzleID }) => {
    const puzzle = puzzles[puzzleID];
    if (!puzzle) {
      console.error("Invalid puzzle ID!");
      exit(1);
    }

    console.log(
      JSON.stringify((await puzzle.kpuzzle()).definition, null, "  "),
    );
  },
});

await run(binary(app), process.argv);
