#!/usr/bin/env -S bun run --

import { argv } from "node:process";
import { argument, choice, message, object } from "@optique/core";
import { run } from "@optique/run";
import { puzzles } from "cubing/puzzles";
import { Path } from "path-class";

const { puzzleID } = run(
  object({
    puzzleID: argument(choice(Object.keys(puzzles), { metavar: "PUZZLE" })),
  }),
  {
    programName: new Path(argv[1]).basename.path,
    description: message`Example: cubing-def 3x3x3`,
    help: "option",
    completion: {
      mode: "option",
      name: "plural",
    },
  },
);

const puzzle = puzzles[puzzleID];
if (!puzzle) {
  // This should have been prevented by the parser.
  throw new Error("Invalid puzzle ID!") as never;
}

console.log(JSON.stringify((await puzzle.kpuzzle()).definition, null, "  "));
