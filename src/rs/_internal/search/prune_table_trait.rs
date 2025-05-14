use crate::{_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle, whole_number_newtype};

whole_number_newtype!(Depth, usize);

pub trait PruneTable<TPuzzle: SemiGroupActionPuzzle> {
    fn lookup(&self, pattern: &TPuzzle::Pattern) -> Depth;

    // TODO: generalize to more powerful notions of "extend"?
    fn extend_for_search_depth(&mut self, search_depth: Depth, approximate_num_entries: usize);
}
