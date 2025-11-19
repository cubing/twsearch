use crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle;

use super::prune_table_trait::{Depth, PruneTable};

pub(crate) struct BlankPruneTable {}

impl<TPuzzle: SemiGroupActionPuzzle> PruneTable<TPuzzle> for BlankPruneTable {
    fn lookup(&self, _pattern: &TPuzzle::Pattern) -> Depth {
        Depth(0)
    }

    fn extend_for_search_depth(
        &mut self,
        _search_depth: super::prune_table_trait::Depth,
        _approximate_num_entries: usize,
    ) {
        // no-op
    }
}
