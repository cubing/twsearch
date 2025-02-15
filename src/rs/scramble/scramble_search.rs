use std::{default::Default, marker::PhantomData};

use cubing::{
    alg::{Alg, Move},
    kpuzzle::{KPattern, KPuzzle},
};

use crate::_internal::{
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::{
        idf_search::{
            idf_search::{IDFSearch, IndividualSearchOptions},
            search_adaptations::{DefaultSearchAdaptations, SearchAdaptations},
        },
        move_count::MoveCount,
        prune_table_trait::Depth,
    },
};

pub fn move_list_from_vec(move_str_list: Vec<&str>) -> Vec<Move> {
    move_str_list
        .iter()
        .map(|move_str| move_str.parse::<Move>().unwrap())
        .collect()
}

pub struct FilteredSearch<
    TPuzzle: SemiGroupActionPuzzle + DefaultSearchAdaptations<TPuzzle> = KPuzzle,
    Adaptations: SearchAdaptations<TPuzzle> = <TPuzzle as DefaultSearchAdaptations<
        TPuzzle,
    >>::Adaptations,
> {
    pub(crate) idfs: IDFSearch<TPuzzle, Adaptations>,

    phantom_data: PhantomData<(TPuzzle, Adaptations)>,
}

impl<
        TPuzzle: SemiGroupActionPuzzle + DefaultSearchAdaptations<TPuzzle>,
        Adaptations: SearchAdaptations<TPuzzle>,
    > FilteredSearch<TPuzzle, Adaptations>
{
    pub fn new(idfs: IDFSearch<TPuzzle, Adaptations>) -> Self {
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
                    max_depth: Some(Depth(min_optimal_moves.0)),
                    ..Default::default()
                },
            )
            .next()
    }

    /// This function depends on the caller to pass parameters that will always result in an alg.
    pub fn generate_scramble(
        &mut self,
        scramble_pattern: &TPuzzle::Pattern,
        min_scramble_moves: Option<MoveCount>,
    ) -> Alg {
        self.idfs
            .search(
                scramble_pattern,
                IndividualSearchOptions {
                    min_num_solutions: Some(1),
                    min_depth: min_scramble_moves.map(|move_count| Depth(move_count.0)),
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
    generator_moves: Vec<Move>,
    min_optimal_moves: MoveCount,
    min_scramble_moves: Option<MoveCount>,
) -> Option<Alg> {
    let kpuzzle = scramble_pattern.kpuzzle();
    let mut filtered_search = <FilteredSearch>::new(
        IDFSearch::try_new(
            kpuzzle.clone(),
            generator_moves,
            kpuzzle.default_pattern(),
            Default::default(),
        )
        .unwrap(),
    );
    if filtered_search
        .filter(scramble_pattern, min_optimal_moves)
        .is_some()
    {
        return None;
    }
    Some(filtered_search.generate_scramble(scramble_pattern, min_scramble_moves))
}
