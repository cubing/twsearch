use cubing::kpuzzle::KPattern;

pub trait CheckPattern {
    fn is_valid(pattern: &KPattern) -> bool;
}

pub struct AlwaysValid;

impl CheckPattern for AlwaysValid {
    fn is_valid(_pattern: &KPattern) -> bool {
        true
    }
}
