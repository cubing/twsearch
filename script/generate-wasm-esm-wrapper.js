import { readFile, writeFile } from "fs/promises";

const spaghettiInPath = new URL("../build/wasm-wrapped/twsearch.js", import.meta.url).pathname
const spaghettiOutPath = new URL("../src/js/generated-wasm/twsearch.esm-compatible.js", import.meta.url).pathname
const spaghettiString = await readFile(spaghettiInPath, "utf-8");
const spaghettiStringESMCompatible = spaghettiString
  .replace("module['exports']", "fakeBogusSillyVariable")
  .replace("wasmBinaryFile =", `import { wasmSourceDataURL } from "./twsearch.wasm.serialized.js"; wasmBinaryFile = wasmSourceDataURL; //`)
  .replace("var Module = typeof Module", `
let resolveInitialized;
export const emscriptenModule = new Promise((resolve) => {resolveInitialized = () => resolve(Module)})
const Module = {onRuntimeInitialized: () => resolveInitialized()}; // typeof Module`)
await writeFile(spaghettiOutPath, spaghettiStringESMCompatible, "utf-8");

