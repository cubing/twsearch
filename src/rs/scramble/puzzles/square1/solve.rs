use std::panic;
use std::process::exit;
use std::time::{Duration, Instant};

use std::cmp::min;

use cubing::alg::parse_alg;
use cubing::kpuzzle::KPuzzle;
use cubing::{
    alg::{parse_move, Alg, AlgBuilder, AlgNode, Grouping, Move},
    kpuzzle::KPattern,
};
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle;
use crate::_internal::search::check_pattern;
use crate::_internal::search::coordinates::phase_coordinate_puzzle::PhaseCoordinatePuzzle;
use crate::_internal::search::pattern_stack::PatternStack;
use crate::_internal::search::idf_search::IDFSearchConstructionOptions;
use crate::{
    _internal::{
        cli::args::VerbosityLevel,
        errors::SearchError,
        search::{
            idf_search::{IDFSearch, IndividualSearchOptions},
            prune_table_trait::Depth,
            search_logger::SearchLogger,
        },
    },
    scramble::{
        puzzles::square1::phase1::Square1Phase1Puzzle, scramble_search::move_list_from_vec,
    },
};

use super::super::definitions::square1_unbandaged_kpuzzle;
use super::phase1::Square1Phase1Coordinate;
use super::phase2::Square1Phase2Puzzle;

pub(crate) struct Square1Solver {
    square1_phase1_puzzle: Square1Phase1Puzzle,
    phase1_idfs: IDFSearch<Square1Phase1Puzzle>,
    square1_phase2_puzzle: Square1Phase2Puzzle,
    phase2_idfs: IDFSearch<Square1Phase2Puzzle>,
}

impl Square1Solver {
    pub(crate) fn new() -> Self {
        let kpuzzle = square1_unbandaged_kpuzzle();
        let generator_moves = move_list_from_vec(vec!["U_SQ_", "D_SQ_", "/"]);

        let square1_phase1_puzzle: PhaseCoordinatePuzzle<KPuzzle, Square1Phase1Coordinate> =
            Square1Phase1Puzzle::new(
                kpuzzle.clone(),
                kpuzzle.default_pattern(),
                generator_moves.clone(),
            );

        let phase1_target_pattern =
            square1_phase1_puzzle.full_pattern_to_phase_coordinate(&kpuzzle.default_pattern()).unwrap();

        let phase1_idfs = IDFSearch::<Square1Phase1Puzzle>::try_new(
            square1_phase1_puzzle.clone(),
            generator_moves.clone(),
            phase1_target_pattern,
        IDFSearchConstructionOptions {
            search_logger: SearchLogger {
                verbosity: VerbosityLevel::Info,
            }
            .into(),
            ..Default::default()
        },
        )
        .unwrap();

        let square1_phase2_puzzle: Square1Phase2Puzzle = Square1Phase2Puzzle::new(
            kpuzzle.clone(),
            kpuzzle.default_pattern(),
            generator_moves.clone(),
        );
        let phase2_target_pattern =
            square1_phase2_puzzle.full_pattern_to_phase_coordinate(&kpuzzle.default_pattern()).unwrap();

        let phase2_idfs = IDFSearch::<Square1Phase2Puzzle>::try_new(
            square1_phase2_puzzle.clone(),
            generator_moves.clone(),
            phase2_target_pattern,
   IDFSearchConstructionOptions {
                search_logger: (SearchLogger {
                        // <<< verbosity: VerbosityLevel::Warning,
                        verbosity: VerbosityLevel::Info, //<<<
                    }).into(),
                    ..Default::default()
            },
        )
        .unwrap();
        Self {
            square1_phase1_puzzle,
            phase1_idfs,
            square1_phase2_puzzle,
            phase2_idfs,
        }
    }

