use crate::_internal::puzzle_traits::SemiGroupActionPuzzle;

pub trait PruneTable<TPuzzle: SemiGroupActionPuzzle> {
    fn lookup(&self, coordinates: &TPuzzle::Pattern) -> usize;

    // TODO
    fn extend_for_search_depth(&mut self, search_depth: usize, approximate_num_entries: usize);
}
