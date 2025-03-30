use std::marker::PhantomData;

use crate::_internal::{
    errors::SearchError,
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::{
        coordinates::pattern_deriver::DerivedPuzzle,
        iterative_deepening::iterative_deepening_search::{
            IndividualSearchOptions, IterativeDeepeningSearch,
        },
    },
};

use super::SearchPhase;

pub(crate) struct DerivedPuzzleSearchPhase<
    TSourcePuzzle: SemiGroupActionPuzzle,
    TDerivedPuzzle: DerivedPuzzle<TSourcePuzzle>,
> {
    phase_name: String,
    derived_puzzle: TDerivedPuzzle,
    iterative_deepening_search: IterativeDeepeningSearch<TDerivedPuzzle>,
    phantom_data: PhantomData<(TSourcePuzzle, TDerivedPuzzle)>,
    individual_search_options: IndividualSearchOptions,
}

impl<TSourcePuzzle: SemiGroupActionPuzzle, TDerivedPuzzle: DerivedPuzzle<TSourcePuzzle>>
    DerivedPuzzleSearchPhase<TSourcePuzzle, TDerivedPuzzle>
{
    pub fn new(
        phase_name: String,
        derived_puzzle: TDerivedPuzzle,
        iterative_deepening_search: IterativeDeepeningSearch<TDerivedPuzzle>,
        individual_search_options: IndividualSearchOptions,
    ) -> Self {
        Self {
            phase_name,
            derived_puzzle,
            iterative_deepening_search,
            individual_search_options,
            phantom_data: PhantomData,
        }
    }
}

impl<
        TSourcePuzzle: SemiGroupActionPuzzle + Send + Sync,
        TDerivedPuzzle: DerivedPuzzle<TSourcePuzzle> + Send + Sync,
    > SearchPhase<TSourcePuzzle> for DerivedPuzzleSearchPhase<TSourcePuzzle, TDerivedPuzzle>
where
    TDerivedPuzzle::Pattern: Send + Sync,
    TDerivedPuzzle::Transformation: Send + Sync,
{
    fn phase_name(&self) -> &str {
        &self.phase_name
    }

    fn first_solution(
        &mut self,
        phase_search_pattern: &TSourcePuzzle::Pattern,
    ) -> Result<Option<cubing::alg::Alg>, crate::_internal::errors::SearchError> {
        let Some(search_pattern) = self.derived_puzzle.derive_pattern(phase_search_pattern) else {
            return Err(SearchError {
                description: "Could not derive pattern for search.".to_owned(),
            });
        };
        Ok(self
            .iterative_deepening_search
            .search_with_default_individual_search_adaptations(
                &search_pattern,
                self.individual_search_options.clone(),
            )
            .next())
    }
}
