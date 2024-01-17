use std::sync::Arc;

use cubing::alg::{Alg, Move};

use crate::_internal::{
    options::{CustomGenerators, VerbosityLevel},
    options::{Generators, MetricEnum},
    GenericPuzzle, IDFSearch, IndividualSearchOptions, SearchLogger,
};

pub fn move_list_from_vec(move_str_list: Vec<&str>) -> Vec<Move> {
    move_str_list
        .iter()
        .map(|move_str| move_str.parse::<Move>().unwrap())
        .collect()
}

pub fn generators_from_vec_str(move_str_list: Vec<&str>) -> Generators {
    Generators::Custom(CustomGenerators {
        moves: move_list_from_vec(move_str_list),
        algs: vec![],
    })
}

pub(crate) fn idfs_with_target_pattern<TPuzzle: GenericPuzzle>(
    tpuzzle: &TPuzzle,
    generators: Generators,
    target_pattern: TPuzzle::Pattern,
    min_size: Option<usize>,
) -> IDFSearch<TPuzzle> {
    IDFSearch::try_new(
        tpuzzle.clone(),
        target_pattern,
        generators,
        Arc::new(SearchLogger {
            verbosity: VerbosityLevel::Info,
        }),
        &MetricEnum::Hand,
        true,
        min_size,
    )
    .unwrap()
}

pub(crate) fn basic_idfs<TPuzzle: GenericPuzzle>(
    puzzle: &TPuzzle,
    generators: Generators,
    min_size: Option<usize>,
) -> IDFSearch<TPuzzle> {
    idfs_with_target_pattern(
        puzzle,
        generators,
        puzzle.puzzle_default_pattern(),
        min_size,
    )
}

pub(crate) fn filtered_search<TPuzzle: GenericPuzzle>(
    puzzle: &TPuzzle,
    scramble_pattern: &TPuzzle::Pattern,
    generators: Generators,
    min_optimal_moves: Option<usize>,
    min_scramble_moves: Option<usize>,
) -> Option<Alg> {
    let mut idfs = basic_idfs(puzzle, generators, None);
    if idfs
        .search(
            scramble_pattern,
            IndividualSearchOptions {
                min_num_solutions: Some(1),
                min_depth: Some(0),
                max_depth: min_optimal_moves.map(|v| v - 1),
                disallowed_initial_quanta: None,
                disallowed_final_quanta: None,
            },
        )
        .next()
        .is_some()
    {
        return None;
    }
    Some(
        idfs.search(
            scramble_pattern,
            IndividualSearchOptions {
                min_num_solutions: Some(1),
                min_depth: min_scramble_moves,
                max_depth: None,
                disallowed_initial_quanta: None,
                disallowed_final_quanta: None,
            },
        )
        .next()
        .unwrap()
        .invert(),
    )
}
