use std::collections::HashSet;

use cubing::{
    alg::{Alg, QuantumMove},
    kpuzzle::KPuzzle,
    puzzles::cube2x2x2_kpuzzle,
};

use crate::{
    _internal::{
        canonical_fsm::canonical_fsm::CanonicalFSMConstructionOptions,
        errors::SearchError,
        search::{
            idf_search::idf_search::{IDFSearch, IDFSearchConstructionOptions},
            move_count::MoveCount,
        },
    },
    scramble::{
        collapse::collapse_adjacent_moves,
        puzzles::static_move_list::{add_random_suffixes_from, static_parsed_opt_list},
        randomize::{ConstraintForFirstPiece, OrbitRandomizationConstraints},
        scramble_search::{move_list_from_vec, FilteredSearch},
        solving_based_scramble_finder::{
            FilteringDecision, NoScrambleOptions, SolvingBasedScrambleFinder,
        },
    },
};

use super::super::randomize::{randomize_orbit_naïve, OrbitOrientationConstraint};

#[allow(non_snake_case)] // Move meanings are case sensitive.
pub(crate) struct Cube2x2x2ScrambleFinder {
    kpuzzle: KPuzzle,
    filtered_search_L_B_D: FilteredSearch<KPuzzle>,
    filtered_search_U_L_F_R: FilteredSearch<KPuzzle>,
}

impl Default for Cube2x2x2ScrambleFinder {
    fn default() -> Self {
        let kpuzzle = cube2x2x2_kpuzzle();

        #[allow(non_snake_case)] // Move meanings are case sensitive.
        let filtered_search_L_B_D = <FilteredSearch>::new(
            IDFSearch::try_new(
                kpuzzle.clone(),
                move_list_from_vec(vec!["L", "B", "D"]),
                kpuzzle.default_pattern(),
                Default::default(),
            )
            .unwrap(),
        );

        #[allow(non_snake_case)] // Move meanings are case sensitive.
        let filtered_search_U_L_F_R = <FilteredSearch>::new(
            IDFSearch::try_new(
                kpuzzle.clone(),
                move_list_from_vec(vec!["U", "L", "F", "R"]),
                kpuzzle.default_pattern(),
                IDFSearchConstructionOptions {
                    canonical_fsm_construction_options: CanonicalFSMConstructionOptions {
                        forbid_transitions_by_quantums_either_direction: HashSet::from([(
                            QuantumMove::new("L", None),
                            QuantumMove::new("R", None),
                        )]),
                    },
                    ..Default::default()
                },
            )
            .unwrap(),
        );
        Self {
            kpuzzle: kpuzzle.clone(),
            filtered_search_L_B_D,
            filtered_search_U_L_F_R,
        }
    }
}

pub(crate) struct Cube2x2x2ScrambleAssociatedData {
    orientation_randomization_alg: Alg,
}

impl SolvingBasedScrambleFinder for Cube2x2x2ScrambleFinder {
    type TPuzzle = KPuzzle;
    type ScrambleAssociatedData = Cube2x2x2ScrambleAssociatedData;
    type ScrambleOptions = NoScrambleOptions;

    fn generate_fair_unfiltered_random_pattern(
        &mut self,
        _scramble_options: &Self::ScrambleOptions,
    ) -> (
        <<Self as SolvingBasedScrambleFinder>::TPuzzle as crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle>::Pattern,
        Self::ScrambleAssociatedData,
    ){
        let mut scramble_pattern_fixed_corner = self.kpuzzle.default_pattern();
        randomize_orbit_naïve(
            &mut scramble_pattern_fixed_corner,
            0,
            "CORNERS",
            OrbitRandomizationConstraints {
                orientation: Some(OrbitOrientationConstraint::SumToZero),
                first_piece: Some(ConstraintForFirstPiece::KeepSolved),
                ..Default::default()
            },
        );

        #[allow(non_snake_case)] // Move meanings are case sensitive.
        let orientation_randomization_U = static_parsed_opt_list(&["", "x", "x2", "x'", "z", "z'"]);
        #[allow(non_snake_case)] // Move meanings are case sensitive.
        let orientation_randomization_F = static_parsed_opt_list(&["", "y", "y2", "y'"]);
        let orientation_randomization_alg = add_random_suffixes_from(
            Alg::default(),
            [orientation_randomization_U, orientation_randomization_F],
        );

        let scramble_pattern = scramble_pattern_fixed_corner
            .apply_alg(&orientation_randomization_alg)
            .unwrap();

        (
            scramble_pattern,
            Cube2x2x2ScrambleAssociatedData {
                orientation_randomization_alg,
            },
        )
    }

    fn filter_pattern(
        &mut self,
        pattern: &<<Self as SolvingBasedScrambleFinder>::TPuzzle as crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle>::Pattern,
        scramble_associated_data: &Self::ScrambleAssociatedData,
        _scramble_options: &Self::ScrambleOptions,
    ) -> FilteringDecision {
        let oriented_pattern = pattern
            .apply_alg(
                &scramble_associated_data
                    .orientation_randomization_alg
                    .invert(),
            )
            .unwrap(); // TODO

        self.filtered_search_L_B_D
            .filtering_decision(&oriented_pattern, MoveCount(4))
    }

    fn solve_pattern(
        &mut self,
        pattern: &<<Self as SolvingBasedScrambleFinder>::TPuzzle as crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle>::Pattern,
        _scramble_associated_data: &Self::ScrambleAssociatedData,
        _scramble_options: &Self::ScrambleOptions,
    ) -> Result<Alg, crate::_internal::errors::SearchError> {
        let Some(alg) = self
            .filtered_search_U_L_F_R
            .solve(pattern, Some(MoveCount(11)))
        else {
            return Err(SearchError {
                description: "Could not solve 2×2×2 pattern".to_owned(),
            });
        };
        Ok(alg)
    }

    fn collapse_inverted_alg(&mut self, alg: Alg) -> Alg {
        collapse_adjacent_moves(alg, 4, -1)
    }
}
