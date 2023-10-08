console.log("loading…");

import {
  default as init,
  internal_init,
  search_test,
} from "../dist/wasm/twsearch.js";

console.log("Initializating WASM");

await init();
await internal_init();

console.log("Initialized!");
console.log("Found alg:", search_test());
