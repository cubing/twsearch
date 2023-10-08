console.log("loading…");

import { default as init, invert_alg } from "../dist/wasm/twsearch.js";

console.log("Initializating WASM");

await init();

console.log("Initialized!");
console.log("Inverted alg test:", invert_alg("R U R'"));
