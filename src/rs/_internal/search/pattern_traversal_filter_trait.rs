use crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle;

pub trait PatternTraversalFilter<TPuzzle: SemiGroupActionPuzzle> {
    fn is_valid(pattern: &TPuzzle::Pattern) -> bool;
}

pub struct PatternTraversalFilterNoOp;

impl<TPuzzle: SemiGroupActionPuzzle> PatternTraversalFilter<TPuzzle>
    for PatternTraversalFilterNoOp
{
    fn is_valid(_pattern: &TPuzzle::Pattern) -> bool {
        true
    }
}
