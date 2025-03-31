use cubing::kpuzzle::{KPattern, KPuzzle};

use crate::_internal::{
    puzzle_traits::puzzle_traits::HashablePatternPuzzle, search::mask_pattern::apply_mask,
};

use super::{
    pattern_deriver::PatternDeriver,
    unenumerated_derived_pattern_puzzle::UnenumeratedDerivedPatternPuzzle,
};

#[derive(Clone, Debug)]
pub struct MaskedKPuzzleDeriver {
    mask: KPattern,
}

impl MaskedKPuzzleDeriver {
    pub fn new(mask: KPattern) -> Self {
        Self { mask }
    }
}

impl PatternDeriver<KPuzzle> for MaskedKPuzzleDeriver {
    type DerivedPattern = KPattern;

    fn derive_pattern(&self, source_puzzle_pattern: &KPattern) -> Option<Self::DerivedPattern> {
        apply_mask(source_puzzle_pattern, &self.mask).ok()
    }
}

pub type MaskedDerivedKPuzzle =
    UnenumeratedDerivedPatternPuzzle<KPuzzle, KPuzzle, MaskedKPuzzleDeriver>;

impl MaskedDerivedKPuzzle {
    pub fn new_from_mask(kpattern: KPattern) -> Self {
        let kpuzzle = kpattern.kpuzzle().clone();
        let pattern_deriver = MaskedKPuzzleDeriver::new(kpattern);
        Self {
            source_puzzle: kpuzzle.clone(),
            derived_puzzle: kpuzzle,
            pattern_deriver,
        }
    }
}

impl HashablePatternPuzzle for MaskedDerivedKPuzzle {
    fn pattern_hash_u64(&self, pattern: &Self::Pattern) -> u64 {
        self.derived_puzzle.pattern_hash_u64(pattern)
    }
}
