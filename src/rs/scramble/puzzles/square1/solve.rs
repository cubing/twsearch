use std::time::{Duration, Instant};

use std::cmp::min;

use cubing::{
    alg::{parse_move, Alg, AlgBuilder, AlgNode, Grouping, Move},
    kpuzzle::KPattern,
};

use crate::{
    _internal::{
        cli::args::{MetricEnum, VerbosityLevel},
        errors::SearchError,
        search::{
            idf_search::{IDFSearch, IndividualSearchOptions},
            prune_table_trait::Depth,
            search_logger::SearchLogger,
        },
    },
    scramble::{
        puzzles::square1::{phase1::Square1Phase1Puzzle, phase2::Square1Phase2Puzzle},
        scramble_search::move_list_from_vec,
    },
};

use super::super::definitions::square1_unbandaged_kpuzzle;

pub(crate) fn solve_square1(pattern: &KPattern) -> Result<Alg, SearchError> {
    let kpuzzle = square1_unbandaged_kpuzzle();
    let generator_moves = move_list_from_vec(vec!["U_SQ_", "D_SQ_", "/"]);

    let square1_phase1_puzzle = Square1Phase1Puzzle::new(
        kpuzzle.clone(),
        kpuzzle.default_pattern(),
        generator_moves.clone(),
    );

    let Ok(phase1_start_pattern) = square1_phase1_puzzle.full_pattern_to_phase_coordinate(pattern)
    else {
        return Err(SearchError {
            description: "Could not convert pattern into phase 1 coordinate".to_owned(),
        });
    };
    let phase1_target_pattern = square1_phase1_puzzle
        .full_pattern_to_phase_coordinate(&kpuzzle.default_pattern())
        .unwrap();
    let mut phase1_idfs = IDFSearch::<Square1Phase1Puzzle>::try_new(
        square1_phase1_puzzle,
        phase1_target_pattern,
        generator_moves.clone(),
        SearchLogger {
            verbosity: VerbosityLevel::Info,
        }
        .into(),
        &MetricEnum::Hand,
        false,
        None,
    )
    .unwrap();

    // let start_time = Instant::now();
    // let mut last_solution: Alg = parse_alg!("/");
    let start_time = Instant::now();
    let mut phase1_start_time = Instant::now();
    let num_solutions = 10_000_000;
    let phase1_search = phase1_idfs.search(
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

    let square1_phase2_puzzle: Square1Phase2Puzzle = Square1Phase2Puzzle::new(
        kpuzzle.clone(),
        kpuzzle.default_pattern(),
        generator_moves.clone(),
    );
    let phase2_target_pattern = square1_phase2_puzzle
        .full_pattern_to_phase_coordinate(&kpuzzle.default_pattern())
        .unwrap();
    let mut phase2_idfs = IDFSearch::<Square1Phase2Puzzle>::try_new(
        square1_phase2_puzzle.clone(),
        phase2_target_pattern,
        generator_moves.clone(),
        SearchLogger {
            verbosity: VerbosityLevel::Warning,
        }
        .into(),
        &MetricEnum::Hand,
        false,
        None,
    )
    .unwrap();

    eprintln!("PHASE1ING");

    let mut num_phase2_starts = 0;
    let mut phase1_cumulative_time = Duration::default();
    let mut phase2_cumulative_time = Duration::default();
    #[allow(non_snake_case)]
    let _SLASH_ = parse_move!("/");
    'phase1_loop: for mut phase1_solution in phase1_search {
        phase1_cumulative_time += Instant::now() - phase1_start_time;

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

        let Ok(phase2_start_pattern) = square1_phase2_puzzle
            .full_pattern_to_phase_coordinate(&pattern.apply_alg(&phase1_solution).unwrap())
        else {
            return Err(SearchError {
                description: "Could not convert pattern into phase 2 coordinate".to_owned(),
            });
        };

        num_phase2_starts += 1;
        // eprintln!("\n{}", phase1_solution);
        // eprintln!("\nSearching for a phase2 solution");
        let phase2_start_time = Instant::now();
        let phase2_solution = phase2_idfs
            .search(
                &phase2_start_pattern,
                IndividualSearchOptions {
                    min_num_solutions: Some(1),
                    max_depth: Some(Depth(min(31 - phase1_solution.nodes.len(), 17))),
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
