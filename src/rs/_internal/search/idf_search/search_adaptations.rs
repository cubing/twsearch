use std::marker::PhantomData;

use cubing::kpuzzle::KPuzzle;

use crate::_internal::puzzle_traits::puzzle_traits::{
    HashablePatternPuzzle, SemiGroupActionPuzzle,
};

use super::super::{
    hash_prune_table::HashPruneTable,
    pattern_validity_checker::{AlwaysValid, PatternValidityChecker},
    prune_table_trait::PruneTable,
};

/// The [`SearchAdaptations`] trait bundles various traits that IDFS can invoke
/// to change its search behaviour.
///
/// Each associated type could theoretically be a type parameter on
/// [`IDFSearch`](super::idf_search::IDFSearch) itself, but this would make
/// [`IDFSearch`](super::idf_search::IDFSearch) types rather unwieldy. So
/// instead of this:
///
///    IDFSearch<TPuzzle: …, PatternValidityChecker: …, PruneTable: …, (more in
///    the future…)>
///
/// we have this:
///
///    IDFSearch<TPuzzle: …, Adaptations: …>
///
/// In addition, the [`DefaultSearchAdaptations`] trait can be implemented for
/// any given [`SemiGroupActionPuzzle`]. This allows us to avoid specifying the
/// `Adaptations` type parameter on [`IDFSearch`](super::idf_search::IDFSearch)
/// in common situations. For example, the type for a [`KPuzzle`] search will
/// often just be:
///
///     IDFSearch<KPuzzle>
///
/// Note: the main reason that these are traits is that it enables "zero-cost"
/// abstraction in code that is run tens of millions of times per second. If you
/// implement a trait with trivial code, then this code can be inlined in an
/// instantiated [`IDFSearch`](super::idf_search::IDFSearch) without dynamic
/// dispatch.
///
/// TODO: figure out if/when dynamic dispatch is actually cheap and ergonomic
/// enough once we know all the adaptations we need for common puzzles.
pub trait SearchAdaptations<TPuzzle: SemiGroupActionPuzzle> {
    type PatternValidityChecker: PatternValidityChecker<TPuzzle>;
    type PruneTable: PruneTable<TPuzzle>;
}

pub struct SearchAdaptationsHashPruneTableOnly<TPuzzle: HashablePatternPuzzle> {
    phantom_data: PhantomData<TPuzzle>,
}

impl<TPuzzle: HashablePatternPuzzle> SearchAdaptations<TPuzzle>
    for SearchAdaptationsHashPruneTableOnly<TPuzzle>
{
    type PatternValidityChecker = AlwaysValid;
    type PruneTable = HashPruneTable<TPuzzle, Self::PatternValidityChecker>;
}

pub trait DefaultSearchAdaptations<TPuzzle: SemiGroupActionPuzzle> {
    type Adaptations: SearchAdaptations<TPuzzle>;
}

impl DefaultSearchAdaptations<KPuzzle> for KPuzzle {
    type Adaptations = SearchAdaptationsHashPruneTableOnly<KPuzzle>;
}
