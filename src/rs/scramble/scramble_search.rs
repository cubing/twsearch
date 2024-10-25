use std::{default::Default, sync::Arc};

use cubing::{
    alg::{Alg, Move},
    kpuzzle::{KPattern, KPuzzle},
};

use crate::_internal::{
    options::{CustomGenerators, Generators, MetricEnum, VerbosityLevel},
    AlwaysValid, IDFSearch, IndividualSearchOptions, PatternValidityChecker, SearchLogger,
    SearchSolutions,
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

pub(crate) fn idfs_with_target_pattern<ValidityChecker: PatternValidityChecker<KPuzzle>>(
    kpuzzle: &KPuzzle,
    generators: Generators,
    target_pattern: KPattern,
    min_prune_table_size: Option<usize>,
) -> IDFSearch<KPuzzle, ValidityChecker> {
    IDFSearch::try_new(
        kpuzzle.clone(),
        target_pattern,
        generators.enumerate_moves_for_kpuzzle(kpuzzle),
        Arc::new(SearchLogger {
            verbosity: VerbosityLevel::Silent,
            // verbosity: VerbosityLevel::Info, //<<<
        }),
        &MetricEnum::Hand,
        true,
        min_prune_table_size,
    )
    .unwrap()
}

pub(crate) fn basic_idfs<ValidityChecker: PatternValidityChecker<KPuzzle>>(
    kpuzzle: &KPuzzle,
    generators: Generators,
    min_prune_table_size: Option<usize>,
    target_pattern: KPattern,
) -> IDFSearch<KPuzzle, ValidityChecker> {
    idfs_with_target_pattern(kpuzzle, generators, target_pattern, min_prune_table_size)
}

pub struct FilteredSearch<ValidityChecker: PatternValidityChecker<KPuzzle> = AlwaysValid> {
    pub(crate) idfs: IDFSearch<KPuzzle, ValidityChecker>,
}

impl<ValidityChecker: PatternValidityChecker<KPuzzle>> FilteredSearch<ValidityChecker> {
    pub fn new(
        kpuzzle: &KPuzzle,
        generators: Generators,
        min_prune_table_size: Option<usize>,
        target_pattern: KPattern,
    ) -> FilteredSearch<ValidityChecker> {
        let idfs = basic_idfs(kpuzzle, generators, min_prune_table_size, target_pattern);
        Self { idfs }
    }

    pub fn filter(&mut self, scramble_pattern: &KPattern, min_optimal_moves: usize) -> Option<Alg> {
        if min_optimal_moves == 0 {
            return None;
        }
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

    pub fn search(
        &mut self,
        scramble_pattern: &KPattern,
        min_num_solutions: Option<usize>,
        min_depth: Option<usize>,
        max_depth: Option<usize>,
    ) -> SearchSolutions {
        self.idfs.search(
            scramble_pattern,
            IndividualSearchOptions {
                min_num_solutions,
                min_depth,
                max_depth,
                ..Default::default()
            },
        )
    }
}

pub(crate) fn simple_filtered_search(
    scramble_pattern: &KPattern,
    generators: Generators,
    min_optimal_moves: usize,
    min_scramble_moves: Option<usize>,
) -> Option<Alg> {
    let kpuzzle = scramble_pattern.kpuzzle();
    let mut filtered_search =
        <FilteredSearch>::new(kpuzzle, generators, None, kpuzzle.default_pattern());
    if filtered_search
        .filter(scramble_pattern, min_optimal_moves)
        .is_some()
    {
        return None;
    }
    Some(filtered_search.generate_scramble(scramble_pattern, min_scramble_moves))
}
