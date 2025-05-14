use std::collections::HashSet;

use cubing::{
    alg::{Alg, QuantumMove},
    kpuzzle::{KPattern, KPuzzle},
    puzzles::cube2x2x2_kpuzzle,
};

use crate::{
    _internal::{
        canonical_fsm::canonical_fsm::CanonicalFSMConstructionOptions,
        errors::SearchError,
        search::{
            filter::filtering_decision::FilteringDecision,
            hash_prune_table::HashPruneTableSizeBounds,
            iterative_deepening::{
                iterative_deepening_search::{
                    ImmutableSearchData, ImmutableSearchDataConstructionOptions,
                    IterativeDeepeningSearch,
                },
                search_adaptations::StoredSearchAdaptations,
            },
            move_count::MoveCount,
        },
    },
    scramble::{
        collapse::collapse_adjacent_moves,
        get_kpuzzle::GetKPuzzle,
        randomize::OrbitRandomizationConstraints,
        scramble_finder::{
            scramble_finder::ScrambleFinder,
            solving_based_scramble_finder::{NoScrambleOptions, SolvingBasedScrambleFinder},
        },
        scramble_search::{move_list_from_vec, FilteredSearch},
    },
};

use super::{
    super::randomize::{randomize_orbit, OrbitOrientationConstraint},
    canonicalizing_solved_kpattern_depth_filter::{
        CanonicalizingSolvedKPatternDepthFilter,
        CanonicalizingSolvedKPatternDepthFilterConstructionParameters,
    },
    definitions::cube2x2x2_orientation_canonicalization_kpattern,
};

#[allow(non_snake_case)] // Move meanings are case sensitive.
pub(crate) struct Cube2x2x2ScrambleFinder {
    kpuzzle: KPuzzle,
    depth_filtering_search: CanonicalizingSolvedKPatternDepthFilter,
    search: FilteredSearch<KPuzzle>,
}

impl Default for Cube2x2x2ScrambleFinder {
    fn default() -> Self {
        let kpuzzle = cube2x2x2_kpuzzle();

        let depth_filtering_search = CanonicalizingSolvedKPatternDepthFilter::try_new(
            CanonicalizingSolvedKPatternDepthFilterConstructionParameters {
                canonicalization_mask: cube2x2x2_orientation_canonicalization_kpattern().clone(),
                canonicalization_generator_moves: move_list_from_vec(vec!["x", "y"]),
                max_canonicalizing_move_count_below: MoveCount(4),
                solved_pattern: cube2x2x2_kpuzzle().default_pattern(),
                depth_filtering_generator_moves: move_list_from_vec(vec!["U", "F", "R"]),
                min_optimal_solution_move_count: MoveCount(4), // store associated with events?
            },
        )
        .unwrap();

        #[allow(non_snake_case)] // Move meanings are case sensitive.
        let search = <FilteredSearch>::new(IterativeDeepeningSearch::new_with_hash_prune_table(
            ImmutableSearchData::try_from_common_options_with_auto_search_generators(
                kpuzzle.clone(),
                move_list_from_vec(vec!["U", "L", "F", "R"]),
                vec![kpuzzle.default_pattern()],
                ImmutableSearchDataConstructionOptions {
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
            StoredSearchAdaptations::default(),
            HashPruneTableSizeBounds::default(),
        ));
        Self {
            kpuzzle: kpuzzle.clone(),
            depth_filtering_search,
            search,
        }
    }
}

impl ScrambleFinder for Cube2x2x2ScrambleFinder {
    type TPuzzle = KPuzzle;
    type ScrambleOptions = NoScrambleOptions;

    fn filter_pattern(
        &mut self,
        pattern: &KPattern,
        _scramble_options: &Self::ScrambleOptions,
    ) -> FilteringDecision {
        self.depth_filtering_search.depth_filter(pattern).unwrap() // TODO: avoid `.unwrap()`.
    }
}

impl SolvingBasedScrambleFinder for Cube2x2x2ScrambleFinder {
    fn generate_fair_unfiltered_random_pattern(
        &mut self,
        _scramble_options: &Self::ScrambleOptions,
    ) -> KPattern {
        let mut scramble_pattern = self.kpuzzle.default_pattern();
        randomize_orbit(
            &mut scramble_pattern,
            0,
            "CORNERS",
            OrbitRandomizationConstraints {
                orientation: Some(OrbitOrientationConstraint::SumToZero),
                ..Default::default()
            },
        );
        scramble_pattern
    }

    fn solve_pattern(
        &mut self,
        pattern: &KPattern,
        _scramble_options: &Self::ScrambleOptions,
    ) -> Result<Alg, SearchError> {
        self.search.solve_or_error(pattern, Some(MoveCount(11)))
    }

    fn collapse_inverted_alg(&mut self, alg: Alg) -> Alg {
        collapse_adjacent_moves(alg, 4, -1)
    }
}

impl GetKPuzzle for Cube2x2x2ScrambleFinder {
    fn get_kpuzzle(&self) -> &KPuzzle {
        &self.kpuzzle
    }
}

#[cfg(test)]
mod tests {
    use cubing::{
        alg::{parse_alg, Alg},
        puzzles::cube2x2x2_kpuzzle,
    };

    use crate::scramble::scramble_finder::{
        scramble_finder::ScrambleFinder, solving_based_scramble_finder::NoScrambleOptions,
    };

    use super::Cube2x2x2ScrambleFinder;

    #[test]
    // TODO: generalize and automate this across all events.
    fn simple_scramble_filtering_test() -> Result<(), String> {
        let mut scramble_finder = Cube2x2x2ScrambleFinder::default();
        let pattern = |alg: &Alg| {
            cube2x2x2_kpuzzle()
                .default_pattern()
                .apply_alg(alg)
                .unwrap()
        };
        assert!(scramble_finder
            .filter_pattern(&pattern(parse_alg!("U F R x y")), &NoScrambleOptions {},)
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(&pattern(parse_alg!("z")), &NoScrambleOptions {},)
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(&pattern(parse_alg!("x y x")), &NoScrambleOptions {},)
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(&pattern(parse_alg!("L F2 R")), &NoScrambleOptions {},)
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(&pattern(parse_alg!("R2 F2 U2 R2")), &NoScrambleOptions {},)
            .is_accept());
        Ok(())
    }
}
