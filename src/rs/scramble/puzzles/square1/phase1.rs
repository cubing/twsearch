use std::{fmt::Debug, ops::Range};

use cubing::{
    alg::Move,
    kpuzzle::{KPattern, KPuzzle},
};

use crate::{
    _internal::{
        canonical_fsm::{canonical_fsm::MoveClassIndex, search_generators::MoveTransformationInfo},
        search::{
            coordinates::phase_coordinate_puzzle::{
                PhaseCoordinatePruneTable, PhaseCoordinatePuzzle, SemanticCoordinate,
            },
            idf_search::search_adaptations::SearchAdaptations,
            mask_pattern::apply_mask,
            pattern_validity_checker::{AlwaysValid, PatternValidityChecker},
            prune_table_trait::Depth,
            recursion_filter_trait::RecursionFilter,
        },
    },
    scramble::{
        puzzles::square1::wedges::{WedgeType, WEDGE_TYPE_LOOKUP},
        randomize::BasicParity,
    },
};

use super::{
    super::definitions::square1_square_square_shape_kpattern, parity::bandaged_wedge_parity,
    solve::Square1SearchPhase,
};

use lazy_static::lazy_static;

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
    fn phase_name() -> &'static str {
        "U/D shape (Square-1 → phase 1)"
    }

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

// TODO: allow flipping this depending on whether this is for a scramble (backwards) or a solution (forwards)?
const D_SQ_MOVE_RESTRICTED_RANGE: Range<i32> = -3..3;

// This is exported so it can be reused by phase 2.
#[allow(non_snake_case)]
pub fn restrict_D_move<Phase: Square1SearchPhase>(
    move_transformation_info: &MoveTransformationInfo<Phase>,
) -> bool {
    lazy_static! {
        // TODO: perform a one-time check that this matches the search generator indexing.
        static ref D_MOVE_CLASS_INDEX: MoveClassIndex = MoveClassIndex(1);
    }
    if move_transformation_info.move_class_index != *D_MOVE_CLASS_INDEX {
        return true;
    }
    let Move { amount, .. } = move_transformation_info.r#move;
    D_SQ_MOVE_RESTRICTED_RANGE.contains(&amount)
}

impl RecursionFilter<Square1Phase1Puzzle> for Square1Phase1Puzzle {
    fn keep_move(
        move_transformation_info: &MoveTransformationInfo<Square1Phase1Puzzle>,
        _remaining_depth: Depth,
    ) -> bool {
        restrict_D_move(move_transformation_info)
    }
}

pub(crate) struct Square1Phase1SearchAdaptations {}

/// Explicitly specifies search adaptations for [`Square1Phase1Puzzle`].
impl SearchAdaptations<Square1Phase1Puzzle> for Square1Phase1SearchAdaptations {
    type PatternValidityChecker = AlwaysValid;
    type PruneTable = PhaseCoordinatePruneTable<KPuzzle, Square1Phase1Coordinate>;
    type RecursionFilter = Square1Phase1Puzzle;
}

impl Square1SearchPhase for Square1Phase1Puzzle {}
