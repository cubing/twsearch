import type { KPuzzle, KStateData, KTransformationData } from "cubing/kpuzzle";

const BLANK_LINE = "";
const END = "End";

function sanitize(s: string): string {
  // @ts-ignore
  return s.replaceAll(/[^A-Za-z0-9]/g, "_");
}

export function serializeMoveTransformation(
  name: string,
  t: KTransformationData,
): string {
  const outputLines: string[] = [];
  outputLines.push(`MoveTransformation ${sanitize(name)}`);
  for (const [orbitName, orbitData] of Object.entries(t)) {
    outputLines.push(sanitize(orbitName));
    outputLines.push(orbitData.permutation.join(" "));
    outputLines.push(orbitData.orientation.join(" "));
  }
  outputLines.push(END);
  outputLines.push(BLANK_LINE);
  return outputLines.join("\n");
}

export function serializeScrambleState(name: string, t: KStateData): string {
  const outputLines: string[] = [];
  outputLines.push(`ScrambleState ${sanitize(name)}`);
  // outputLines.push(sanitize());
  for (const [orbitName, orbitData] of Object.entries(t)) {
    outputLines.push(sanitize(orbitName));
    outputLines.push(orbitData.pieces.join(" "));
    outputLines.push(orbitData.orientation.join(" "));
  }
  outputLines.push(END);
  outputLines.push(BLANK_LINE);
  return outputLines.join("\n");
}

export function serializeDefToTws(
  kpuzzle: KPuzzle,
  options?: { moveSubset?: string[]; startState?: string },
): string {
  let outputLines: string[] = [];
  const def = kpuzzle.definition;

  outputLines.push(`Name ${sanitize(def.name ?? "CustomPuzzle")}`);
  outputLines.push(BLANK_LINE);

  for (const [orbitName, orbitInfo] of Object.entries(def.orbits)) {
    outputLines.push(
      `Set ${sanitize(orbitName)} ${orbitInfo.numPieces} ${
        orbitInfo.numOrientations
      }`,
    );
  }
  outputLines.push(BLANK_LINE);

  outputLines.push("StartState");
  if (options?.startState) {
    outputLines.push(options?.startState);
  } else {
    for (const [orbitName, orbitDef] of Object.entries(def.startStateData)) {
      outputLines.push(sanitize(orbitName));
      outputLines.push(orbitDef.pieces.join(" "));
      outputLines.push(orbitDef.orientation.join(" "));
    }
  }
  outputLines.push(END);
  outputLines.push(BLANK_LINE);

  function include(moveName): boolean {
    if (options?.moveSubset) {
      return options.moveSubset.includes(moveName);
    } else {
      return true;
    }
  }

  for (const [moveName, moveDef] of Object.entries(def.moves)) {
    // console.log(moveName, include(moveName))
    if (include(moveName)) {
      outputLines.push(serializeMoveTransformation(moveName, moveDef));
    }
  }
  // console.log(def.experimentalDerivedMoves)
  for (const [moveName, moveAlgDef] of Object.entries(
    def.experimentalDerivedMoves ?? {},
  )) {
    // console.log(moveName, include(moveName))
    if (include(moveName)) {
      const transformation = kpuzzle.algToTransformation(moveAlgDef);
      outputLines.push(
        serializeMoveTransformation(
          moveName,
          transformation.transformationData,
        ),
      );
    }
  }

  return outputLines.join("\n");
}
