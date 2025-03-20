use std::marker::PhantomData;

use cubing::kpuzzle::KPuzzle;

use crate::_internal::{
    puzzle_traits::puzzle_traits::{HashablePatternPuzzle, SemiGroupActionPuzzle},
    search::filter::transformation_traversal_filter_trait::{
        TransformationTraversalFilter, TransformationTraversalFilterNoOp,
    },
};

use super::super::{
    filter::pattern_traversal_filter_trait::{PatternTraversalFilter, PatternTraversalFilterNoOp},
    hash_prune_table::HashPruneTable,
    prune_table_trait::PruneTable,
};

/// The [`SearchAdaptations`] trait bundles various traits that a search can invoke
/// to change its search behaviour.
///
/// Each associated type could theoretically be a type parameter on
/// [`IterativeDeepeningSearch`](super::iterative_deepening_search::IterativeDeepeningSearch) itself, but this would make
/// [`IterativeDeepeningSearch`](super::iterative_deepening_search::IterativeDeepeningSearch) types rather unwieldy. So
/// instead of this:
///
/// ```text
/// IterativeDeepeningSearch<TPuzzle: …, PatternTraversalFilter: …, PruneTable: …, (more in the future…)>
/// ```
///
/// we have this:
///
/// ```text
/// IterativeDeepeningSearch<TPuzzle: …, Adaptations: …>
/// ```
///
/// In addition, the [`DefaultSearchAdaptations`] trait can be implemented for
/// any given [`SemiGroupActionPuzzle`]. This allows us to avoid specifying the
/// `Adaptations` type parameter on [`IterativeDeepeningSearch`](super::iterative_deepening_search::IterativeDeepeningSearch)
/// in common situations. For example, the type for a [`KPuzzle`] search will
/// often just be:
///
/// ```ignore
/// IterativeDeepeningSearch<KPuzzle>
/// ```
///
/// Note: the main reason that these are traits is that it enables "zero-cost"
/// abstraction in code that is run tens of millions of times per second. If you
/// implement a trait with trivial code, then this code can be inlined in an
/// instantiated [`IterativeDeepeningSearch`](super::iterative_deepening_search::IterativeDeepeningSearch) without dynamic
/// dispatch.
///
/// TODO: figure out if/when dynamic dispatch is actually cheap and ergonomic
/// enough once we know all the adaptations we need for common puzzles.
pub trait SearchAdaptations<TPuzzle: SemiGroupActionPuzzle> {
    type PruneTable: PruneTable<TPuzzle>;
    type PatternTraversalFilter: PatternTraversalFilter<TPuzzle>;
    type TransformationTraversalFilter: TransformationTraversalFilter<TPuzzle>;
}

pub struct SearchAdaptationsHashPruneTableOnly<TPuzzle: HashablePatternPuzzle> {
    phantom_data: PhantomData<TPuzzle>,
}

impl<TPuzzle: HashablePatternPuzzle> SearchAdaptations<TPuzzle>
    for SearchAdaptationsHashPruneTableOnly<TPuzzle>
{
    type PatternTraversalFilter = PatternTraversalFilterNoOp;
    type PruneTable = HashPruneTable<TPuzzle, Self::PatternTraversalFilter>;
    type TransformationTraversalFilter = TransformationTraversalFilterNoOp;
}

pub trait DefaultSearchAdaptations<TPuzzle: SemiGroupActionPuzzle> {
    type Adaptations: SearchAdaptations<TPuzzle>;
}

impl DefaultSearchAdaptations<KPuzzle> for KPuzzle {
    type Adaptations = SearchAdaptationsHashPruneTableOnly<KPuzzle>;
}
