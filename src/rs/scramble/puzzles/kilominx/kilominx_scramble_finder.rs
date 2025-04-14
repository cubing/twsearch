use cubing::{
    alg::{parse_alg, Alg, Move},
    kpuzzle::{KPattern, KPuzzle},
};

use crate::{
    _internal::{
        errors::SearchError,
        search::{filter::filtering_decision::FilteringDecision, move_count::MoveCount},
    },
    experimental_lib_api::{
        ConstantAlgSearchPhase, KPuzzleSimpleMaskPhase, KPuzzleSimpleMaskPhaseConstructionOptions,
        MultiPhaseSearch, MultiPhaseSearchOptions,
    },
    scramble::{
        collapse::collapse_adjacent_moves,
        get_kpuzzle::GetKPuzzle,
        puzzles::{
            definitions::{
                kilominx_phase2_mask_kpattern, kilominx_phase2_target_kpattern,
                kilominx_phase3_target_kpattern,
            },
            kpattern_to_ktransformation::invert_kpattern_as_transformation,
        },
        randomize::{OrbitPermutationConstraint, OrbitRandomizationConstraints},
        scramble_finder::{
            scramble_finder::ScrambleFinder,
            solving_based_scramble_finder::{NoScrambleOptions, SolvingBasedScrambleFinder},
        },
        scramble_search::move_list_from_vec,
    },
};

use super::{
    super::{
        super::randomize::{randomize_orbit, OrbitOrientationConstraint},
        canonicalizing_solved_kpattern_depth_filter::{
            CanonicalizingSolvedKPatternDepthFilter,
            CanonicalizingSolvedKPatternDepthFilterConstructionParameters,
        },
        definitions::{kilominx_kpuzzle, kilominx_orientation_canonicalization_kpattern},
    },
    phase1::KilominxPhase1Search,
};

pub fn kilominx_front_moves() -> Vec<Move> {
    move_list_from_vec(vec!["U", "L", "F", "R", "FL", "FR"])
}

#[allow(non_snake_case)] // Move meanings are case sensitive.
pub(crate) struct KilominxScrambleFinder {
    kpuzzle: KPuzzle,
    depth_filtering_search: CanonicalizingSolvedKPatternDepthFilter,
    multi_phase_search: MultiPhaseSearch<KPuzzle>,
}

impl Default for KilominxScrambleFinder {
    fn default() -> Self {
        // TODO: solve the inverse for erogonomics?

        let kpuzzle = kilominx_kpuzzle();
        let depth_filtering_search = CanonicalizingSolvedKPatternDepthFilter::try_new(
            CanonicalizingSolvedKPatternDepthFilterConstructionParameters {
                canonicalization_mask: kilominx_orientation_canonicalization_kpattern().clone(),
                canonicalization_generator_moves: move_list_from_vec(vec!["Uv", "Rv"]),
                max_canonicalizing_move_count_below: MoveCount(5),
                solved_pattern: kpuzzle.default_pattern(),
                depth_filtering_generator_moves: move_list_from_vec(vec![
                    "U", "L", "F", "R", "BR", "BL", "FL", "FR", "DR", "Uw", "Fw", "Rw",
                ]),
                min_optimal_solution_move_count: MoveCount(5), // store associated with events?
            },
        )
        .unwrap();

        let multi_phase_search = MultiPhaseSearch::try_new(
            kpuzzle.clone(),
            vec![
                // TODO: skip phases 1 and its following `x2` when none of the back pieces are on the back.
                Box::new(KilominxPhase1Search::default()),
                Box::new(ConstantAlgSearchPhase {
                    phase_name: "first flip".to_owned(),
                    alg: parse_alg!("x2").to_owned(),
                }),
                Box::new(
                    KPuzzleSimpleMaskPhase::try_new(
                        "solve B pieces on F".to_owned(),
                        kilominx_phase2_mask_kpattern().clone(),
                        kilominx_front_moves(),
                        KPuzzleSimpleMaskPhaseConstructionOptions {
                            masked_target_patterns: Some(vec![
                                kilominx_phase2_target_kpattern().clone()
                            ]),
                            ..Default::default()
                        },
                    )
                    .unwrap(),
                ),
                Box::new(ConstantAlgSearchPhase {
                    phase_name: "second flip".to_owned(),
                    alg: parse_alg!("x2").to_owned(),
                }),
                Box::new(
                    KPuzzleSimpleMaskPhase::try_new(
                        "solve DL and D".to_owned(),
                        kilominx_phase3_target_kpattern().clone(),
                        move_list_from_vec(vec!["U", "L", "F", "R", "FL", "FR", "D"]),
                        KPuzzleSimpleMaskPhaseConstructionOptions {
                            ..Default::default()
                        },
                    )
                    .unwrap(),
                ),
                Box::new(
                    // TODO: we're not masking, there should probably be something simpler than `KPuzzleSimpleMaskPhase` for us to use.
                    KPuzzleSimpleMaskPhase::try_new(
                        "solve DL and D".to_owned(),
                        kpuzzle.default_pattern().clone(),
                        move_list_from_vec(vec!["U", "F", "R"]),
                        KPuzzleSimpleMaskPhaseConstructionOptions {
                            ..Default::default()
                        },
                    )
                    .unwrap(),
                ),
            ],
            MultiPhaseSearchOptions {
                // search_logger: SearchLogger {
                //     verbosity: VerbosityLevel::Info,
                // },
                ..Default::default()
            },
        )
        .unwrap();

        Self {
            kpuzzle: kpuzzle.clone(),
            depth_filtering_search,
            multi_phase_search,
        }
    }
}

impl ScrambleFinder for KilominxScrambleFinder {
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

impl SolvingBasedScrambleFinder for KilominxScrambleFinder {
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
                permutation: Some(OrbitPermutationConstraint::EvenParity),
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
        // We solve the inverse (and reverse the solution) to ensure the `x2` moves are at the start.
        let Some(pattern) = invert_kpattern_as_transformation(pattern) else {
            return Err("Could not invert pattern for solving".into());
        };
        Ok(self
            .multi_phase_search
            .chain_first_solution_for_each_phase(&pattern)?
            .invert())
    }

    fn collapse_inverted_alg(&mut self, alg: Alg) -> Alg {
        // TODO: this relies on the fact that `x2` is unmodified (even though it has order 2).
        collapse_adjacent_moves(alg, 5, -2)
    }
}

impl GetKPuzzle for KilominxScrambleFinder {
    fn get_kpuzzle(&self) -> &KPuzzle {
        &self.kpuzzle
    }
}
