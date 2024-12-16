use std::fmt::Debug;

use cubing::kpuzzle::{KPattern, KPuzzle};

use crate::{
    _internal::search::{
        check_pattern::PatternValidityChecker,
        coordinates::phase_coordinate_puzzle::{PhaseCoordinatePuzzle, SemanticCoordinate},
    },
    scramble::{
        puzzles::{
            mask_pattern::apply_mask,
            square1::wedges::{WedgeType, WEDGE_TYPE_LOOKUP},
        },
        randomize::BasicParity,
    },
};

use super::{
    super::definitions::square1_square_square_shape_kpattern, parity::bandaged_wedge_parity,
};

pub(crate) struct Phase1Checker;

const SLOTS_THAT_ARE_AFTER_SLICES: [u8; 4] = [0, 6, 12, 18];

impl PatternValidityChecker<KPuzzle> for Phase1Checker {
    fn is_valid(pattern: &cubing::kpuzzle::KPattern) -> bool {
        let orbit_info = &pattern.kpuzzle().data.ordered_orbit_info[0];
        assert_eq!(orbit_info.name.0, "WEDGES");

        for slot in SLOTS_THAT_ARE_AFTER_SLICES {
            let value = unsafe {
                pattern
                    .packed_orbit_data()
                    .get_raw_piece_or_permutation_value(orbit_info, slot)
            };

            // TODO: consider removing this lookup. We know that the wedge values are only 0, 1, or
            // 2 during this phase.
            if WEDGE_TYPE_LOOKUP[value as usize] == WedgeType::CornerUpper {
                return false;
            }
        }

        true
    }
}

// Note that this entire struct consists of single coordinate.
// The fields themselves are more like "subcoordinates" rather than coordinates in themselves.
// TODO: Implement automatic coordinate composition?
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct Square1Phase1Coordinate {
    masked_pattern: KPattern,
    parity: BasicParity,
}

impl SemanticCoordinate<KPuzzle> for Square1Phase1Coordinate {
    fn try_new(_kpuzzle: &KPuzzle, full_pattern: &KPattern) -> Option<Self> {
        let phase_mask = &square1_square_square_shape_kpattern(); // TODO: Store this with the coordinate lookup?
        let Ok(masked_pattern) = apply_mask(full_pattern, phase_mask) else {
            panic!("Mask application failed");
        };

        // TODO: this isn't a full validity check for scramble positions.
        if !Phase1Checker::is_valid(&masked_pattern) {
            return None;
        }

        let parity = bandaged_wedge_parity(full_pattern);
        Some(Self {
            masked_pattern,
            parity,
        })
    }
}

pub(crate) type Square1Phase1Puzzle = PhaseCoordinatePuzzle<KPuzzle, Square1Phase1Coordinate>;