    pub(crate) fn solve_square1(&mut self, pattern: &KPattern) -> Result<Alg, SearchError> {
        let Ok(phase1_start_pattern) = self
            .square1_phase1_puzzle
            .full_pattern_to_phase_coordinate(pattern) else {
            return Err(SearchError{ description: "invalid pattern".to_owned() });
        };

        // let start_time = Instant::now();
        // let mut last_solution: Alg = parse_alg!("/");
        let start_time = Instant::now();
        let mut phase1_start_time = Instant::now();
        let num_solutions = 100;
        let phase1_search = self.phase1_idfs.search(
            &phase1_start_pattern,
            IndividualSearchOptions {
                min_num_solutions: Some(num_solutions),
                ..Default::default()
            },
        );
        // for (i, solution) in phase1_search.enumerate() {
        //     if (i + 1) % (num_solutions / 10) == 0 {
        //         eprintln!(
        //             "// Phase 1 solution #{}
        // {}
        // ",
        //             i + 1,
        //             solution
        //         )
        //     }
        //     last_solution = solution;
        // }
        // eprintln!(
        //     "Elapsed time to find {} solutions for phase 1 test: {:?}
        // ",
        //     num_solutions,
        //     Instant::now() - start_time
        // );

        // todo!();

        eprintln!("PHASE1ING");

        let mut num_phase2_starts = 0;
        let mut phase1_cumulative_time = Duration::default();
        let mut phase2_cumulative_time = Duration::default();
        #[allow(non_snake_case)]
        let _SLASH_ = parse_move!("/");
        'phase1_loop: for mut phase1_solution in phase1_search {
            phase1_cumulative_time += Instant::now() - phase1_start_time;
            eprintln!("alg: {:?}", &phase1_solution.to_string());

            // TODO: move below the while loop
            // <<< self.sanity_checker(pattern.apply_alg(&phase1_solution).unwrap());
            let Ok(phase2_start_pattern) = self.square1_phase2_puzzle
                .full_pattern_to_phase_coordinate(&pattern.apply_alg(&phase1_solution).unwrap())
            else {
                return Err(SearchError {
                    description: "Could not convert pattern into phase 2 coordinate".to_owned(),
                });
            };


            let edges_coord = self.square1_phase2_puzzle.data.puzzle1.data.index_to_semantic_coordinate.at(phase2_start_pattern.coordinate1);
            let corners_coord = self.square1_phase2_puzzle.data.puzzle2.data.index_to_semantic_coordinate.at(phase2_start_pattern.coordinate2);
            let equator_coord = self.square1_phase2_puzzle.data.puzzle3.data.index_to_semantic_coordinate.at(phase2_start_pattern.coordinate3);
            let edges = edges_coord.edges.clone();
            let corners = corners_coord.corners.clone();
            let equator = equator_coord.equator.clone();
            let mut full_pattern = edges.clone();
            for i in 0..24 {
                unsafe {
                    let wedge_orbit_info = &pattern.kpuzzle().data.ordered_orbit_info[0];
                    assert_eq!(wedge_orbit_info.name.0, "WEDGES");

                    if 0 == edges.packed_orbit_data().get_raw_piece_or_permutation_value(wedge_orbit_info, i) {
                        let corner_value = corners.packed_orbit_data().get_raw_piece_or_permutation_value(wedge_orbit_info, i);
                        full_pattern.packed_orbit_data_mut().set_raw_piece_or_permutation_value(wedge_orbit_info, i, corner_value);
                    }
                }
            }
            let equator_orbit_info = &pattern.kpuzzle().data.ordered_orbit_info[1];
            assert_eq!(equator_orbit_info.name.0, "EQUATOR");
            for i in 0..equator_orbit_info.num_pieces {
                let value = equator.get_piece(equator_orbit_info, i);
                full_pattern.set_piece(equator_orbit_info, i, value);

                let value = equator.get_orientation_with_mod(equator_orbit_info, i);
                full_pattern.set_orientation_with_mod(equator_orbit_info, i, value);
            }

            let alt_pattern = &pattern.apply_alg(&phase1_solution).unwrap();

            if *alt_pattern == full_pattern {
                println!("They match!");
            } else {
                println!("They do not match!");
                dbg!(full_pattern);
                dbg!(alt_pattern);
            }

            // <<< dbg!(full_pattern);
            // <<< dbg!(corners);
            // <<< dbg!(equator);
            // <<< dbg!(&pattern.apply_alg(&phase1_solution).unwrap());

            eprintln!("coordinates: {:?}", phase2_start_pattern); //<<<

            // TODO: Push the candidate check into a trait for `IDFSearch`.
            while let Some(cubing::alg::AlgNode::MoveNode(r#move)) = phase1_solution.nodes.last() {
                if r#move == &_SLASH_
                // TODO: redundant parsing
                {
                    break;
                }
                // Discard equivalent phase 1 solutions (reduces redundant phase 2 searches by a factor of 16).
                if r#move.amount > 2 || r#move.amount < 0 {
                    phase1_start_time = Instant::now();
                    continue 'phase1_loop;
                }
                phase1_solution.nodes.pop();
            }

            num_phase2_starts += 1;
            // eprintln!("\n{}", phase1_solution);
            // eprintln!("\nSearching for a phase2 solution");
            let phase2_start_time = Instant::now();
            let phase2_solution = self
                .phase2_idfs
                .search(
                    &phase2_start_pattern,
                    IndividualSearchOptions {
                        min_num_solutions: Some(1),
                        // <<< max_depth: Some(Depth(min(31 - phase1_solution.nodes.len(), 17))),
                        max_depth: None, //<<<
                        ..Default::default()
                    },
                )
                .next();

            phase2_cumulative_time += Instant::now() - phase2_start_time;
            let cumulative_time = Instant::now() - start_time;

            if let Some(mut phase2_solution) = phase2_solution {
                let mut nodes = phase1_solution.nodes;
                nodes.append(&mut phase2_solution.nodes);
                dbg!(&phase1_start_pattern);

            return Ok(group_square_1_tuples(Alg { nodes }.invert()));
            }

            if num_phase2_starts % 100 == 0 {
                eprintln!(
                    "\n{} phase 2 starts so far, {:?} in phase 1, {:?} in phase 2, {:?} in phase transition\n",
                    num_phase2_starts,
                    phase1_cumulative_time,
                    phase2_cumulative_time,
                    cumulative_time - phase1_cumulative_time - phase2_cumulative_time,
                )
            }

            phase1_start_time = Instant::now();
        }

