use cubing::kpuzzle::KPattern;

pub trait PatternValidityChecker {
    fn is_valid(pattern: &KPattern) -> bool;
}

pub struct AlwaysValid;

impl PatternValidityChecker for AlwaysValid {
    fn is_valid(_pattern: &KPattern) -> bool {
        true
    }
}
