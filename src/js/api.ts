// TODO: Worker
import { Alg } from "cubing/alg";

type CWrap = (fn: string, returnType: string, argType: string[]) => any;

interface EmscriptenModule {
	cwrap: CWrap;
}

async function importOnce(): Promise<EmscriptenModule> {
	const {emscriptenModule} = (await import(
		"./generated-wasm/twsearch.esm-compatible.js",
	));
	return emscriptenModule;
}

let cachedEmscriptenModule: null | Promise<
	 EmscriptenModule
> = null;
async function emscriptenModule(): Promise<EmscriptenModule> {
	return cachedEmscriptenModule ??= importOnce();
}

function cwrap(
	fn: string,
	returnType: string,
	argTypes: string[],
	processReturnValue: (v: any) => any = (v) => v
): (...args: any[]) => Promise<any> {
	const wrapped = (async () => (await emscriptenModule()).cwrap(fn, returnType, argTypes) )()
	return async (...args: any[]) => {
		return processReturnValue((await wrapped)(...args))
	};
}

export const setArgs: (s: string) => Promise<void> = cwrap("w_args", "void", [
	"string",
]);
export const setKPuzzleDefString: (s: string) => Promise<void> = cwrap(
	"w_setksolve",
	"void",
	["string"],
);
export const solveScramble: (s: string) => Promise<Alg> = cwrap(
	"w_solvescramble",
	"string",
	["string"],
	Alg.fromString
);
export const solveState: (s: string) => Promise<Alg> = cwrap(
	"w_solveposition",
	"string",
	["string"],
	Alg.fromString
);
