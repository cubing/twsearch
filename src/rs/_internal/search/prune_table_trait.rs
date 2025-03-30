use std::sync::Arc;

use crate::{_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle, whole_number_newtype};

use super::{
    iterative_deepening::{
        iterative_deepening_search::IterativeDeepeningSearchAPIData,
        search_adaptations::StoredSearchAdaptationsWithoutPruneTable,
    },
    search_logger::SearchLogger,
};

whole_number_newtype!(Depth, usize);

pub trait PruneTable<TPuzzle: SemiGroupActionPuzzle> {
    fn lookup(&self, pattern: &TPuzzle::Pattern) -> Depth;

    // TODO: generalize to more powerful notions of "extend"?
    fn extend_for_search_depth(&mut self, search_depth: Depth, approximate_num_entries: usize);
}

pub trait LegacyConstructablePruneTable<TPuzzle: SemiGroupActionPuzzle>:
    PruneTable<TPuzzle>
{
    // TODO: design a proper API. The args here are currently inherited from `HashPruneTable`
    fn new(
        tpuzzle: TPuzzle,
        search_api_data: Arc<IterativeDeepeningSearchAPIData<TPuzzle>>,
        search_logger: Arc<SearchLogger>,
        min_size: Option<usize>,
        search_adaptations_without_prune_table: StoredSearchAdaptationsWithoutPruneTable<TPuzzle>,
    ) -> Self;
}
