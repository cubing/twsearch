use cubing::{alg::Alg, kpuzzle::KPuzzle};

use crate::{
    _internal::{
        errors::SearchError,
        search::{idf_search::idf_search::IDFSearch, move_count::MoveCount},
    },
    scramble::{
        collapse::collapse_adjacent_moves,
        randomize::OrbitRandomizationConstraints,
        scramble_search::{move_list_from_vec, FilteredSearch},
        solving_based_scramble_finder::{
            FilteringDecision, NoScrambleOptions, SolvingBasedScrambleFinder,
        },
    },
};

use super::{
    super::randomize::{
        randomize_orbit_naïve, OrbitOrientationConstraint, OrbitPermutationConstraint,
    },
    definitions::pyraminx_kpuzzle,
    static_move_list::{add_random_suffixes_from, static_parsed_opt_list},
};

pub(crate) struct PyraminxScrambleAssociatedData {
    tip_randomization_alg: Alg,
}

pub(crate) struct PyraminxScrambleFinder {
    kpuzzle: KPuzzle,
    filtered_search_without_tips: FilteredSearch<KPuzzle>,
}

impl Default for PyraminxScrambleFinder {
    fn default() -> Self {
        let kpuzzle = pyraminx_kpuzzle();

        let generator_moves = move_list_from_vec(vec!["U", "L", "R", "B"]);
        let filtered_search_without_tips = <FilteredSearch>::new(
            IDFSearch::try_new(
                kpuzzle.clone(),
                generator_moves,
                kpuzzle.default_pattern(),
                Default::default(),
            )
            .unwrap(),
        );
        Self {
            kpuzzle: kpuzzle.clone(),
            filtered_search_without_tips,
        }
    }
}

impl SolvingBasedScrambleFinder for PyraminxScrambleFinder {
    type TPuzzle = KPuzzle;
    type ScrambleAssociatedData = PyraminxScrambleAssociatedData;
    type ScrambleOptions = NoScrambleOptions;

    fn generate_fair_unfiltered_random_pattern(
        &mut self,
        _scramble_options: &Self::ScrambleOptions,
    ) -> (
        <<Self as SolvingBasedScrambleFinder>::TPuzzle as crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle>::Pattern,
        Self::ScrambleAssociatedData,
    ){
        let mut scramble_pattern = self.kpuzzle.default_pattern();

        randomize_orbit_naïve(
            &mut scramble_pattern,
            0,
            "EDGES",
            OrbitRandomizationConstraints {
                permutation: Some(OrbitPermutationConstraint::EvenParity),
                orientation: Some(OrbitOrientationConstraint::SumToZero),
                ..Default::default()
            },
        );
        randomize_orbit_naïve(
            &mut scramble_pattern,
            1,
            "CORNERS",
            OrbitRandomizationConstraints {
                permutation: Some(OrbitPermutationConstraint::IdentityPermutation),
                ..Default::default()
            },
        );

        // TODO: `add_random_suffixes_from` is designed for BLD orientation, so
        // it is hardcoded for 2 entries. We should change it to accept 4, but
        // for now we just call it here twice with 2 each.
        let tip_randomization_alg = add_random_suffixes_from(
            add_random_suffixes_from(
                Alg::default(),
                [
                    static_parsed_opt_list(&["", "u", "u'"]),
                    static_parsed_opt_list(&["", "l", "l'"]),
                ],
            ),
            [
                static_parsed_opt_list(&["", "r", "r'"]),
                static_parsed_opt_list(&["", "b", "b'"]),
            ],
        );

        let scramble_pattern = scramble_pattern.apply_alg(&tip_randomization_alg).unwrap();

        (
            scramble_pattern,
            PyraminxScrambleAssociatedData {
                tip_randomization_alg,
            },
        )
    }

    fn filter_pattern(
        &mut self,
        pattern: &<<Self as SolvingBasedScrambleFinder>::TPuzzle as crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle>::Pattern,
        scramble_associated_data: &Self::ScrambleAssociatedData,
        _scramble_options: &Self::ScrambleOptions,
    ) -> FilteringDecision {
        // TODO: if `scramble_associated_data.tip_randomization_alg` is invalid, this will loop infinitely: https://github.com/cubing/twsearch/issues/94
        let pattern = pattern
            .apply_alg(&scramble_associated_data.tip_randomization_alg.invert())
            .unwrap(); // TODO
        self.filtered_search_without_tips.filtering_decision(
            &pattern,
            MoveCount(6 - scramble_associated_data.tip_randomization_alg.nodes.len()),
        )
    }

    fn solve_pattern(
        &mut self,
        pattern: &<<Self as SolvingBasedScrambleFinder>::TPuzzle as crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle>::Pattern,
        scramble_associated_data: &Self::ScrambleAssociatedData,
        _scramble_options: &Self::ScrambleOptions,
    ) -> Result<Alg, SearchError> {
        // TODO: if `scramble_associated_data.tip_randomization_alg` is invalid, this will loop infinitely: https://github.com/cubing/twsearch/issues/94
        let pattern = pattern
            .apply_alg(&scramble_associated_data.tip_randomization_alg.invert())
            .unwrap(); // TODO
        let alg_without_tips = self.filtered_search_without_tips.solve_or_error(
            &pattern,
            Some(MoveCount(
                11 - scramble_associated_data.tip_randomization_alg.nodes.len(),
            )),
        )?;
        Ok(Alg {
            nodes: [
                scramble_associated_data
                    .tip_randomization_alg
                    .invert()
                    .nodes,
                alg_without_tips.nodes,
            ]
            .concat(),
        })
    }

    fn collapse_inverted_alg(&mut self, alg: Alg) -> Alg {
        collapse_adjacent_moves(alg, 3, -1)
    }
}
