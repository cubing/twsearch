// import { setArgs, setKPuzzleDefString, solveScramble } from "./api";
import "./api.js";
import { setArgs, setKPuzzleDefString, solveScramble, solveState } from "./api.js";
import { serializeDefToTws, serializeKTransformationDataToTws } from "./serialize.js";
import { cube3x3x3 } from "cubing/puzzles";

(async () => { 
  await setArgs("--nowrite");


  await setKPuzzleDefString(serializeDefToTws(await cube3x3x3.kpuzzle(), {
    moveSubset: ["U", "L", "F", "R", "B", "D"],
    startState: `EDGES
0 0 0 0 1 2 3 4 5 6 7 8
0 0 0 0 0 0 0 0 0 0 0 0
CORNERS
0 0 0 0 1 2 3 4
0 0 0 0 0 0 0 0
CENTERS
0 1 2 3 4 5
0 0 0 0 0 0`
  }));
  // console.log(await (solveState(
  //   serializeKTransformationDataToTws("wheee", (await cube3x3x3.kpuzzle()).algToTransformation("R' F R U R U' R' F' R U' R' U' R U' R' U' R U' R' U2 R U' R2 U2 R U R' U R")
  //     .transformationData, true)
  // )));
  // console.log(await (solveState(
  //     serializeKTransformationDataToTws("wheee", (await cube3x3x3.kpuzzle()).algToTransformation("R' F R U R U' R' F' R U' R' U' R U' R' U' R U' R' U2 R U' R2 U2 R U R' U R")
  //       .transformationData, true)
  //   )));
  console.log(await (solveState(`ScrambleState test
EDGES
0 0 0 0 1 2 3 4 5 6 7 8
1 1 1 1 0 0 0 0 0 0 0 0
CORNERS
0 0 0 0 1 2 3 4
0 0 0 0 0 0 0 0
CENTERS
0 1 2 3 4 5
0 0 0 0 0 0
End`)));


//   await setKPuzzleDefString(`Name 2x2x2

// Set CORNERS 8 3

// StartState
// CORNERS
// 0 1 2 3 4 5 6 7
// 0 0 0 0 0 0 0 0
// End

// MoveTransformation U
// CORNERS
// 1 2 3 0 4 5 6 7
// 0 0 0 0 0 0 0 0
// End

// MoveTransformation F
// CORNERS
// 3 1 2 5 0 4 6 7
// 1 0 0 2 2 1 0 0
// End

// MoveTransformation R
// CORNERS
// 4 0 2 3 7 5 6 1
// 2 1 0 0 1 0 0 2
// End
// `);

// // console.log(await (solveScramble("U F R U F R U F R U F R")));
// console.log(await (solveState(`ScrambleState wheee
// CORNERS
// 0 3 5 4 1 7 6 2
// 0 0 0 0 0 0 0 0
// End`)));

// //   await solveScramble("U F R D B L U F R D B L U F R D B L")
// // cout << w_solvescramble("U F R D B L U F R D B L U F R D B L") << endl ;
// // cout << w_solveposition(scrfile) << endl ;
// // cout << w_solvescramble("U F R D B L U F R D B L U F R D B L") << endl ;
// // cout << w_solveposition(scrfile) << endl ;

})()
