use std::marker::PhantomData;

use cubing::kpuzzle::KPuzzle;

use crate::_internal::puzzle_traits::puzzle_traits::{
    HashablePatternPuzzle, SemiGroupActionPuzzle,
};

use super::super::{
    check_pattern::{AlwaysValid, PatternValidityChecker},
    hash_prune_table::HashPruneTable,
    prune_table_trait::PruneTable,
};

pub trait SearchOptimizations<TPuzzle: SemiGroupActionPuzzle> {
    type PatternValidityChecker: PatternValidityChecker<TPuzzle>;
    type PruneTable: PruneTable<TPuzzle>;
}

pub struct NoSearchOptimizations<TPuzzle: HashablePatternPuzzle> {
    phantom_data: PhantomData<TPuzzle>,
}

impl<TPuzzle: HashablePatternPuzzle> SearchOptimizations<TPuzzle>
    for NoSearchOptimizations<TPuzzle>
{
    type PatternValidityChecker = AlwaysValid;
    type PruneTable = HashPruneTable<TPuzzle, Self::PatternValidityChecker>;
}

pub trait DefaultSearchOptimizations<TPuzzle: SemiGroupActionPuzzle> {
    type Optimizations: SearchOptimizations<TPuzzle>;
}

impl DefaultSearchOptimizations<KPuzzle> for KPuzzle {
    type Optimizations = NoSearchOptimizations<KPuzzle>;
}
