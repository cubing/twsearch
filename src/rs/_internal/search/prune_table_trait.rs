use std::sync::Arc;

use crate::{_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle, whole_number_newtype};

use super::iterative_deepening::{
    iterative_deepening_search::ImmutableSearchData, search_adaptations::StoredSearchAdaptations,
};

whole_number_newtype!(Depth, usize);

pub trait PruneTable<TPuzzle: SemiGroupActionPuzzle> {
    fn lookup(&self, pattern: &TPuzzle::Pattern) -> Depth;

    // TODO: generalize to more powerful notions of "extend"?
    fn extend_for_search_depth(&mut self, search_depth: Depth, approximate_num_entries: usize);
}

#[derive(Default)]
pub struct PruneTableSizeBounds {
    pub(crate) min_size: Option<usize>,
    pub(crate) max_size: Option<usize>,
}

pub trait LegacyConstructablePruneTable<TPuzzle: SemiGroupActionPuzzle>:
    PruneTable<TPuzzle>
{
    // TODO: design a proper API. The args here are currently inherited from `HashPruneTable`
    fn new(
        immutable_search_data: Arc<ImmutableSearchData<TPuzzle>>,
        stored_search_adaptations: StoredSearchAdaptations<TPuzzle>,
        size_bounds: PruneTableSizeBounds,
    ) -> Self;
}
