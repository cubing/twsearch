use std::{default::Default, sync::Arc};

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
    min_prune_table_size: Option<usize>,
) -> IDFSearch {
    IDFSearch::try_new(
        kpuzzle.clone(),
        target_pattern,
        generators,
        Arc::new(SearchLogger {
            verbosity: VerbosityLevel::Silent,
        }),
        &MetricEnum::Hand,
        true,
        min_prune_table_size,
    )
    .unwrap()
}

pub(crate) fn basic_idfs(
    kpuzzle: &KPuzzle,
    generators: Generators,
    min_prune_table_size: Option<usize>,
) -> IDFSearch {
    idfs_with_target_pattern(
        kpuzzle,
        generators,
        kpuzzle.default_pattern(),
        min_prune_table_size,
    )
}

pub struct FilteredSearch {
    idfs: IDFSearch,
}

impl FilteredSearch {
    pub fn new(
        kpuzzle: &KPuzzle,
        generators: Generators,
        min_prune_table_size: Option<usize>,
    ) -> FilteredSearch {
        let idfs = basic_idfs(kpuzzle, generators, min_prune_table_size);
        Self { idfs }
    }

    pub fn filter(&mut self, scramble_pattern: &KPattern, min_optimal_moves: usize) -> Option<Alg> {
        self.idfs
            .search(
                scramble_pattern,
                IndividualSearchOptions {
                    min_num_solutions: Some(1),
                    min_depth: Some(0),
                    max_depth: Some(min_optimal_moves - 1),
                    ..Default::default()
                },
            )
            .next()
    }

    // This function depends on the caller to have passed parameters that will always result in an alg.
    pub fn generate_scramble(
        &mut self,
        scramble_pattern: &KPattern,
        min_scramble_moves: Option<usize>,
    ) -> Alg {
        self.idfs
            .search(
                scramble_pattern,
                IndividualSearchOptions {
                    min_num_solutions: Some(1),
                    min_depth: min_scramble_moves,
                    ..Default::default()
                },
            )
            .next()
            .unwrap()
            .invert()
    }
}

pub(crate) fn simple_filtered_search(
    scramble_pattern: &KPattern,
    generators: Generators,
    min_optimal_moves: usize,
    min_scramble_moves: Option<usize>,
) -> Option<Alg> {
    let mut filtered_search = FilteredSearch::new(scramble_pattern.kpuzzle(), generators, None);
    if filtered_search
        .filter(scramble_pattern, min_optimal_moves)
        .is_some()
    {
        return None;
    }
    Some(filtered_search.generate_scramble(scramble_pattern, min_scramble_moves))
}
