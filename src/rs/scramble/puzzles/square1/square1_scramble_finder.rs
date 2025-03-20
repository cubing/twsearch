use crate::{
    _internal::search::filter::filtering_decision::FilteringDecision,
    scramble::{
        puzzles::square1::square1_shape_traversal_filter::shape_traversal_filter_pattern,
        solving_based_scramble_finder::{
            NoScrambleAssociatedData, NoScrambleOptions, SolvingBasedScrambleFinder,
        },
    },
};

use cubing::{alg::Alg, kpuzzle::KPattern};
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    _internal::search::{mask_pattern::apply_mask, move_count::MoveCount},
    scramble::randomize::{
        randomize_orbit_naïve, ConstraintForFirstPiece, OrbitRandomizationConstraints,
    },
};

use super::{
    super::definitions::{square1_square_square_shape_kpattern, square1_unbandaged_kpuzzle},
    depth_filtering::square1_depth_filtering_search_adaptations_without_prune_table,
    phase1::{square1_phase1_search_adaptations, Square1Phase1Pattern},
    phase2::{square1_phase2_search_adaptations, Square1Phase2Puzzle},
};
use cubing::alg::{parse_move, AlgBuilder, AlgNode, Grouping, Move};
use cubing::kpuzzle::KPuzzle;

use crate::scramble::scramble_search::FilteredSearch;
use crate::{
    _internal::search::{
        coordinates::graph_enumerated_derived_pattern_puzzle::GraphEnumeratedDerivedPatternPuzzle,
        iterative_deepening::iterative_deepening_search::IterativeDeepeningSearchConstructionOptions,
        prune_table_trait::Depth,
    },
    _internal::{
        errors::SearchError,
        search::iterative_deepening::iterative_deepening_search::{
            IndividualSearchOptions, IterativeDeepeningSearch,
        },
    },
    scramble::{
        puzzles::square1::phase1::Square1Phase1Puzzle, scramble_search::move_list_from_vec,
    },
};

const DEV_DEBUG_SQUARE1: bool = true;

// https://www.worldcubeassociation.org/regulations/#4b3d
const SQUARE_1_SCRAMBLE_MIN_OPTIMAL_MOVE_COUNT: MoveCount = MoveCount(11);

pub(crate) struct Square1ScrambleFinder {
    square1_phase1_puzzle: Square1Phase1Puzzle,
    phase1_iterative_deepening_search: IterativeDeepeningSearch<Square1Phase1Puzzle>,
    square1_phase2_puzzle: Square1Phase2Puzzle,
    phase2_iterative_deepening_search: IterativeDeepeningSearch<Square1Phase2Puzzle>,
    // TODO: lazy-initialize `depth_filtering_search`?
    depth_filtering_search: FilteredSearch<KPuzzle>,
}

impl Default for Square1ScrambleFinder {
    fn default() -> Self {
        let kpuzzle = square1_unbandaged_kpuzzle();
        let generator_moves = move_list_from_vec(vec!["U_SQ_", "D_SQ_", "/"]);

        let square1_phase1_puzzle: GraphEnumeratedDerivedPatternPuzzle<
            KPuzzle,
            Square1Phase1Pattern,
        > = Square1Phase1Puzzle::new(
            kpuzzle.clone(),
            kpuzzle.default_pattern(),
            generator_moves.clone(),
        );

        let phase1_target_pattern = square1_phase1_puzzle
            .full_pattern_to_derived_pattern(&kpuzzle.default_pattern())
            .unwrap();

        let phase1_iterative_deepening_search =
            IterativeDeepeningSearch::<Square1Phase1Puzzle>::legacy_try_new(
                square1_phase1_puzzle.clone(),
                generator_moves.clone(),
                vec![phase1_target_pattern],
                IterativeDeepeningSearchConstructionOptions {
                    ..Default::default()
                },
                square1_phase1_search_adaptations(square1_phase1_puzzle.clone()),
            )
            .unwrap();

        let square1_phase2_puzzle: Square1Phase2Puzzle = Square1Phase2Puzzle::new();
        let phase2_target_pattern = square1_phase2_puzzle
            .full_pattern_to_phase_coordinate(&kpuzzle.default_pattern())
            .unwrap();

        let phase2_iterative_deepening_search =
            IterativeDeepeningSearch::<Square1Phase2Puzzle>::legacy_try_new(
                square1_phase2_puzzle.clone(),
                generator_moves.clone(),
                vec![phase2_target_pattern],
                IterativeDeepeningSearchConstructionOptions {
                    ..Default::default()
                },
                square1_phase2_search_adaptations(square1_phase2_puzzle.clone()),
            )
            .unwrap();

        let depth_filtering_search = {
            let kpuzzle = square1_unbandaged_kpuzzle();
            let generator_moves = move_list_from_vec(vec!["U_SQ_", "D_SQ_", "/"]);

            let iterative_deepening_search =
                IterativeDeepeningSearch::<KPuzzle>::try_new_kpuzzle_with_hash_prune_table_shim(
                    kpuzzle.clone(),
                    generator_moves,
                    vec![kpuzzle.default_pattern()],
                    Default::default(),
                    Some(square1_depth_filtering_search_adaptations_without_prune_table()),
                )
                .unwrap();
            FilteredSearch::<KPuzzle>::new(iterative_deepening_search)
        };

        Self {
            square1_phase1_puzzle,
            phase1_iterative_deepening_search,
            square1_phase2_puzzle,
            phase2_iterative_deepening_search,
            depth_filtering_search,
        }
    }
}

