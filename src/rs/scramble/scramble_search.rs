use std::{default::Default, marker::PhantomData, sync::Arc};

use cubing::{
    alg::{Alg, Move},
    kpuzzle::{KPattern, KPuzzle},
};

use crate::_internal::{
    options::{MetricEnum, VerbosityLevel},
    puzzle_traits::SemiGroupActionPuzzle,
    DefaultSearchOptimizations, Depth, IDFSearch, IndividualSearchOptions, MoveCount, SearchLogger,
    SearchOptimizations, SearchSolutions,
};

pub fn move_list_from_vec(move_str_list: Vec<&str>) -> Vec<Move> {
    move_str_list
        .iter()
        .map(|move_str| move_str.parse::<Move>().unwrap())
        .collect()
}

pub struct FilteredSearch<
    TPuzzle: SemiGroupActionPuzzle + DefaultSearchOptimizations<TPuzzle> = KPuzzle,
    Optimizations: SearchOptimizations<TPuzzle> = <TPuzzle as DefaultSearchOptimizations<
        TPuzzle,
    >>::Optimizations,
> {
    pub(crate) idfs: IDFSearch<TPuzzle, Optimizations>,

    phantom_data: PhantomData<(TPuzzle, Optimizations)>,
}

impl<
        TPuzzle: SemiGroupActionPuzzle + DefaultSearchOptimizations<TPuzzle>,
        Optimizations: SearchOptimizations<TPuzzle>,
    > FilteredSearch<TPuzzle, Optimizations>
{
    pub(crate) fn idfs_with_target_pattern(
        puzzle: &TPuzzle,
        generator_moves: Vec<Move>,
        target_pattern: TPuzzle::Pattern,
        min_prune_table_size: Option<usize>,
    ) -> IDFSearch<TPuzzle, Optimizations> {
        IDFSearch::<TPuzzle, Optimizations>::try_new(
            puzzle.clone(),
            target_pattern,
            generator_moves,
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

    pub(crate) fn basic_idfs(
        puzzle: &TPuzzle,
        generator_moves: Vec<Move>,
        min_prune_table_size: Option<usize>,
        target_pattern: TPuzzle::Pattern,
    ) -> IDFSearch<TPuzzle, Optimizations> {
        Self::idfs_with_target_pattern(
            puzzle,
            generator_moves,
            target_pattern,
            min_prune_table_size,
        )
    }

    pub fn new(
        puzzle: &TPuzzle,
        generator_moves: Vec<Move>,
        min_prune_table_size: Option<usize>,
        target_pattern: TPuzzle::Pattern,
    ) -> Self {
        let idfs = Self::basic_idfs(
            puzzle,
            generator_moves,
            min_prune_table_size,
            target_pattern,
        );
        Self {
            idfs,
            phantom_data: PhantomData,
        }
    }

    pub fn filter(
        &mut self,
        scramble_pattern: &TPuzzle::Pattern,
        min_optimal_moves: MoveCount,
    ) -> Option<Alg> {
        if min_optimal_moves == MoveCount(0) {
            return None;
        }
        self.idfs
            .search(
                scramble_pattern,
                IndividualSearchOptions {
                    min_num_solutions: Some(1),
                    min_depth: Some(Depth(0)),
                    max_depth: Some(Depth(min_optimal_moves.0 - 1)),
                    ..Default::default()
                },
            )
            .next()
    }

    // This function depends on the caller to have passed parameters that will always result in an alg.
    pub fn generate_scramble(
        &mut self,
        scramble_pattern: &TPuzzle::Pattern,
        min_scramble_moves: Option<MoveCount>,
    ) -> Alg {
        self.search(
            scramble_pattern,
            Some(1),
            min_scramble_moves.map(|move_count| Depth(move_count.0)),
            None,
        )
        .next()
        .unwrap()
        .invert()
    }

    pub fn search(
        &mut self,
        scramble_pattern: &TPuzzle::Pattern,
        min_num_solutions: Option<usize>,
        min_depth: Option<Depth>,
        max_depth: Option<Depth>,
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
    generator_moves: Vec<Move>,
    min_optimal_moves: MoveCount,
    min_scramble_moves: Option<MoveCount>,
) -> Option<Alg> {
    let kpuzzle = scramble_pattern.kpuzzle();
    let mut filtered_search =
        <FilteredSearch>::new(kpuzzle, generator_moves, None, kpuzzle.default_pattern());
    if filtered_search
        .filter(scramble_pattern, min_optimal_moves)
        .is_some()
    {
        return None;
    }
    Some(filtered_search.generate_scramble(scramble_pattern, min_scramble_moves))
}
