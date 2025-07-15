use std::collections::HashSet;

use cubing::{
    alg::{Alg, QuantumMove},
    kpuzzle::{KPattern, KPuzzle},
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

// pub fn scramble_baby_fto() -> Alg {
//     loop {
//         let mut rng = thread_rng();
//         // TODO: Have a consistent way to handle orientation (de)normalization.
//         let scramble_pattern = scramble_pattern
//             .apply_alg(
//                 [parse_alg!(""), parse_alg!("Rv Uv")]
//                     .choose(&mut rng)
//                     .unwrap(),
//             )
//             .unwrap();
//         let scramble_pattern = scramble_pattern
//             .apply_alg(
//                 [parse_alg!(""), parse_alg!("Rv'")]
//                     .choose(&mut rng)
//                     .unwrap(),
//             )
//             .unwrap();
//         let scramble_pattern = scramble_pattern
//             .apply_alg(
//                 [parse_alg!(""), parse_alg!("Uv"), parse_alg!("Uv'")]
//                     .choose(&mut rng)
//                     .unwrap(),
//             )
//             .unwrap();
//         if let Some(solution) =
//         {
//             return solution.invert();
//         }
//     }
// }

pub(crate) struct BabyFTOScrambleFinder {
    kpuzzle: KPuzzle,
    filtered_search: FilteredSearch<KPuzzle>,
    search: FilteredSearch<KPuzzle>,
}

impl Default for BabyFTOScrambleFinder {
    fn default() -> Self {
        let kpuzzle = baby_fto_kpuzzle();
        let filter_generator_moves = move_list_from_vec(vec!["U", "L", "F", "R"]);
        let filtered_search =
            <FilteredSearch>::new(IterativeDeepeningSearch::new_with_hash_prune_table(
                ImmutableSearchData::try_from_common_options_with_auto_search_generators(
                    kpuzzle.clone(),
                    filter_generator_moves,
                    vec![kpuzzle.default_pattern()],
                    Default::default(),
                )
                .unwrap(),
                StoredSearchAdaptations::default(),
                HashPruneTableSizeBounds::default(),
            ));

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
            filtered_search,
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
        self.filtered_search
            .filtering_decision(pattern, MoveCount(5))
    }
}

impl SolvingBasedScrambleFinder for BabyFTOScrambleFinder {
    fn generate_fair_unfiltered_random_pattern(
        &mut self,
        _scramble_options: &NoScrambleOptions,
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
