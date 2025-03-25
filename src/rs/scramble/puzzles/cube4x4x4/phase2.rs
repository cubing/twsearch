use cubing::kpuzzle::{KPattern, KPuzzle};

use crate::{
    _internal::search::coordinates::{
        pattern_deriver::PatternDeriver,
        unenumerated_derived_pattern_puzzle::UnenumeratedDerivedPatternPuzzle,
    },
    scramble::{
        puzzles::definitions::cube4x4x4_phase2_wing_parity_kpuzzle,
        randomize::{basic_parity, BasicParity},
    },
};

// #[derive(Debug, Clone, Hash, PartialEq, Eq)]
// pub(crate) struct WingParityPattern {
//     pub(crate) parity: BasicParity,
// }

fn wing_permutation_slice(pattern: &KPattern) -> &[u8] {
    let orbit = &pattern.kpuzzle().data.ordered_orbit_info[1];
    assert_eq!(orbit.name.0, "WINGS");

    let from = orbit.orientations_offset;
    let to = from + (orbit.num_pieces as usize);

    let full_byte_slice = unsafe { pattern.byte_slice() };
    &full_byte_slice[from..to]
}

#[derive(Clone, Debug)]
pub(crate) struct WingParityPatternDeriver {}

impl PatternDeriver<KPuzzle> for WingParityPatternDeriver {
    type DerivedPattern = KPattern;

    fn derive_pattern(&self, source_puzzle_pattern: &KPattern) -> Option<Self::DerivedPattern> {
        // TODO: optimize this
        let kpuzzle = cube4x4x4_phase2_wing_parity_kpuzzle(); // TODO: cache on self?
        let mut pattern = kpuzzle.default_pattern();
        let orbit = &kpuzzle.data.ordered_orbit_info[0];
        let parity = basic_parity(wing_permutation_slice(source_puzzle_pattern));
        pattern.set_piece(
            orbit,
            0,
            match parity {
                BasicParity::Even => 0,
                BasicParity::Odd => 1,
            },
        );
        Some(pattern)
    }
}

pub type WingParityPuzzle =
    UnenumeratedDerivedPatternPuzzle<KPuzzle, KPuzzle, WingParityPatternDeriver>;
