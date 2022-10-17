// import { setArgs, setKPuzzleDefString, solveScramble } from "./api";
import "./api.js";
import { setArgs, setKPuzzleDefString, solveScramble, solveState } from "./api.js";

(async () => { 
  await setArgs("--nowrite");
  await setKPuzzleDefString(`Name PuzzleGeometryPuzzle

Set CORNER 8 3

Solved
CORNER
1 2 3 4 5 6 7 8
0 0 0 0 0 0 0 0
End

Move F
CORNER
7 1 3 2 5 6 4 8
2 1 0 2 0 0 1 0
End

Move B
CORNER
1 2 5 4 8 3 7 6
0 0 1 0 2 2 0 1
End

Move D
CORNER
1 4 3 8 2 6 7 5
0 0 0 0 0 0 0 0
End

Move U
CORNER
3 2 6 4 5 7 1 8
0 0 0 0 0 0 0 0
End

Move L
CORNER
1 2 3 7 5 8 6 4
0 0 0 1 0 1 2 2
End

Move R
CORNER
2 5 1 4 3 6 7 8
1 2 2 0 1 0 0 0
End`);

console.log(await (solveScramble("U F R D B L U F R D B L U F R D B L")));
console.log(await (solveState(`Scramble CornerTwist
CORNER
1 2 3 4 5 6 7 8
1 2 0 0 0 0 0 0
End
`)));

//   await solveScramble("U F R D B L U F R D B L U F R D B L")
// cout << w_solvescramble("U F R D B L U F R D B L U F R D B L") << endl ;
// cout << w_solveposition(scrfile) << endl ;
// cout << w_solvescramble("U F R D B L U F R D B L U F R D B L") << endl ;
// cout << w_solveposition(scrfile) << endl ;

})()
