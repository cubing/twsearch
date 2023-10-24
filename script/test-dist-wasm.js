console.log("loading…");

import { wasmRandomScrambleForEvent } from "../dist/wasm/index.js";

for (const eventID of ["222", "pyram", "minx", ...new Array(10).fill("333")]) {
  const startTime = performance.now();
  const scramble = await wasmRandomScrambleForEvent(eventID);
  console.log(
    `${scramble} // ${eventID} (${Math.floor(
      performance.now() - startTime,
    )}ms)`,
  );
}