        panic!("at the (lack of) disco(very)")
    }

    fn sanity_checker(&mut self, mut pattern: KPattern) {
        let mut coordinate = self
            .square1_phase2_puzzle
            .full_pattern_to_phase_coordinate(&pattern).unwrap();

        let mut pattern_stack = PatternStack::new(self.square1_phase2_puzzle.clone(), coordinate.clone());

        for _ in 1..100000 {
            let info = self.phase2_idfs.api_data.search_generators.flat.0.choose(&mut thread_rng()).unwrap();

            let success = pattern_stack.push(&info.transformation);

            let next_pattern = pattern.apply_move(&info.r#move).unwrap();
            let Ok(next_pattern_as_coordinate) = self
                    .square1_phase2_puzzle
                    .full_pattern_to_phase_coordinate(&next_pattern) else {
                assert!(!success);
                continue;
            };

            assert!(success);

            let next_coordinate = self.square1_phase2_puzzle.pattern_apply_transformation(&coordinate, &info.transformation).unwrap();

            // <<< dbg!(&next_pattern_as_coordinate);
            // <<< dbg!(&next_coordinate);
            if next_coordinate != next_pattern_as_coordinate {
                panic!("\t\t\t\t*************** LOKKEEE HEEREEEEEEEEEEEE 1 ***************");
            }
            let stack_coordinate = pattern_stack.current_pattern();
            if next_coordinate != *stack_coordinate {
                panic!("\t\t\t\t*************** LOKKEEE HEEREEEEEEEEEEEE 2 ***************");
            }

            pattern = next_pattern;
            coordinate = next_coordinate;
            // <<< if next_pattern_as_coordinate != next_coordinate {
            // <<<     eprintln!("asdf");
            // <<< }
    
            // <<< eprintln!("{:?} -- {} --> {:?}", coordinate, info.r#move, next_coordinate);
        }
        println!("current_idx: {}", pattern_stack.current_idx);//<<<
        if pattern_stack.current_idx < 100 {
            panic!("\t\t\t\t*************** LOKKEEE HEEREEEEEEEEEEEE 3 ***************");
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
