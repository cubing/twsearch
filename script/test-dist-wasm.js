console.log("loading…");

import {
  default as init,
  wasmRandomScrambleForEvent,
} from "../dist/wasm/twsearch_wasm.js";

console.log("Initializating WASM");

await init();

console.log("Initialized!");

for (const eventID of ["222", "pyram", "minx", ...new Array(10).fill("333")]) {
  const startTime = performance.now();
  const scramble = await wasmRandomScrambleForEvent(eventID);
  console.log(
    `${scramble} // ${eventID} (${Math.floor(
      performance.now() - startTime,
    )}ms)`,
  );
}
