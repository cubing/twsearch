use crate::{
    _internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    scramble::solving_based_scramble_finder::{
        FilteringDecision, NoScrambleAssociatedData, NoScrambleOptions, SolvingBasedScrambleFinder,
    },
};

use cubing::{alg::Alg, kpuzzle::KPattern};
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    _internal::search::{
        mask_pattern::apply_mask, move_count::MoveCount,
        pattern_traversal_filter_trait::PatternTraversalFilter,
    },
    scramble::{
        puzzles::square1::square1_shape_traversal_filter::Square1ShapeTraversalFilter,
        randomize::{
            randomize_orbit_naïve, ConstraintForFirstPiece, OrbitRandomizationConstraints,
        },
    },
};

use super::super::definitions::{square1_square_square_shape_kpattern, square1_unbandaged_kpuzzle};
use cubing::alg::{parse_move, AlgBuilder, AlgNode, Grouping, Move};
use cubing::kpuzzle::KPuzzle;

use crate::_internal::search::hash_prune_table::HashPruneTable;
use crate::_internal::search::iterative_deepening::search_adaptations::SearchAdaptations;
use crate::_internal::search::transformation_traversal_filter_trait::TransformationTraversalFilterNoOp;
use crate::scramble::scramble_search::FilteredSearch;
use crate::{
    _internal::search::{
        coordinates::phase_coordinate_puzzle::PhaseCoordinatePuzzle,
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

use super::phase1::{Square1Phase1Coordinate, Square1Phase1SearchAdaptations};
use super::phase2::{Square1Phase2Puzzle, Square1Phase2SearchAdaptations};

const DEV_DEBUG_SQUARE1: bool = false;

// https://www.worldcubeassociation.org/regulations/#4b3d
const SQUARE_1_SCRAMBLE_MIN_OPTIMAL_MOVE_COUNT: MoveCount = MoveCount(11);

pub(crate) struct FilteringSearchAdaptations {}

impl SearchAdaptations<KPuzzle> for FilteringSearchAdaptations {
    type PruneTable = HashPruneTable<KPuzzle, Square1ShapeTraversalFilter>;
    type PatternTraversalFilter = Square1ShapeTraversalFilter;
    type TransformationTraversalFilter = TransformationTraversalFilterNoOp;
}

pub(crate) struct Square1ScrambleFinder {
    square1_phase1_puzzle: Square1Phase1Puzzle,
    phase1_iterative_deepening_search:
        IterativeDeepeningSearch<Square1Phase1Puzzle, Square1Phase1SearchAdaptations>,
    square1_phase2_puzzle: Square1Phase2Puzzle,
    phase2_iterative_deepening_search:
        IterativeDeepeningSearch<Square1Phase2Puzzle, Square1Phase2SearchAdaptations>,
    // TODO: lazy-initialize `depth_filtering_search`?
    depth_filtering_search: FilteredSearch<KPuzzle, FilteringSearchAdaptations>,
}

impl Default for Square1ScrambleFinder {
    fn default() -> Self {
        let kpuzzle = square1_unbandaged_kpuzzle();
        let generator_moves = move_list_from_vec(vec!["U_SQ_", "D_SQ_", "/"]);

        let square1_phase1_puzzle: PhaseCoordinatePuzzle<KPuzzle, Square1Phase1Coordinate> =
            Square1Phase1Puzzle::new(
                kpuzzle.clone(),
                kpuzzle.default_pattern(),
                generator_moves.clone(),
            );

        let phase1_target_pattern = square1_phase1_puzzle
            .full_pattern_to_phase_coordinate(&kpuzzle.default_pattern())
            .unwrap();

        let phase1_iterative_deepening_search = IterativeDeepeningSearch::<
            Square1Phase1Puzzle,
            Square1Phase1SearchAdaptations,
        >::try_new(
            square1_phase1_puzzle.clone(),
            generator_moves.clone(),
            phase1_target_pattern,
            IterativeDeepeningSearchConstructionOptions {
                ..Default::default()
            },
        )
        .unwrap();

        let square1_phase2_puzzle: Square1Phase2Puzzle = Square1Phase2Puzzle::new();
        let phase2_target_pattern = square1_phase2_puzzle
            .full_pattern_to_phase_coordinate(&kpuzzle.default_pattern())
            .unwrap();

        let phase2_iterative_deepening_search = IterativeDeepeningSearch::<
            Square1Phase2Puzzle,
            Square1Phase2SearchAdaptations,
        >::try_new(
            square1_phase2_puzzle.clone(),
            generator_moves.clone(),
            phase2_target_pattern,
            IterativeDeepeningSearchConstructionOptions {
                ..Default::default()
            },
        )
        .unwrap();

        let depth_filtering_search = {
            let kpuzzle = square1_unbandaged_kpuzzle();
            let generator_moves = move_list_from_vec(vec!["U_SQ_", "D_SQ_", "/"]);

            let iterative_deepening_search =
                IterativeDeepeningSearch::<KPuzzle, FilteringSearchAdaptations>::try_new(
                    kpuzzle.clone(),
                    generator_moves,
                    kpuzzle.default_pattern(),
                    Default::default(),
                )
                .unwrap();
            FilteredSearch::<KPuzzle, FilteringSearchAdaptations>::new(iterative_deepening_search)
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
            if current_tuple_U.is_some() {
                panic!("Invalid Square-1 scramble alg in internal code.");
            } else {
                current_tuple_U = Some(r#move);
            }
        } else if r#move.quantum == D_SQ_0.quantum {
            if current_tuple_D.is_some() {
                panic!("Invalid Square-1 scramble alg in internal code.");
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

/// An empty trait that can implemented by traits to indicate that they are a
/// Square-1 search phase (rather than just a generic
/// [`SemiGroupActionPuzzle`]).
pub trait Square1SearchPhase: SemiGroupActionPuzzle {}

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
            if Square1ShapeTraversalFilter::is_valid(&phase1_start_pattern) {
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
            .full_pattern_to_phase_coordinate(pattern)
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
            let mut checked_phase1_solutions = 0;
            'phase1_loop: for mut phase1_solution in phase1_search {
                found_phase1_solutions += 1;
                // phase1_cumulative_time += Instant::now() - phase1_start_time;
                // TODO: Push the candidate check into a trait for `IterativeDeepeningSearch`.
                while let Some(cubing::alg::AlgNode::MoveNode(r#move)) =
                    phase1_solution.nodes.last()
                {
                    if r#move == _SLASH_
                    // TODO: redundant parsing
                    {
                        break;
                    }
                    // Discard equivalent phase 1 solutions (reduces redundant phase 2 searches by a factor of 16).
                    if r#move.amount > 2 || r#move.amount < 0 {
                        // phase1_start_time = Instant::now();
                        continue 'phase1_loop;
                    }
                    phase1_solution.nodes.pop();
                }
                checked_phase1_solutions += 1;
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
                    if DEV_DEBUG_SQUARE1 {
                        println!(
                            "-- depth {} returned sols {} checked sols {}",
                            current_depth, found_phase1_solutions, checked_phase1_solutions
                        );
                    }
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
            if DEV_DEBUG_SQUARE1 && found_phase1_solutions > 0 {
                println!(
                    "At depth {} returned sols {} checked sols {}",
                    current_depth, found_phase1_solutions, checked_phase1_solutions
                );
            }
        }
        panic!("at the (lack of) disco(very)")
    }

    fn collapse_inverted_alg(&mut self, alg: cubing::alg::Alg) -> cubing::alg::Alg {
        group_square_1_tuples(alg)
    }
}
