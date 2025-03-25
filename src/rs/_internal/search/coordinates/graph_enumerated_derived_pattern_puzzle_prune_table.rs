use std::{hash::Hash, marker::PhantomData};

use crate::_internal::{
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::prune_table_trait::{Depth, PruneTable},
};

use super::{
    graph_enumerated_derived_pattern_puzzle::GraphEnumeratedDerivedPatternPuzzle,
    pattern_deriver::PatternDeriver,
};

pub struct GraphEnumeratedDerivedPatternPuzzlePruneTable<
    TSourcePuzzle: SemiGroupActionPuzzle,
    TPatternDeriver: PatternDeriver<TSourcePuzzle>,
> where
    TPatternDeriver::DerivedPattern: Hash,
{
    // TODO: store just the prune table here
    puzzle: GraphEnumeratedDerivedPatternPuzzle<TSourcePuzzle, TPatternDeriver>,
    phantom_data: PhantomData<TPatternDeriver>,
}

impl<TSourcePuzzle: SemiGroupActionPuzzle, TPatternDeriver: PatternDeriver<TSourcePuzzle>>
    GraphEnumeratedDerivedPatternPuzzlePruneTable<TSourcePuzzle, TPatternDeriver>
where
    TPatternDeriver::DerivedPattern: Hash,
{
    pub fn new(
        puzzle: GraphEnumeratedDerivedPatternPuzzle<TSourcePuzzle, TPatternDeriver>,
    ) -> Self {
        Self {
            puzzle,
            phantom_data: PhantomData,
        }
    }
}

impl<TSourcePuzzle: SemiGroupActionPuzzle, TPatternDeriver: PatternDeriver<TSourcePuzzle>>
    PruneTable<GraphEnumeratedDerivedPatternPuzzle<TSourcePuzzle, TPatternDeriver>>
    for GraphEnumeratedDerivedPatternPuzzlePruneTable<TSourcePuzzle, TPatternDeriver>
where
    TPatternDeriver::DerivedPattern: Hash,
{
    fn lookup(
        &self,
        pattern: &<GraphEnumeratedDerivedPatternPuzzle<TSourcePuzzle, TPatternDeriver> as SemiGroupActionPuzzle>::Pattern,
    ) -> Depth {
        self.puzzle.data.exact_prune_table[*pattern]
    }

    fn extend_for_search_depth(&mut self, _search_depth: Depth, _approximate_num_entries: usize) {
        // no-op
    }
}
