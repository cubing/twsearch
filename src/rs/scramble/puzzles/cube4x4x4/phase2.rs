use cubing::kpuzzle::{KPattern, KPuzzle};

use crate::{
    _internal::search::coordinates::{
        masked_kpuzzle_deriver::MaskedPuzzleDeriver, pattern_deriver::PatternDeriver,
        unenumerated_derived_pattern_puzzle::UnenumeratedDerivedPatternPuzzle,
    },
    experimental_lib_api::{CompoundDerivedPuzzle, CompoundPuzzle},
    scramble::{
        puzzles::definitions::{
            cube4x4x4_kpuzzle, cube4x4x4_phase2_centers_target_kpattern,
            cube4x4x4_phase2_wing_parity_kpuzzle,
        },
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

pub(crate) type WingParityPuzzle =
    UnenumeratedDerivedPatternPuzzle<KPuzzle, KPuzzle, WingParityPatternDeriver>;

pub(crate) type Cube4x4x4Phase2Puzzle = CompoundDerivedPuzzle<
    KPuzzle,
    UnenumeratedDerivedPatternPuzzle<KPuzzle, KPuzzle, MaskedPuzzleDeriver>,
    WingParityPuzzle,
>;

impl Default for Cube4x4x4Phase2Puzzle {
    fn default() -> Self {
        let kpuzzle = cube4x4x4_kpuzzle();

        let masked_centers_puzzle_deriver =
            MaskedPuzzleDeriver::new(cube4x4x4_phase2_centers_target_kpattern().clone());
        let masked_centers_derived_puzzle = UnenumeratedDerivedPatternPuzzle::new(
            kpuzzle.clone(),
            kpuzzle.clone(),
            masked_centers_puzzle_deriver,
        );

        let wing_parity_pattern_deriver = WingParityPatternDeriver {};
        let wing_parity_derived_puzzle = UnenumeratedDerivedPatternPuzzle::new(
            kpuzzle.clone(),
            cube4x4x4_phase2_wing_parity_kpuzzle().clone(),
            wing_parity_pattern_deriver,
        );

        let compound_puzzle: CompoundPuzzle<
            UnenumeratedDerivedPatternPuzzle<KPuzzle, KPuzzle, MaskedPuzzleDeriver>,
            WingParityPuzzle,
        > = CompoundPuzzle {
            tpuzzle0: masked_centers_derived_puzzle,
            tpuzzle1: wing_parity_derived_puzzle.clone(),
        };
        compound_puzzle.into()
    }
}
