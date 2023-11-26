use std::sync::Arc;

use cubing::{
    alg::{Alg, Move},
    kpuzzle::{KPattern, KPuzzle},
};

use crate::_internal::{
    options::{CustomGenerators, VerbosityLevel},
    options::{Generators, MetricEnum},
    IDFSearch, IndividualSearchOptions, SearchLogger,
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

pub(crate) fn idfs_with_target_pattern(
    kpuzzle: &KPuzzle,
    generators: Generators,
    target_pattern: KPattern,
    min_size: Option<usize>,
) -> IDFSearch<KPuzzle> {
    IDFSearch::try_new(
        kpuzzle.clone(),
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

pub(crate) fn basic_idfs(
    kpuzzle: &KPuzzle,
    generators: Generators,
    min_size: Option<usize>,
) -> IDFSearch<KPuzzle> {
    idfs_with_target_pattern(kpuzzle, generators, kpuzzle.default_pattern(), min_size)
}

pub(crate) fn filtered_search(
    scramble_pattern: &KPattern,
    generators: Generators,
    min_optimal_moves: Option<usize>,
    min_scramble_moves: Option<usize>,
) -> Option<Alg> {
    let mut idfs = basic_idfs(scramble_pattern.kpuzzle(), generators, None);
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
