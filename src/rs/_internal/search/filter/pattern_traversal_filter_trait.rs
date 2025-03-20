use crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle;

use super::filtering_decision::FilteringDecision;

pub trait PatternTraversalFilter<TPuzzle: SemiGroupActionPuzzle> {
    fn filter_pattern(pattern: &TPuzzle::Pattern) -> FilteringDecision;
}

pub struct PatternTraversalFilterNoOp;

impl<TPuzzle: SemiGroupActionPuzzle> PatternTraversalFilter<TPuzzle>
    for PatternTraversalFilterNoOp
{
    fn filter_pattern(_pattern: &TPuzzle::Pattern) -> FilteringDecision {
        FilteringDecision::Accept
    }
}
