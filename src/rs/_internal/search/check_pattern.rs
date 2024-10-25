use crate::_internal::puzzle_traits::SemiGroupActionPuzzle;

pub trait PatternValidityChecker<TPuzzle: SemiGroupActionPuzzle> {
    fn is_valid(pattern: &TPuzzle::Pattern) -> bool;
}

pub struct AlwaysValid;

impl<TPuzzle: SemiGroupActionPuzzle> PatternValidityChecker<TPuzzle> for AlwaysValid {
    fn is_valid(_pattern: &TPuzzle::Pattern) -> bool {
        true
    }
}
