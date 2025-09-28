---
[English](https://github.com/b10101101/twsearch/blob/main/README_FORK_EN.md) 

[Русский](https://github.com/b10101101/twsearch/blob/main/README_FORK_RU.md) 

---

# Information About This Fork

The primary goal of this fork was to obtain a working Windows executable for the twsearch simulator. The secondary objective was to create a bundle of two simulators:

- [`twsearch`](https://github.com/cubing/twsearch)
- [`Geraniums Pot`](https://github.com/grigorusha/GeraniumsPot)

## About Geraniums Pot

Geraniums Pot is a simulator for puzzles of the "Intersecting Circles" class and simulates almost all existing puzzles of this type. Puzzle descriptions and move sequences use the original author's notation and are defined in text files.

A typical configuration description:

```
# ring number, center coordinates x y, radius, angle
Ring: 1,  0, 0, 50, 90
Ring: 2, 50, 0, 38.3, 45
```

This describes two circles (numbered 1 and 2) with center coordinates (0,0 and 50,0), radii (50 and 38.3), and minimal rotation angles (90 and 45) respectively.

Moves and move sequences in this puzzle are written in the following notation:

```
(1R1, 2L1, 1L3, 2R2),3
```

where:

`1R1` - rotate circle 1 clockwise by 1 minimal angle

`2L1` - rotate circle 2 counterclockwise by 1 minimal angle

`1L3` - rotate circle 1 counterclockwise by 3 minimal angles

`2R2` - rotate circle 2 clockwise by 2 minimal angles

`(...),3` - number of sequence repetitions after parentheses

Note that the emulator can process macros - user-recorded move sequences. When working in a bundle, it was necessary to convert move sequences output by [`twsearch`](https://github.com/cubing/twsearch) into [`Geraniums Pot`](https://github.com/grigorusha/GeraniumsPot) simulator notation.

## Puzzle Working Example

Let's consider the existing Arecibo puzzle. The twsearch emulator has its ready description [`Arecibo`](https://github.com/cubing/twsearch/blob/main/samples/main/arecibo.tws) with the following defined moves:

```
Move L
 ........
End

MoveSequence R Fv L Fv

Move Fv
 .... flip all elements ....
End
```

During algorithm search we get something like:

```
(L R L R' L' R' L' R)45
(L R L R L R' L' R L' R' L' R' L R')30
```

In Geraniums Pot, Arecibo is described as:

```
# ring number, center coordinates x y, radius, angle
Ring: 1, 50, 50, 50, 120 # left circle
Ring: 2, 91, 50, 50, 120 # right circle
```

Thus, to input formulas into Geraniums Pot they need to be converted to:

```
(L  R  L  R' L' R' L' R)45  corresponds to
(1R,2R,1R,2L,1L,2L,1L,2R),45

(L  R  L  R  L  R' L' R  L' R' L' R' L  R')30  corresponds to
(1R,2R,1R,2R,1R,2L,1L,2R,1L,2L,1L,2L,1R,2L),30
```

I might have made mistakes in this conversion as it's easy to err here. So I found another solution.

I create description in twsearch using moves like:

```
Solved
 ........ 
End

MoveAlias 1L 1R'
MoveAlias 2L 2R'

Move 1R
 ........
End

Move 2R
 ........
End
```

This way I immediately define moves closer to Geraniums Pot notation and get output like: `2R 1R 2R2' 1R 2R2 1R2' 2R' 1R2 2R' 1R' 2R 1R'` but this is also quite inconvenient - commas need to be added, and the inverse move `2R'` corresponds to `2L`. I needed to eliminate inverse moves. So I added the `ReplaceMoves` command to .tws file processing to replace "inconvenient" notation moves:

```
Solved
 ........ 
End

MoveAlias 1L 1R'
MoveAlias 2L 2R'
MoveAlias 3L 3R'

MoveSequence PP 2L 1R 2R 1L 2L 1R 2R 1L 2L 1R 2R 1L 
ReplaceMoves PP  (2L,1R,2R,1L),3
ReplaceMoves PP' (1R,2L,1L,2R),3

ReplaceMoves R' L
ReplaceMoves L' R
ReplaceMoves R2' L2
ReplaceMoves L2' R2
ReplaceMoves R3' L3
ReplaceMoves L3' R3
```

## Implemented Features

- Added makefile `Makefiles/cpp_win.Makefile`. This allows compiling ONLY `twsearch.exe` for Windows. The method suggested by the author:
  
  ```
  cl /o twsearch.exe /EHsc src\cpp\*.cpp src\cpp\cityhash\src\city.cc
  ```
  
  Didn't work for me, so I created my own Makefile. It produces `twsearch.exe` working in Windows multithreaded mode (`-t` option). To compile, run:
  
  ```
  mingw32-make -f ./Makefiles/cpp_win.Makefile
  ```
  
   or run the `compile_for_win64.bat` file`
  
  *Note: In this context I don't provide Rust implementation!*

- Added `--gp` option for outputting search results and algorithms in Geraniums Pot notation. This option replaces spaces with commas in algorithms with classical notation
  
  ```
  (1R 2L' 3R2)45 will be converted to (1R,2L',3R2),45
  (1R3 2R' 1L 2L')7 will be converted to (1R3,2R',1L,2L'),7
  ```
  
  *Note: This option only works when outputting regular sequences (without brackets or with round brackets). In "commutator" notation (`-A3` option), the comma serves a different purpose, so this method isn't implemented there.*

- Inverse moves replacement with `ReplaceMoves` command. Command usage examples:
  
  ```
  ReplaceMoves A B  
  → fromMove = "A", toMove = "B"
  
  ReplaceMoves U (1R2, 3L4, 5R1),3
  → fromMove = "U", toMove = "(1R2, 3L4, 5R1),3"
  
  ReplaceMoves F2 F F  
  → fromMove = "F2", toMove = "F F"
  
  ReplaceMoves SomeMove Another Move, with comma
  → fromMove = "SomeMove", toMove = "Another Move, with comma"
  ```
  
  *It's important to understand that the ReplaceMoves command doesn't participate in the solution or algorithm search process. Its effect applies ONLY to the output text*
  
  Let's apply this command in practice
  
  ```
  ReplaceMoves 2L' 2R
  ReplaceMoves 2R' 2L
  ```
  
  as a result we get
  
  ```
  (1R 2L' 3R2)45 will be converted to (1R 2R 3R2)45
  (1R3 2R' 1L 2L')7 will be converted to (1R3 2L 1L 2R)7
  
  and with --gp option added
  (1R 2L' 3R2)45 will be converted to (1R,2R,3R2),45
  (1R3 2R' 1L 2L')7 will be converted to (1R3,2L,1L,2R),7
  ```
  
  <u>After such conversion, you can select the resulting algorithm as text and paste it into the Geraniums Pot simulator by pressing the Ins or Ctrl+V button</u>

## Testing

- To study the program's work and search for new algorithms, I created a test puzzle similar to [Trio](https://twistypuzzles.com/app/museum/museum_showitem.php?pkey=1462), but with edge and triangle orientation. Located in folder `/test/trio_orientir.tws`. Also a test scramble for this puzzle with triangle rotation example in `/test/trio.scr`. You can run the test by clicking the `test.bat` file.`
  
  For process visualization, you can load the test puzzle file `/test/Trio_orientir.txt` into Geraniums Pot.
  
  *Please note! This puzzle's description includes the PP sequence*
  
  ```
  MoveSequence PP 2L 1R 2R 1L 2L 1R 2R 1L 2L 1R 2R 1L
  ```
  
  *This is a useful move that I often use when searching for algorithms, analogous to the (R U R' U')3 move in a regular Rubik's Cube, which swaps positions of 4 corner elements in pairs without changing edge positions. In the Trio puzzle and similar ones, it can also be used. This will produce a clean pairwise exchange of 4 triangles with rotations. Using similar sequences increases solution search speed but increases the move count of the resulting solution.**

## Installation in Windows Environment:

1. To compile the program in Windows I used MinGW-w64 which can be downloaded:
- From the official page https://www.mingw-w64.org/ 

- Or, in my case, I used a ready-made standalone build https://github.com/brechtsanders/winlibs_mingw
2. Download the version matching your requirements. I downloaded and installed on my Windows 8 Pro 64bit system the archive https://github.com/brechtsanders/winlibs_mingw/releases/download/15.2.0posix-13.0.0-msvcrt-r1/winlibs-x86_64-posix-seh-gcc-15.2.0-mingw-w64msvcrt-13.0.0-r1.7z.

3. Extract it to working folder c:\mingw64\

4. Next, add this folder's path to the PATH variable:
- Press Win + R, type sysdm.cpl. "Environment Variables" → "System variables" find Path

- Click "Edit" → "New" → Add path to your MinGW's bin folder: C:\mingw64\bin
5. Download and extract https://github.com/b10101101/twsearch/. For example, mine is located in folder d:\my_twsearch\twsearch

6. Open Windows command line: press Win + R, type cmd. Further work is done in command line.

7. Navigate to project folder cd d:\my_twsearch\twsearch.

8. To get the .exe executable file, run the compile_for_win64.bat file. The resulting file will be located in the folder `/build/win64/twsearch.exe`

9. To delete object files and the compiled file, run the clean.bat file.