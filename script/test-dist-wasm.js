console.log("loading…");

import {
  default as init,
  wasmRandomScrambleForEvent,
} from "../dist/wasm/twsearch.js";

console.log("Initializating WASM");

await init();

console.log("Initialized!");
console.log("Found alg:", wasmRandomScrambleForEvent("222"));
console.log("Found alg:", wasmRandomScrambleForEvent("pyram"));
console.log("Found alg:", wasmRandomScrambleForEvent("333"));
