use cubing::kpuzzle::{KPattern, KPuzzle};

use crate::{
    _internal::search::coordinates::pattern_deriver::PatternDeriver,
    scramble::randomize::{basic_parity, BasicParity},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct WingParityPattern {
    pub(crate) parity: BasicParity,
}

impl WingParityPattern {
    // TODO: is there a good way to implement this without `unsafe`? Or should we expose this directly on `KPattern`?
    pub(crate) fn wing_permutation_slice(pattern: &KPattern) -> &[u8] {
        let orbit = &pattern.kpuzzle().data.ordered_orbit_info[0];
        assert_eq!(orbit.name.0, "WINGS");

        let from = orbit.orientations_offset;
        let to = from + (orbit.num_pieces as usize);

        let full_byte_slice = unsafe { pattern.byte_slice() };
        &full_byte_slice[from..to]
    }
}

#[derive(Clone, Debug)]
pub(crate) struct WingParityPuzzle {}

impl PatternDeriver<KPuzzle> for WingParityPuzzle {
    type DerivedPattern = WingParityPattern;

    fn derive_pattern(&self, source_puzzle_pattern: &KPattern) -> Option<WingParityPattern> {
        Some(WingParityPattern {
            parity: basic_parity(WingParityPattern::wing_permutation_slice(
                source_puzzle_pattern,
            )),
        })
    }
}
