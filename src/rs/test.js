console.log("loading…");

import { internal_init, invert_alg } from "./pkg/twsearch_bg.js";

console.log("Initializating WASM");

await internal_init();

console.log("Initialized!");
console.log("Inverted alg test:", invert_alg("R U R'"));

// if (!globalThis.document) {
//   console.info("Not running in a browser. Exiting!");
//   process.exit(1);
// }

// const input = document.querySelector(".input");
// function register(elem, f) {
//   const output = elem.querySelector(".output");
//   const durationElem = elem.querySelector(".duration");
//   durationElem.textContent = "";
//   input.addEventListener("input", () => {
//     try {
//       const start = performance.now();
//       output.textContent = f(input.value);
//       const duration = performance.now() - start;
//       durationElem.textContent = ` (≈${Math.round(duration * 1_000)}µs)`;
//       output.classList.remove("error");
//     } catch (e) {
//       output.textContent = e;
//       output.classList.add("error");
//     }
//   });
// }

// register(document.querySelector("#rust"), invert_alg);
// register(document.querySelector("#js"), (s) => {
//   return new Alg(s).invert().toString();
// });

// document.getElementById("use-wr").addEventListener("click", () => {
//   input.value = `y x' // inspection
// U R2 U' F' L F' U' L' // XX-Cross + EO
// U' R U R' // 3rd slot
// R' U R U2' R' U R // 4th slot
// U R' U' R U' R' U2 R // OLL / ZBLL
// U // AUF

// // from http://cubesolv.es/solve/5757`;
//   input.dispatchEvent(new CustomEvent("input"));
// });
