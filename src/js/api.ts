// TODO: Worker
import { Alg } from "cubing/alg";

type CWrap = (fn: string, returnType: string, argType: string[]) => any;

interface EmscriptenModule {
  cwrap: CWrap;
}

async function importOnce(): Promise<EmscriptenModule> {
  const fn = (await import("../../build/wasm-single-file/twsearch.mjs"))
    .default;
  return await fn();
}

let cachedEmscriptenModule: null | Promise<EmscriptenModule> = null;
async function emscriptenModule(): Promise<EmscriptenModule> {
  // rome-ignore lint/suspicious/noAssignInExpressions: Caching pattern
  return (cachedEmscriptenModule ??= importOnce());
}

function cwrap(
  fn: string,
  returnType: string,
  argTypes: string[],
  processReturnValue: (v: any) => any = (v) => v,
): (...args: any[]) => Promise<any> {
  const wrapped = (async () =>
    (await emscriptenModule()).cwrap(fn, returnType, argTypes))();
  return async (...args: any[]) => {
    return processReturnValue((await wrapped)(...args));
  };
}

export class NoSolutionError extends Error {}

function parseResult(s: string): Alg {
  if (s === "--no solution--") {
    throw new NoSolutionError("");
  }
  return Alg.fromString(s);
}

const stringArg = ["string"];
export const setArg: (s: string) => Promise<void> = cwrap(
  "wasm_api_set_arg",
  "void",
  stringArg,
);
export const setKPuzzleDefString: (s: string) => Promise<void> = cwrap(
  "wasm_api_set_kpuzzle_definition",
  "void",
  stringArg,
);
export const solveScramble: (s: string) => Promise<Alg> = cwrap(
  "wasm_api_solve_scramble",
  "string",
  stringArg,
  parseResult,
);
export const solveState: (s: string) => Promise<Alg> = cwrap(
  "wasm_api_solve_position",
  "string",
  stringArg,
  parseResult,
);
