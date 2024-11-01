use cubing::kpuzzle::{KPattern, KPuzzle};

use crate::{
    _internal::search::{
        check_pattern::PatternValidityChecker,
        coordinates::{
            phase_coordinate_puzzle::SemanticCoordinate,
            triple_phase_coordinate_puzzle::TriplePhaseCoordinatePuzzle,
        },
    },
    scramble::puzzles::{
        definitions::{square1_corners_kpattern, square1_edges_kpattern, square1_equator_kpattern},
        mask_pattern::apply_mask,
        square1::wedges::{WedgeType, WEDGE_TYPE_LOOKUP},
    },
};

struct Phase2Checker;

impl PatternValidityChecker<KPuzzle> for Phase2Checker {
    fn is_valid(pattern: &cubing::kpuzzle::KPattern) -> bool {
        let orbit_info = &pattern.kpuzzle().data.ordered_orbit_info[0];
        assert_eq!(orbit_info.name.0, "WEDGES");

        for slot in [0, 1, 2, 12, 13, 14] {
            let value = unsafe {
                pattern
                    .packed_orbit_data()
                    .get_raw_piece_or_permutation_value(orbit_info, slot)
            };
            let wedge_type = &WEDGE_TYPE_LOOKUP[value as usize];

            if *wedge_type == WedgeType::CornerUpper && (slot == 0 || slot == 12) {
                // We can't slice.
                return false;
            }

            for slot_offset in [3, 6, 9] {
                let offset_value = unsafe {
                    pattern
                        .packed_orbit_data()
                        .get_raw_piece_or_permutation_value(orbit_info, slot + slot_offset)
                };
                let offset_wedge_type = &WEDGE_TYPE_LOOKUP[offset_value as usize];

                if wedge_type != offset_wedge_type {
                    return false;
                }
            }
        }

        true
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct Phase2EdgesCoordinate {
    edges: KPattern,
}

impl SemanticCoordinate<KPuzzle> for Phase2EdgesCoordinate {
    fn try_new(_kpuzzle: &KPuzzle, full_pattern: &KPattern) -> Option<Self> {
        // TODO: this isn't a full validity check for scramble positions.
        if !Phase2Checker::is_valid(full_pattern) {
            return None;
        }

        let phase_mask = square1_edges_kpattern(); // TODO: Store this with the coordinate lookup?
        let Ok(masked_pattern) = apply_mask(full_pattern, phase_mask) else {
            panic!("Mask application failed");
        };

        Some(Self {
            edges: masked_pattern,
        })
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct Phase2CornersCoordinate {
    corners: KPattern,
}

impl SemanticCoordinate<KPuzzle> for Phase2CornersCoordinate {
    fn try_new(_kpuzzle: &KPuzzle, full_pattern: &KPattern) -> Option<Self> {
        // TODO: this isn't a full validity check for scramble positions.
        if !Phase2Checker::is_valid(full_pattern) {
            return None;
        }

        let phase_mask = square1_corners_kpattern(); // TODO: Store this with the coordinate lookup?
        let Ok(masked_pattern) = apply_mask(full_pattern, phase_mask) else {
            panic!("Mask application failed");
        };

        Some(Self {
            corners: masked_pattern,
        })
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct Phase2EquatorCoordinate {
    equator: KPattern,
}

impl SemanticCoordinate<KPuzzle> for Phase2EquatorCoordinate {
    fn try_new(_kpuzzle: &KPuzzle, full_pattern: &KPattern) -> Option<Self> {
        // TODO: this isn't a full validity check for scramble positions.
        if !Phase2Checker::is_valid(full_pattern) {
            return None;
        }

        let phase_mask = square1_equator_kpattern(); // TODO: Store this with the coordinate lookup?
        let Ok(masked_pattern) = apply_mask(full_pattern, phase_mask) else {
            panic!("Mask application failed");
        };

        Some(Self {
            equator: masked_pattern,
        })
    }
}

pub(crate) type Square1Phase2Puzzle = TriplePhaseCoordinatePuzzle<
    KPuzzle,
    Phase2EdgesCoordinate,
    Phase2CornersCoordinate,
    Phase2EquatorCoordinate,
>;
