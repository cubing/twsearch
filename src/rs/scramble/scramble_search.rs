use std::sync::Arc;

use cubing::alg::{Alg, Move};

use crate::_internal::{
    CustomGenerators, Generators, IDFSearch, IndividualSearchOptions, PackedKPattern,
    PackedKPuzzle, SearchLogger,
};

pub fn move_list_from_vec(move_str_list: Vec<&str>) -> Vec<Move> {
    move_str_list
        .iter()
        .map(|move_str| move_str.parse::<Move>().unwrap())
        .collect()
}

pub fn generators_from_vec_str(move_str_list: Vec<&str>) -> Generators {
    crate::_internal::Generators::Custom(CustomGenerators {
        moves: move_list_from_vec(move_str_list),
        algs: vec![],
    })
}

pub(crate) fn idfs_with_target_pattern(
    packed_kpuzzle: &PackedKPuzzle,
    generators: Generators,
    target_pattern: PackedKPattern,
    min_size: Option<usize>,
) -> IDFSearch {
    IDFSearch::try_new(
        packed_kpuzzle.clone(),
        target_pattern,
        generators,
        Arc::new(SearchLogger {
            verbosity: crate::_internal::VerbosityLevel::Silent,
        }),
        &crate::_internal::MetricEnum::Hand,
        true,
        min_size,
    )
    .unwrap()
}

pub(crate) fn basic_idfs(
    packed_kpuzzle: &PackedKPuzzle,
    generators: Generators,
    min_size: Option<usize>,
) -> IDFSearch {
    idfs_with_target_pattern(
        packed_kpuzzle,
        generators,
        packed_kpuzzle.default_pattern(),
        min_size,
    )
}

pub(crate) fn filtered_search(
    scramble_pattern: &PackedKPattern,
    generators: Generators,
    min_optimal_moves: Option<usize>,
    min_scramble_moves: Option<usize>,
) -> Option<Alg> {
    let mut idfs = basic_idfs(
        &scramble_pattern.packed_orbit_data.packed_kpuzzle,
        generators,
        None,
    );
    if idfs
        .search(
            scramble_pattern,
            IndividualSearchOptions {
                min_num_solutions: Some(1),
                min_depth: Some(0),
                max_depth: min_optimal_moves.map(|v| v - 1),
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
            },
        )
        .next()
        .unwrap()
        .invert(),
    )
}
