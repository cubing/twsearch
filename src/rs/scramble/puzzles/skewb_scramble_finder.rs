use cubing::{alg::Alg, kpuzzle::KPuzzle};

use crate::{
    _internal::{
        errors::SearchError,
        search::{
            iterative_deepening::iterative_deepening_search::IterativeDeepeningSearch,
            move_count::MoveCount,
        },
    },
    scramble::{
        collapse::collapse_adjacent_moves,
        randomize::{ConstraintForFirstPiece, OrbitRandomizationConstraints},
        scramble_search::{move_list_from_vec, FilteredSearch},
        solving_based_scramble_finder::{
            FilteringDecision, NoScrambleAssociatedData, NoScrambleOptions,
            SolvingBasedScrambleFinder,
        },
    },
};

use super::{
    super::randomize::{
        randomize_orbit_naïve, OrbitOrientationConstraint, OrbitPermutationConstraint,
    },
    definitions::skewb_fixed_corner_with_co_tweaks_kpuzzle,
};

pub(crate) struct SkewbScrambleFinder {
    kpuzzle: KPuzzle,
    filtered_search: FilteredSearch<KPuzzle>,
}

impl Default for SkewbScrambleFinder {
    fn default() -> Self {
        let kpuzzle = skewb_fixed_corner_with_co_tweaks_kpuzzle();
        let generator_moves = move_list_from_vec(vec!["U", "L", "R", "B"]);
        let filtered_search = FilteredSearch::new(
            IterativeDeepeningSearch::try_new(
                kpuzzle.clone(),
                generator_moves,
                kpuzzle.default_pattern(),
                Default::default(),
            )
            .unwrap(),
        );
        Self {
            kpuzzle: kpuzzle.clone(),
            filtered_search,
        }
    }
}

impl SolvingBasedScrambleFinder for SkewbScrambleFinder {
    type TPuzzle = KPuzzle;
    type ScrambleAssociatedData = NoScrambleAssociatedData;
    type ScrambleOptions = NoScrambleOptions;

    fn generate_fair_unfiltered_random_pattern(
        &mut self,
        _scramble_options: &Self::ScrambleOptions,
    ) -> (
        <<Self as SolvingBasedScrambleFinder>::TPuzzle as crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle>::Pattern,
        Self::ScrambleAssociatedData,
    ){
        let mut scramble_pattern = self.kpuzzle.default_pattern();

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
            OrbitRandomizationConstraints {
                permutation: Some(OrbitPermutationConstraint::EvenParity),
                first_piece: Some(ConstraintForFirstPiece::IgnoredOrientation),
                ..Default::default()
            },
        );

        randomize_orbit_naïve(
            &mut scramble_pattern,
            1,
            "CORNERS2",
            OrbitRandomizationConstraints {
                permutation: Some(OrbitPermutationConstraint::EvenParity),
                first_piece: Some(ConstraintForFirstPiece::IgnoredOrientation),
                ..Default::default()
            },
        );

        randomize_orbit_naïve(
            &mut scramble_pattern,
            2,
            "CENTERS",
            OrbitRandomizationConstraints {
                permutation: Some(OrbitPermutationConstraint::EvenParity),
                orientation: Some(OrbitOrientationConstraint::SumToZero),
                ..Default::default()
            },
        );
        (scramble_pattern, NoScrambleAssociatedData {})
    }

    fn filter_pattern(
        &mut self,
        pattern: &<<Self as SolvingBasedScrambleFinder>::TPuzzle as crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle>::Pattern,
        _scramble_associated_data: &Self::ScrambleAssociatedData,
        _scramble_options: &Self::ScrambleOptions,
    ) -> FilteringDecision {
        self.filtered_search
            .filtering_decision(pattern, MoveCount(7))
    }

    fn solve_pattern(
        &mut self,
        pattern: &<<Self as SolvingBasedScrambleFinder>::TPuzzle as crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle>::Pattern,
        _scramble_associated_data: &Self::ScrambleAssociatedData,
        _scramble_options: &Self::ScrambleOptions,
    ) -> Result<Alg, SearchError> {
        Ok(self
            .filtered_search
            .generate_scramble(pattern, Some(MoveCount(11))))
    }

    fn collapse_inverted_alg(&mut self, alg: Alg) -> Alg {
        collapse_adjacent_moves(alg, 3, -1)
    }
}
