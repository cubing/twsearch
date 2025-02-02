// use std::time::{Duration, Instant};

use std::sync::Mutex;

use cubing::kpuzzle::KPuzzle;
use cubing::{
    alg::{parse_move, Alg, AlgBuilder, AlgNode, Grouping, Move},
    kpuzzle::KPattern,
};
use lazy_static::lazy_static;

use crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle;
use crate::_internal::search::hash_prune_table::HashPruneTable;
use crate::_internal::search::idf_search::search_adaptations::SearchAdaptations;
use crate::_internal::search::transformation_traversal_filter_trait::TransformationTraversalFilterNoOp;
use crate::scramble::scramble_search::FilteredSearch;
use crate::{
    _internal::search::{
        coordinates::phase_coordinate_puzzle::PhaseCoordinatePuzzle,
        idf_search::idf_search::IDFSearchConstructionOptions, prune_table_trait::Depth,
    },
    _internal::{
        errors::SearchError,
        search::idf_search::idf_search::{IDFSearch, IndividualSearchOptions},
    },
    scramble::{
        puzzles::square1::phase1::Square1Phase1Puzzle, scramble_search::move_list_from_vec,
    },
};

use super::super::definitions::square1_unbandaged_kpuzzle;
use super::phase1::{Square1Phase1Coordinate, Square1Phase1SearchAdaptations};
use super::phase2::{Square1Phase2Puzzle, Square1Phase2SearchAdaptations};
use super::square1_shape_traversal_filter::Square1ShapeTraversalFilter;

const DEV_DEBUG_SQUARE1: bool = false;

pub(crate) struct FilteringSearchAdaptations {}

impl SearchAdaptations<KPuzzle> for FilteringSearchAdaptations {
    type PruneTable = HashPruneTable<KPuzzle, Square1ShapeTraversalFilter>;
    type PatternTraversalFilter = Square1ShapeTraversalFilter;
    type TransformationTraversalFilter = TransformationTraversalFilterNoOp;
}

pub(crate) struct Square1Solver {
    square1_phase1_puzzle: Square1Phase1Puzzle,
    phase1_idfs: IDFSearch<Square1Phase1Puzzle, Square1Phase1SearchAdaptations>,
    square1_phase2_puzzle: Square1Phase2Puzzle,
    phase2_idfs: IDFSearch<Square1Phase2Puzzle, Square1Phase2SearchAdaptations>,
    pub(crate) depth_filtering_search: FilteredSearch<KPuzzle, FilteringSearchAdaptations>,
}

lazy_static! {
    pub(crate) static ref SQUARE1_SOLVER: Mutex<Square1Solver> = Mutex::new(Square1Solver::new());
}

impl Square1Solver {
    /// Usage: `Square1Solver::get_globally_shared().lock().unwrap()`
    /// Note that usage is exclusive, and no search can start until all previous onces finish.
    // TODO: make a better pattern for this.
    pub(crate) fn get_globally_shared() -> &'static SQUARE1_SOLVER {
        &SQUARE1_SOLVER
    }

    pub(crate) fn new() -> Self {
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

        let phase1_idfs =
            IDFSearch::<Square1Phase1Puzzle, Square1Phase1SearchAdaptations>::try_new(
                square1_phase1_puzzle.clone(),
                generator_moves.clone(),
                phase1_target_pattern,
                IDFSearchConstructionOptions {
                    ..Default::default()
                },
            )
            .unwrap();

        let square1_phase2_puzzle: Square1Phase2Puzzle = Square1Phase2Puzzle::new();
        let phase2_target_pattern = square1_phase2_puzzle
            .full_pattern_to_phase_coordinate(&kpuzzle.default_pattern())
            .unwrap();

        let phase2_idfs =
            IDFSearch::<Square1Phase2Puzzle, Square1Phase2SearchAdaptations>::try_new(
                square1_phase2_puzzle.clone(),
                generator_moves.clone(),
                phase2_target_pattern,
                IDFSearchConstructionOptions {
                    ..Default::default()
                },
            )
            .unwrap();

        let depth_filtering_search = {
            let kpuzzle = square1_unbandaged_kpuzzle();
            let generator_moves = move_list_from_vec(vec!["U_SQ_", "D_SQ_", "/"]);

            let idfs = IDFSearch::<KPuzzle, FilteringSearchAdaptations>::try_new(
                kpuzzle.clone(),
                generator_moves,
                kpuzzle.default_pattern(),
                Default::default(),
            )
            .unwrap();
            FilteredSearch::<KPuzzle, FilteringSearchAdaptations>::new(idfs)
        };

        Self {
            square1_phase1_puzzle,
            phase1_idfs,
            square1_phase2_puzzle,
            phase2_idfs,
            depth_filtering_search,
        }
    }

    pub(crate) fn solve_square1(&mut self, pattern: &KPattern) -> Result<Alg, SearchError> {
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
            let phase1_search = self.phase1_idfs.search(
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
                // TODO: Push the candidate check into a trait for `IDFSearch`.
                while let Some(cubing::alg::AlgNode::MoveNode(r#move)) =
                    phase1_solution.nodes.last()
                {
                    if r#move == &_SLASH_
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
                    .phase2_idfs
                    .search(
                        &phase2_start_pattern,
                        IndividualSearchOptions {
                            min_num_solutions: Some(1),
                            // max_depth: Some(Depth(min(31 - phase1_solution.nodes.len(), 17))), // TODO
                            max_depth: Some(Depth(17)), //<<<
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
                    return Ok(group_square_1_tuples(Alg { nodes }.invert()));
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
                            .unwrap_or_else(|| parse_move!("U_SQ_0")),
                    ),
                    cubing::alg::AlgNode::MoveNode(
                        current_tuple_D
                            .take()
                            .unwrap_or_else(|| parse_move!("D_SQ_0")),
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
    let U_SQ_0: Move = parse_move!("U_SQ_0");
    #[allow(non_snake_case)]
    let D_SQ_0 = parse_move!("D_SQ_0");
    #[allow(non_snake_case)]
    let _SLASH_ = parse_move!("/");

    // TODO: Push the candidate check into a trait for `IDFSearch`.
    for node in alg.nodes {
        let cubing::alg::AlgNode::MoveNode(r#move) = node else {
            panic!("Invalid Square-1 scramble alg in internal code.");
        };
        if r#move == _SLASH_ {
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
