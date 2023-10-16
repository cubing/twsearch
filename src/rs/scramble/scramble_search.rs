use std::sync::Arc;

use cubing::alg::Alg;

use crate::_internal::{
    Generators, IDFSearch, IndividualSearchOptions, PackedKPattern, PackedKPuzzle, SearchLogger,
};

pub(crate) fn idfs_with_target_pattern(
    packed_kpuzzle: &PackedKPuzzle,
    generators: Generators,
    target_pattern: PackedKPattern,
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
    )
    .unwrap()
}

pub(crate) fn basic_idfs(packed_kpuzzle: &PackedKPuzzle, generators: Generators) -> IDFSearch {
    idfs_with_target_pattern(packed_kpuzzle, generators, packed_kpuzzle.default_pattern())
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
