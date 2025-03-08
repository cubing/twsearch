use std::{default::Default, marker::PhantomData};

use cubing::{
    alg::{Alg, Move},
    kpuzzle::KPuzzle,
};

use crate::_internal::{
    errors::SearchError,
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

use super::solving_based_scramble_finder::FilteringDecision;

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

    pub fn filtering_decision(
        &mut self,
        scramble_pattern: &TPuzzle::Pattern,
        min_optimal_moves: MoveCount,
    ) -> FilteringDecision {
        match self.filter(scramble_pattern, min_optimal_moves) {
            Some(_) => FilteringDecision::Reject,
            None => FilteringDecision::Accept,
        }
    }

    /// This function depends on the caller to pass parameters that will always result in an alg.
    pub fn solve(
        &mut self,
        scramble_pattern: &TPuzzle::Pattern,
        min_scramble_moves: Option<MoveCount>,
    ) -> Option<Alg> {
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
    }

    /// This function depends on the caller to pass parameters that will always result in an alg.
    pub fn solve_or_error(
        &mut self,
        scramble_pattern: &TPuzzle::Pattern,
        min_scramble_moves: Option<MoveCount>,
    ) -> Result<Alg, SearchError> {
        let Some(alg) = self.solve(scramble_pattern, min_scramble_moves) else {
            return Err(SearchError {
                description: "Could not solve pattern".to_owned(),
            });
        };
        Ok(alg)
    }

    /// This function depends on the caller to pass parameters that will always result in an alg.
    pub fn generate_scramble(
        &mut self,
        scramble_pattern: &TPuzzle::Pattern,
        min_scramble_moves: Option<MoveCount>,
    ) -> Alg {
        self.solve(scramble_pattern, min_scramble_moves)
            .unwrap()
            .invert()
    }
}
