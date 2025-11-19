use std::collections::HashSet;

use cubing::{
    alg::{Alg, QuantumMove},
    kpuzzle::{KPattern, KPuzzle},
};
use rand::Rng;

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
        get_kpuzzle::GetKPuzzle,
        puzzles::{
            canonicalizing_solved_kpattern_depth_filter::{
                CanonicalizingSolvedKPatternDepthFilter,
                CanonicalizingSolvedKPatternDepthFilterConstructionParameters,
            },
            definitions::baby_fto_orientation_canonicalization_kpattern,
        },
        randomize::OrbitRandomizationConstraints,
        scramble_finder::{
            scramble_finder::ScrambleFinder,
            solving_based_scramble_finder::{NoScrambleOptions, SolvingBasedScrambleFinder},
        },
        scramble_search::{move_list_from_vec, FilteredSearch},
    },
};

use super::{
    super::randomize::{randomize_orbit, OrbitOrientationConstraint, OrbitPermutationConstraint},
    definitions::baby_fto_kpuzzle,
};

const BABY_FTO_MINIMUM_OPTIMAL_SOLUTION_MOVE_COUNT: MoveCount = MoveCount(5);

pub(crate) struct BabyFTOScrambleFinder {
    kpuzzle: KPuzzle,
    canonicalizing_solved_kpattern_depth_filter: CanonicalizingSolvedKPatternDepthFilter,
    search: FilteredSearch<KPuzzle>,
}

impl Default for BabyFTOScrambleFinder {
    fn default() -> Self {
        let kpuzzle = baby_fto_kpuzzle();
        let canonicalizing_solved_kpattern_depth_filter =
            CanonicalizingSolvedKPatternDepthFilter::try_new(
                CanonicalizingSolvedKPatternDepthFilterConstructionParameters {
                    canonicalization_mask: baby_fto_orientation_canonicalization_kpattern().clone(),
                    canonicalization_generator_moves: move_list_from_vec(vec!["Rv", "Uv"]),
                    max_canonicalizing_move_count_below: MoveCount(5),
                    solved_pattern: kpuzzle.default_pattern().clone(),
                    depth_filtering_generator_moves: move_list_from_vec(vec!["U", "L", "F", "R"]),
                    min_optimal_solution_move_count: BABY_FTO_MINIMUM_OPTIMAL_SOLUTION_MOVE_COUNT,
                },
            )
            .unwrap();

        let generator_moves = move_list_from_vec(vec!["U", "L", "F", "R", "BR"]);
        let search = <FilteredSearch>::new(IterativeDeepeningSearch::new_with_hash_prune_table(
            ImmutableSearchData::try_from_common_options_with_auto_search_generators(
                kpuzzle.clone(),
                generator_moves,
                vec![kpuzzle.default_pattern()],
                ImmutableSearchDataConstructionOptions {
                    canonical_fsm_construction_options: CanonicalFSMConstructionOptions {
                        forbid_adjacent_moves_by_quantums: vec![HashSet::from([
                            QuantumMove::new("L", None),
                            QuantumMove::new("BR", None),
                        ])],
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
            canonicalizing_solved_kpattern_depth_filter,
            search,
        }
    }
}

impl ScrambleFinder for BabyFTOScrambleFinder {
    type TPuzzle = KPuzzle;
    type ScrambleOptions = NoScrambleOptions;

    fn filter_pattern(
        &mut self,
        pattern: &KPattern,
        _scramble_options: &NoScrambleOptions,
    ) -> FilteringDecision {
        self.canonicalizing_solved_kpattern_depth_filter
            .depth_filter(pattern)
            .unwrap()
    }
}

impl SolvingBasedScrambleFinder for BabyFTOScrambleFinder {
    fn derive_fair_unfiltered_pattern<R: Rng>(
        &mut self,
        _scramble_options: &NoScrambleOptions,
        mut rng: R,
    ) -> KPattern {
        let mut scramble_pattern = self.kpuzzle.default_pattern();

        randomize_orbit(
            &mut scramble_pattern,
            1,
            "C4RNER",
            OrbitRandomizationConstraints {
                permutation: Some(OrbitPermutationConstraint::EvenParity),
                orientation: Some(OrbitOrientationConstraint::EvenOddHackSumToZero(vec![
                    1, 3, 4,
                ])),
                ..Default::default()
            },
            &mut rng,
        );

        for subset in [vec![0, 1, 2, 7], vec![3, 4, 5, 6]] {
            randomize_orbit(
                &mut scramble_pattern,
                0,
                "CENTERS",
                OrbitRandomizationConstraints {
                    permutation: Some(OrbitPermutationConstraint::EvenParity),
                    orientation: Some(OrbitOrientationConstraint::IgnoreAllOrientations),
                    subset: Some(subset),
                    ..Default::default()
                },
                &mut rng,
            );
        }

        scramble_pattern
    }

    fn solve_pattern(
        &mut self,
        pattern: &KPattern,
        _scramble_options: &NoScrambleOptions,
    ) -> Result<Alg, SearchError> {
        Ok(self.search.generate_scramble(pattern, Some(MoveCount(10))))
    }

    fn collapse_inverted_alg(&mut self, alg: Alg) -> Alg {
        alg
    }
}

impl GetKPuzzle for BabyFTOScrambleFinder {
    fn get_kpuzzle(&self) -> &KPuzzle {
        baby_fto_kpuzzle()
    }
}

#[cfg(test)]
mod tests {
    use cubing::alg::parse_alg;

    use crate::scramble::{
        puzzles::{baby_fto::BabyFTOScrambleFinder, definitions::baby_fto_kpuzzle},
        scramble_finder::{
            scramble_finder::ScrambleFinder, solving_based_scramble_finder::NoScrambleOptions,
        },
    };

    #[test]
    fn filter_3_mover() -> Result<(), String> {
        // Regression test for a 3-mover that requires reorientation to work.
        let alg = parse_alg!("F U F L R BR' R' F' L' U'");
        let pattern = baby_fto_kpuzzle().default_pattern().apply_alg(alg).unwrap();

        let mut baby_fto_scramble_finder = BabyFTOScrambleFinder::default();
        assert!(baby_fto_scramble_finder
            .filter_pattern(&pattern, &NoScrambleOptions {})
            .is_reject());

        Ok(())
    }
}
