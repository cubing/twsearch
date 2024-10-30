use cubing::alg::Alg;

use crate::{_internal::MoveCount, scramble::randomize::PieceZeroConstraint};

use super::{
    super::randomize::{
        randomize_orbit_naïve, OrbitOrientationConstraint, OrbitPermutationConstraint,
    },
    super::scramble_search::{generators_from_vec_str, simple_filtered_search},
    definitions::skewb_fixed_corner_with_co_tweaks_kpuzzle,
};

pub fn scramble_skewb() -> Alg {
    let kpuzzle = skewb_fixed_corner_with_co_tweaks_kpuzzle();
    loop {
        let mut scramble_pattern = kpuzzle.default_pattern();

        /* The total orientation of each corner orbit is constrained by the permutation of the other.
         * That is, suppose we have a valid state of Skewb with values labelled as follows:
         *
         * (Take note of the values highlighted by ↓↓ and ↑↑.)
         *
         *                                                               ↓↓
         *                                                               ↓↓
         * {
         *     "CORNERS1": { "pieces": [@2, @2, @2],     "orientation": [#1, @1, @1] },
         *     "CORNERS2": { "pieces": [@1, @1, @1, @1], "orientation": [#2, @2, @2, @2]},
         *     "CENTERS":  { … }
         * }                                                             ↑↑
         *                                                               ↑↑
         *
         * Then:
         *
         * - The orientation of value `#1` is determined by the values labeled `@1`.
         * - The orientation of value `#2` is determined by the values labeled `@2`.
         *
         * Now, we could either:
         *
         * - Do a bit of math to determine the values `#1` and `#2.`
         * - Set the orientations of `#1` and `#2` to "ignored" by using the `orientationMod` feature.
         *
         * We choose to do the latter (with respect to the solved state) while generating a random permutation of this pattern
         * (taking into account permutation parity for each orbit) and solve it. In the resulting state at the end of the solve:
         *
         * - All the `@1` values match the solved state, so the (uniquely determined) value of `#1` must also match the solved state.
         * - All the `@2` values match the solved state, so the (uniquely determined) value of `#2` must also match the solved state.
         *
         * That is: the entire puzzle is solved, and we can use this to return a uniform random scramble (subject to other filtering).
         *
         * This approach does not have any performance implications, and also has the benefit that it allows us to randomize each orbit independently.
         *
         * The numbers check out, as this gives us the following number of distinct states:
         *
         * | Orbit    | Calculation    | Number of possibilities |
         * |----------|----------------|-------------------------|
         * | CORNERS1 | 4! / 2 * 3^3   | 324                     |
         * | CORNERS2 | 3! / 2 * 3^2   | 27                      |
         * | CENTERS  | 6! / 2         | 360                     |
         * |----------|----------------|-------------------------|
         * | Overall  | 324 * 27 * 360 | 3149280                 |
         *
         * This matches: https://www.jaapsch.net/puzzles/skewb.htm
         */

        randomize_orbit_naïve(
            &mut scramble_pattern,
            0,
            "CORNERS1",
            OrbitPermutationConstraint::SingleOrbitEvenParity,
            OrbitOrientationConstraint::AnySum,
            PieceZeroConstraint::IgnoredOrientation,
        );

        randomize_orbit_naïve(
            &mut scramble_pattern,
            1,
            "CORNERS2",
            OrbitPermutationConstraint::SingleOrbitEvenParity,
            OrbitOrientationConstraint::AnySum,
            PieceZeroConstraint::IgnoredOrientation,
        );

        randomize_orbit_naïve(
            &mut scramble_pattern,
            2,
            "CENTERS",
            OrbitPermutationConstraint::SingleOrbitEvenParity,
            OrbitOrientationConstraint::OrientationsMustSumToZero,
            PieceZeroConstraint::AnyPositionAndOrientation,
        );

        let generators = generators_from_vec_str(vec!["U", "L", "R", "B"]); // TODO: cache
        if let Some(scramble) = simple_filtered_search(
            &scramble_pattern,
            generators,
            MoveCount(7),
            Some(MoveCount(11)),
        ) {
            return scramble;
        }
    }
}
