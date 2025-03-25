use cubing::kpuzzle::{KPattern, KPuzzle};

use crate::_internal::search::mask_pattern::apply_mask;

use super::pattern_deriver::PatternDeriver;

#[derive(Clone, Debug)]
pub struct MaskedPuzzleDeriver {
    mask: KPattern,
}

impl MaskedPuzzleDeriver {
    pub fn new(mask: KPattern) -> Self {
        Self { mask }
    }
}

impl PatternDeriver<KPuzzle> for MaskedPuzzleDeriver {
    type DerivedPattern = KPattern;

    fn derive_pattern(&self, source_puzzle_pattern: &KPattern) -> Option<Self::DerivedPattern> {
        apply_mask(source_puzzle_pattern, &self.mask).ok()
    }
}
