// TODO: Worker

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
): (...args: any[]) => Promise<any> {
	return async (...args: any[]) => {
		return (await emscriptenModule()).cwrap(fn, returnType, argTypes)(...args)
	};
}

export const setArgs: (s: string) => Promise<void> = cwrap("w_args", "void", [
	"string",
]);
export const setKPuzzleDefString: (s: string) => void = cwrap(
	"w_setksolve",
	"void",
	["string"],
);
export const solveScramble: (s: string) => void = cwrap(
	"w_solvescramble",
	"string",
	["string"],
);
export const solveState: (s: string) => void = cwrap(
	"w_solveposition",
	"string",
	["string"],
);