fn flush_tuple(
    #[allow(non_snake_case)] current_tuple_U: &mut Option<Move>,
    #[allow(non_snake_case)] current_tuple_D: &mut Option<Move>,
    alg_builder: &mut AlgBuilder,
) {
    if current_tuple_U.is_some() || current_tuple_D.is_some() {
        let grouping = Grouping {
            alg: Alg {
                nodes: vec![
                    cubing::alg::AlgNode::MoveNode(
                        current_tuple_U
                            .take()
                            .unwrap_or_else(|| parse_move!("U_SQ_0").to_owned()),
                    ),
                    cubing::alg::AlgNode::MoveNode(
                        current_tuple_D
                            .take()
                            .unwrap_or_else(|| parse_move!("D_SQ_0").to_owned()),
                    ),
                ],
            }
            .into(),
            amount: 1,
        };
        let alg_node: AlgNode = grouping.into();
        alg_builder.push(&alg_node);
    };
}

fn group_square_1_tuples(alg: Alg) -> Alg {
    #[allow(non_snake_case)]
    let mut current_tuple_U: Option<Move> = None;
    #[allow(non_snake_case)]
    let mut current_tuple_D: Option<Move> = None;

    let mut alg_builder = AlgBuilder::default();

    #[allow(non_snake_case)]
    let U_SQ_0 = parse_move!("U_SQ_0");
    #[allow(non_snake_case)]
    let D_SQ_0 = parse_move!("D_SQ_0");
    #[allow(non_snake_case)]
    let _SLASH_ = parse_move!("/");

    // TODO: Push the candidate check into a trait for `IterativeDeepeningSearch`.
    for node in alg.nodes {
        let cubing::alg::AlgNode::MoveNode(r#move) = node else {
            panic!("Invalid Square-1 scramble alg in internal code.");
        };
        if r#move == *_SLASH_ {
            flush_tuple(&mut current_tuple_U, &mut current_tuple_D, &mut alg_builder);

            let alg_node: AlgNode = r#move.into();
            alg_builder.push(&alg_node)
        } else if r#move.quantum == U_SQ_0.quantum {
            #[allow(non_snake_case)]
            if let Some(current_tuple_U) = &mut current_tuple_U {
                // TODO: handle normalization elsewhere.
                current_tuple_U.amount = (current_tuple_U.amount + r#move.amount + 5) % 12 - 5;
            } else {
                current_tuple_U = Some(r#move);
            }
        } else if r#move.quantum == D_SQ_0.quantum {
            #[allow(non_snake_case)]
            if let Some(current_tuple_D) = &mut current_tuple_D {
                // TODO: handle normalization elsewhere.
                current_tuple_D.amount = (current_tuple_D.amount + r#move.amount + 5) % 12 - 5;
            } else {
                current_tuple_D = Some(r#move);
            }
        } else {
            panic!("Invalid Square-1 scramble alg in internal code.");
        }
    }

    flush_tuple(&mut current_tuple_U, &mut current_tuple_D, &mut alg_builder);
    alg_builder.to_alg()
}

pub fn debug_print_phase1_solutions_searched(found_phase1_solutions: usize, current_depth: usize) {
    if DEV_DEBUG_SQUARE1 {
        println!(
            "Searched {} phase 1 solution{} at depth {}.",
            found_phase1_solutions,
            if found_phase1_solutions == 1 { "" } else { "s" },
            current_depth
        );
    }
}

impl SolvingBasedScrambleFinder for Square1ScrambleFinder {
    type TPuzzle = KPuzzle;
    type ScrambleAssociatedData = NoScrambleAssociatedData;
    type ScrambleOptions = NoScrambleOptions;

    fn generate_fair_unfiltered_random_pattern(
        &mut self,
        _scramble_options: &Self::ScrambleOptions,
    ) -> (KPattern, Self::ScrambleAssociatedData) {
        let mut rng = thread_rng();

        loop {
            let mut scramble_pattern = square1_unbandaged_kpuzzle().default_pattern();

            let mut deep_wedges = vec![
                vec![0, 1],
                vec![2],
                vec![3, 4],
                vec![5],
                vec![6, 7],
                vec![8],
                vec![9, 10],
                vec![11],
                vec![12],
                vec![13, 14],
                vec![15],
                vec![16, 17],
                vec![18],
                vec![19, 20],
                vec![21],
                vec![22, 23],
            ];
            deep_wedges.shuffle(&mut rng);
            let wedge_orbit_info = &scramble_pattern.kpuzzle().clone().data.ordered_orbit_info[0];
            assert_eq!(wedge_orbit_info.name.0, "WEDGES");
            for (i, value) in deep_wedges.into_iter().flatten().enumerate() {
                unsafe {
                    scramble_pattern
                        .packed_orbit_data_mut()
                        .set_raw_piece_or_permutation_value(wedge_orbit_info, i as u8, value);
                }
            }

            randomize_orbit_naïve(
                &mut scramble_pattern,
                1,
                "EQUATOR",
                OrbitRandomizationConstraints {
                    first_piece: Some(ConstraintForFirstPiece::KeepSolved),
                    ..Default::default()
                },
            );

            // TODO: do this check without masking.
            let phase1_start_pattern =
                apply_mask(&scramble_pattern, square1_square_square_shape_kpattern()).unwrap();

            // Note: it is not safe in general to use a traversal filter for
            // scramble pattern filtering. However, this is safe here due to the
            // properties of the Square-1 puzzle.
            if shape_traversal_filter_pattern(&phase1_start_pattern).is_accept() {
                return (scramble_pattern, NoScrambleAssociatedData {});
            }
        }
    }

    fn filter_pattern(
        &mut self,
        pattern: &KPattern,
        _scramble_associated_data: &Self::ScrambleAssociatedData,
        _scramble_options: &Self::ScrambleOptions,
    ) -> FilteringDecision {
        self.depth_filtering_search
            .filtering_decision(pattern, SQUARE_1_SCRAMBLE_MIN_OPTIMAL_MOVE_COUNT)
    }

    fn solve_pattern(
        &mut self,
        pattern: &KPattern,
        _scramble_associated_data: &Self::ScrambleAssociatedData,
        _scramble_options: &Self::ScrambleOptions,
    ) -> Result<cubing::alg::Alg, crate::_internal::errors::SearchError> {
        let Ok(phase1_start_pattern) = self
            .square1_phase1_puzzle
            .full_pattern_to_derived_pattern(pattern)
        else {
            return Err(SearchError {
                description: "invalid pattern".to_owned(),
            });
        };

        // let start_time = Instant::now();
        // let mut phase1_start_time = Instant::now();
        for current_depth in 0..31 {
            let num_solutions = 10000000;
            let phase1_search = self.phase1_iterative_deepening_search.search(
                &phase1_start_pattern,
                IndividualSearchOptions {
                    min_num_solutions: Some(num_solutions),
                    min_depth: Some(Depth(current_depth)),
                    max_depth: Some(Depth(current_depth + 1)),
                    ..Default::default()
                },
            );
            // let mut num_phase2_starts = 0;
            // let mut phase1_cumulative_time = Duration::default();
            // let mut phase2_cumulative_time = Duration::default();
            #[allow(non_snake_case)]
            let _SLASH_ = parse_move!("/");
            let mut found_phase1_solutions = 0;
            for phase1_solution in phase1_search {
                found_phase1_solutions += 1;
                // num_phase2_starts += 1;
                // let phase2_start_time = Instant::now();
                let Ok(phase2_start_pattern) =
                    self.square1_phase2_puzzle.full_pattern_to_phase_coordinate(
                        &pattern.apply_alg(&phase1_solution).unwrap(),
                    )
                else {
                    return Err(SearchError {
                        description: "Could not convert pattern into phase 2 coordinate".to_owned(),
                    });
                };
                let phase2_solution = self
                    .phase2_iterative_deepening_search
                    .search(
                        &phase2_start_pattern,
                        IndividualSearchOptions {
                            min_num_solutions: Some(1),
                            // TODO: we need to solve phase transition for 4x4x4, that will cause
                            // us to revisit this code.
                            // max_depth: Some(Depth(min(31 - phase1_solution.nodes.len(), 17))),
                            max_depth: Some(Depth(17)),
                            ..Default::default()
                        },
                    )
                    .next();

                // phase2_cumulative_time += Instant::now() - phase2_start_time;
                // let cumulative_time = Instant::now() - start_time;

                if let Some(mut phase2_solution) = phase2_solution {
                    let mut nodes = phase1_solution.nodes;
                    nodes.append(&mut phase2_solution.nodes);
                    debug_print_phase1_solutions_searched(found_phase1_solutions, current_depth);
                    return Ok(Alg { nodes });
                }

                // if num_phase2_starts % 100 == 0 {
                //     eprintln!(
                //         "\n{} phase 2 starts so far, {:?} in phase 1, {:?} in phase 2, {:?} in phase transition\n",
                //         num_phase2_starts,
                //         phase1_cumulative_time,
                //         phase2_cumulative_time,
                //         cumulative_time - phase1_cumulative_time - phase2_cumulative_time,
                //     )
                // }
            }
            // phase1_start_time = Instant::now();
            if found_phase1_solutions > 0 {
                debug_print_phase1_solutions_searched(found_phase1_solutions, current_depth);
            }
        }
        panic!("at the (lack of) disco(very)")
    }

    fn collapse_inverted_alg(&mut self, alg: cubing::alg::Alg) -> cubing::alg::Alg {
        group_square_1_tuples(alg)
    }
}

impl Square1ScrambleFinder {
    pub fn get_kpuzzle() -> &'static KPuzzle {
        square1_unbandaged_kpuzzle()
    }
}
