// TODO: remove once `esbuild` supports relative URLs: https://github.com/evanw/esbuild/pull/2508
export const workerURL = import.meta.url;

if (globalThis.WorkerGlobalScope) {
  (async () => {
    const [
      { cube3x3x3 },
      { setKPuzzleDefString, solveState, serializeDefToTws },
    ] = await Promise.all([import("cubing/puzzles"), import("..")]);

    await setKPuzzleDefString(
      serializeDefToTws(await cube3x3x3.kpuzzle(), {
        moveSubset: ["U", "L", "F", "R", "B", "D"],
        startState: `EDGES
  0 0 0 0 1 2 3 4 5 6 7 8
  0 0 0 0 0 0 0 0 0 0 0 0
  CORNERS
  0 0 0 0 1 2 3 4
  0 0 0 0 0 0 0 0
  CENTERS
  0 1 2 3 4 5
  0 0 0 0 0 0`,
      }),
    );
    (
      await solveState(`ScrambleState test
  EDGES
  0 0 0 0 1 2 3 4 5 6 7 8
  1 1 1 1 0 0 0 0 0 0 0 0
  CORNERS
  0 0 0 0 1 2 3 4
  0 0 0 0 0 0 0 0
  CENTERS
  0 1 2 3 4 5
  0 0 0 0 0 0
  End`)
    ).log();
  })();
}
