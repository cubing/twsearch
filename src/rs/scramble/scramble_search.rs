use std::sync::Arc;

use cubing::alg::Alg;

use crate::_internal::{
    Generators, IDFSearch, IndividualSearchOptions, PackedKPattern, SearchLogger,
};

pub(crate) fn scramble_search(
    scramble_pattern: &PackedKPattern,
    generators: Generators,
    min_optimal_moves: usize,
    min_scramble_moves: usize,
) -> Option<Alg> {
    let packed_kpuzzle = &scramble_pattern.packed_orbit_data.packed_kpuzzle;
    let mut idfs = IDFSearch::try_new(
        packed_kpuzzle.clone(),
        packed_kpuzzle.default_pattern(),
        generators,
        Arc::new(SearchLogger {
            verbosity: crate::_internal::VerbosityLevel::Error,
        }),
        &crate::_internal::MetricEnum::Hand,
        true,
    )
    .unwrap();
    // Scramble filtering by rejection sampling : Too close to solved?
    // https://www.worldcubeassociation.org/regulations/#4b3b
    // https://github.com/thewca/tnoodle/blob/master/webscrambles/src/main/resources/wca/readme-scramble.md#scramble-length
    if idfs
        .search(
            scramble_pattern,
            IndividualSearchOptions {
                min_num_solutions: Some(1),
                min_depth: Some(0),
                max_depth: Some(min_optimal_moves - 1),
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
                min_depth: Some(min_scramble_moves),
                max_depth: None,
            },
        )
        .next()
        .unwrap()
        .invert(),
    )
}
