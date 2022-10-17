import { readFile, writeFile } from "fs/promises";

const wasmInPath = new URL("../build/wasm-wrapped/twsearch.wasm", import.meta.url).pathname
const wasmOutPath = new URL("../src/js/generated-wasm/twsearch.wasm.serialized.js", import.meta.url).pathname
const wasmBuffer = await readFile(wasmInPath);
await writeFile(wasmOutPath, `const wasmSourceBase64 = ${JSON.stringify(wasmBuffer.toString("base64"))};
export const wasmSourceDataURL = \`data:application/wasm;base64,\${wasmSourceBase64}\`
`, "utf-8");
