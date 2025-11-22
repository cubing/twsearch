# Twizzle Pattern Searcher (`twips`)

A twisty puzzle search program to that can find algs and scrambles for [WCA](https://www.worldcubeassociation.org/regulations/) puzzles and a wide variety of other permutation puzzles. `twips` is inspired by [KSolve](https://github.com/cubing/ksolve) and uses concepts and algorithms from the C++ `twsearch` implementation and other state-of-the-art solvers. It is primarily designed for [KPuzzle](https://standards.cubing.net/draft/3/kpuzzle/) definitions but also provides an API for arbitrary implementations of semigroup search.

Twizzle Search powers alg search and scramble functionality for [Twizzle](https://alpha.twizzle.net/) and [`cubing.js`](https://github.com/cubing/cubing.js), and can be used from the commandline or as a library in many environments.

## Project Goals

1. Maintainability
      - We want `twips` to serve as a foundation for the cubing software ecosystem for a long time.
      - The project has multiple maintainers from the start, and we want to focus on a sustainable model for stewardship.
2. Ease of use
      - Powerful APIs that are easy to get started with.
      - Can either be used directly, or as a library in other projects.
      - Ability to scale from mobile devices all the way to native binaries that can fully utilize high-end hardware.
      - Can be used on any website through
          [`cubing.js`](https://js.cubing.net/cubing/), either by running in the
          browser itself or optionally connecting to a computer.
3. Performance
    - Great performance out of the box for a wide variety of puzzles.
    - Tunable optimizations for heavy-duty searches, including reusable prune tables for time-memory tradeoff.

## Example usage

Install using one of:

```shell
# Homebrew
brew install --HEAD cubing/cubing/twips

# From the source repo
git clone https://github.com/cubing/twips/ && cd twips
cargo install --path ./src/cli
twips completions <your shell> # Get completions for your shell

# Run without installing
git clone https://github.com/cubing/twips/ && cd twips
cargo run --release -- # Use this instead of `twips` in the commands below.
```

Examples (using files in the repo):

```shell
# Find 10 <U, F, R> algs for T-Perm
twips search \
  --generator-moves "U,F,R" \
  --min-num-solutions 10 \
  samples/json/3x3x3/3x3x3-Reid.def.json \
  samples/json/3x3x3/T-perm.scramble.json
```

```shell
# Generate scrambles
twips scramble --amount 7 sq1 2>/dev/null
```

```shell
# Solve a scramble for a known puzzle.
twips solve-known-puzzle 3x3x3 "U' F2 U' R2 F2 D' B2 D B2 U L2 U2 R2 L2 F2 L' D2 U2 B' U F2 R B' F L"
```

```shell
# Calculate the graphs for God's algorithm for 2×2×2
twips gods-algorithm \
  --generator-moves U,F,R \
  samples/json/2x2x2/2x2x2.kpuzzle.json
```

```shell
# Run a server for the web interface: https://experiments.cubing.net/cubing.js/twips/text-ui.html
twips serve
```

### Scrambles

The Rust implementation contains scrambling code intended to replace [`tnoodle-lib`](https://github.com/thewca/tnoodle-lib).

#### Derived scrambles

`twips` implements [a protocol to derive scrambles](./docs/ADRs/2025-11-02%20—%20Scramble%20derivation/2025-11-02%20—%20Scramble%20derivation.md) from a competition root seed (a 64-character hex string). Test like this:

```shell
twips \
  derive \
  67002dfc95e6d4288f418fbaa9150aa65b239fd5581f2d067d0293b9321a8b67 \
  EBNLEND@MABLNHJFHGFEKFIA@DNBKABHHNANA@FD@KKADJAKNFCIJNJGIFCBLEDF/scrambles/333/r1/g1/a1/333/sub1
```

#### Official events

| Event                   | Supported                                                                                                                                                                          | Min optimal solution moves                                                                                                                                                                   | Min scramble alg moves                                                                                                                                                                                                                                                                    | Prefix/Suffix                                                                                                                                    | Potential features                                                                                                                                   |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `333`, `333oh`, `333ft` | ✅ (MRSS)                                                                                                                                                                           | [✅](https://github.com/cubing/twips/blob/95203455c31b44dc6fdd85973ed6183bddbf7ced/src/rs/scramble/puzzles/cube3x3x3.rs#L181) ([2](https://www.worldcubeassociation.org/regulations/#4b3)) | ☑️ (N/A)                                                                                                                                                                                                                                                                                   | ☑️ (N/A)                                                                                                                                          |                                                                                                                                                      |
| `222`                   | ✅ (MRSS)                                                                                                                                                                           | [✅](https://github.com/cubing/twips/blob/95203455c31b44dc6fdd85973ed6183bddbf7ced/src/rs/scramble/puzzles/cube2x2x2.rs#L55) ([4](https://www.worldcubeassociation.org/regulations/#4b3b)) | [✅](https://github.com/cubing/twips/blob/05506b2a2c9e259eb3b3e09efbafbfbb34c0b18d/src/rs/scramble/puzzles/cube2x2x2.rs#L68) ([11](https://github.com/thewca/tnoodle/blob/d66eb2db5df7efcadf23828fe9211d0a30cfe2c4/webscrambles/src/main/resources/wca/readme-scramble.md?plain=1#L30)) | ☑️ (N/A)                                                                                                                                          |                                                                                                                                                      |
| `333bf`, `333mbf`       | ✅ (MRSS)                                                                                                                                                                           | ✅ ([2](https://www.worldcubeassociation.org/regulations/#4b3))                                                                                                                               | ☑️ (N/A)                                                                                                                                                                                                                                                                                   | [✅](https://github.com/cubing/twips/blob/95203455c31b44dc6fdd85973ed6183bddbf7ced/src/rs/scramble/puzzles/cube3x3x3.rs#L211) (wide moves)     |                                                                                                                                                      |
| `333fm`                 | ✅ (MRSS)                                                                                                                                                                           | ✅ ([2](https://www.worldcubeassociation.org/regulations/#4b3))                                                                                                                               | ☑️ (N/A)                                                                                                                                                                                                                                                                                   | [✅](https://github.com/cubing/twips/blob/05506b2a2c9e259eb3b3e09efbafbfbb34c0b18d/src/rs/scramble/puzzles/cube3x3x3.rs#L214-L241) (`R' U' F`) |                                                                                                                                                      |
| `444`                   | ✅ (MRSS)                                                                                                                                                                           | ✅ ([2](https://www.worldcubeassociation.org/regulations/#4b3))                                                                                                                               | ☑️ (N/A)                                                                                                                                                                                                                                                                                   | ☑️ (N/A)                                                                                                                                          |                                                                                                                                                      |
| `444bf`                 | ✅ (MRSS)                                                                                                                                                                           | ✅ ([2](https://www.worldcubeassociation.org/regulations/#4b3))                                                                                                                               | ☑️ (N/A)                                                                                                                                                                                                                                                                                   | ☑️ (not necessary)                                                                                                                                |                                                                                                                                                      |
| `555`                   | ✅ ([60 random moves](https://github.com/thewca/tnoodle/blob/d66eb2db5df7efcadf23828fe9211d0a30cfe2c4/webscrambles/src/main/resources/wca/readme-scramble.md?plain=1#L25))          | ✅ ([2](https://www.worldcubeassociation.org/regulations/#4b3))                                                                                                                               | ☑️ (N/A)                                                                                                                                                                                                                                                                                   | ☑️ (N/A)                                                                                                                                          | [layered randomization](https://github.com/thewca/tnoodle-lib/pull/40)                                                                               |
| `555bf`                 | ✅ ([60 random moves](https://github.com/thewca/tnoodle/blob/d66eb2db5df7efcadf23828fe9211d0a30cfe2c4/webscrambles/src/main/resources/wca/readme-scramble.md?plain=1#L25) + suffix) | ✅ ([2](https://www.worldcubeassociation.org/regulations/#4b3))                                                                                                                               | ☑️ (N/A)                                                                                                                                                                                                                                                                                   | ✅ (wide moves)                                                                                                                                   | [layered randomization](https://github.com/thewca/tnoodle-lib/pull/40)                                                                               |
| `666`                   | ✅ ([80 random moves](https://github.com/thewca/tnoodle/blob/d66eb2db5df7efcadf23828fe9211d0a30cfe2c4/webscrambles/src/main/resources/wca/readme-scramble.md?plain=1#L25))          | ✅ ([2](https://www.worldcubeassociation.org/regulations/#4b3))                                                                                                                               | ☑️ (N/A)                                                                                                                                                                                                                                                                                   | ☑️ (N/A)                                                                                                                                          | [layered randomization](https://github.com/thewca/tnoodle-lib/pull/40), [use `3Fw` instead of `3Bw`](https://github.com/cubing/cubing.js/issues/241) |
| `777`                   | ✅ ([100 random moves](https://github.com/thewca/tnoodle/blob/d66eb2db5df7efcadf23828fe9211d0a30cfe2c4/webscrambles/src/main/resources/wca/readme-scramble.md?plain=1#L25))         | ✅ ([2](https://www.worldcubeassociation.org/regulations/#4b3))                                                                                                                               | ☑️ (N/A)                                                                                                                                                                                                                                                                                   | ☑️ (N/A)                                                                                                                                          | [layered randomization](https://github.com/thewca/tnoodle-lib/pull/40)                                                                               |
| `clock`                 | ✅ (MRSS)                                                                                                                                                                           | ✅ ([2](https://www.worldcubeassociation.org/regulations/#4b3))                                                                                                                               | ☑️ (N/A)                                                                                                                                                                                                                                                                                   | ☑️ (N/A)                                                                                                                                          |                                                                                                                                                      |
| `minx`                  | ✅ ([random moves](https://www.worldcubeassociation.org/regulations/#4b3e) — [Pochmann style](https://www.worldcubeassociation.org/archive/forum_topics/368))                       | ✅ ([2](https://www.worldcubeassociation.org/regulations/#4b3))                                                                                                                               | ☑️ (N/A)                                                                                                                                                                                                                                                                                   | ☑️ (N/A)                                                                                                                                          | TODO: ask `xyzzy`                                                                                                                                    |
| `pyram`                 | ✅ (MRSS)                                                                                                                                                                           | [✅](https://github.com/cubing/twips/blob/067f0b0300efa9cfea2634c9795840d7d4b96aaf/src/rs/scramble/puzzles/pyraminx.rs#L61) ([6](https://www.worldcubeassociation.org/regulations/#4b3f))  | [✅](https://github.com/cubing/twips/blob/067f0b0300efa9cfea2634c9795840d7d4b96aaf/src/rs/scramble/puzzles/pyraminx.rs#L62) ([11](https://github.com/thewca/tnoodle/blob/d66eb2db5df7efcadf23828fe9211d0a30cfe2c4/webscrambles/src/main/resources/wca/readme-scramble.md?plain=1#L30))  | ☑️ (N/A)                                                                                                                                          |                                                                                                                                                      |
| `skewb`                 | ✅ (MRSS)                                                                                                                                                                           | [✅](https://github.com/cubing/twips/blob/067f0b0300efa9cfea2634c9795840d7d4b96aaf/src/rs/scramble/puzzles/skewb.rs#L108) ([7](https://www.worldcubeassociation.org/regulations/#4b3c))    | [✅](https://github.com/cubing/twips/blob/067f0b0300efa9cfea2634c9795840d7d4b96aaf/src/rs/scramble/puzzles/skewb.rs#L109) ([11](https://github.com/thewca/tnoodle/blob/d66eb2db5df7efcadf23828fe9211d0a30cfe2c4/webscrambles/src/main/resources/wca/readme-scramble.md?plain=1#L30))    | ☑️ (N/A)                                                                                                                                          |                                                                                                                                                      |
| `sq1`                   | ✅ (MRSS)                                                                                                                                                                           | ✅ ([11](https://www.worldcubeassociation.org/regulations/#4b3d))                                                                                                                             | ☑️ (N/A)                                                                                                                                                                                                                                                                                   | ☑️ (N/A)                                                                                                                                          |                                                                                                                                                      |

#### Unofficial events

| Event                    | Supported | Min optimal solution moves                                                                                                              | Min scramble alg moves                                                                                                                    | Prefix/Suffix | Potential features |
| ------------------------ | --------- | --------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ------------- | ------------------ |
| `fto`                    | 🚧 (MRSS)  | 🚧 (2 moves?)                                                                                                                            | ☑️ (N/A)                                                                                                                                   | ☑️ (N/A)       |                    |
| `master_tetraminx` | 🚧 (MRSS)  | 🚧 (2 moves?)                                                                                                                            | ☑️ (N/A)                                                                                                                                   | ☑️ (N/A)       |                    |
| `kilominx`               | ✅ (MRSS)  | ✅ (4 moves)                                                                                                                             | ☑️ (N/A)                                                                                                                                   | ☑️ (N/A)       |                    |
| `redi_cube`              | 🚧 (MRSS)  | 🚧 (2 moves?)                                                                                                                            | ☑️ (N/A)                                                                                                                                   | ☑️ (N/A)       |                    |
| `baby_fto`               | ✅ (MRSS)  | [✅](https://github.com/cubing/twips/blob/d49f32e5cc15b808eb1a8ca73707f9cda69883ee/src/rs/scramble/puzzles/baby_fto.rs#L91) (5 moves) | [✅](https://github.com/cubing/twips/blob/d49f32e5cc15b808eb1a8ca73707f9cda69883ee/src/rs/scramble/puzzles/baby_fto.rs#L125) (10 moves) | ☑️ (N/A)       |                    |

## Build tools

- Rust and `cargo` via [`rustup`](https://rustup.rs/).
  - Note that this project uses [`rust-toolchain.toml`](https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file), which effectively requires `rustup` to manage Rust toolchain versions.
- [`bun`](https://bun.sh/)

## License

This work is dual-licensed under the Mozilla Public License 2.0 and GPL 3.0 (or
any later version). If you use this work, you can choose either (or both) license terms to adhere to.

`SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later`
