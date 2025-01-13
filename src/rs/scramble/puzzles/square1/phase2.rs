use cubing::{
    alg::{parse_move, Move},
    kpuzzle::{KPattern, KPuzzle},
};
use lazy_static::lazy_static;

use crate::{
    _internal::{
        canonical_fsm::{
            canonical_fsm::MoveClassIndex,
            search_generators::{FlatMoveIndex, MoveTransformationInfo, SearchGenerators},
        },
        search::{
            coordinates::{
                phase_coordinate_puzzle::{SemanticCoordinate, TransformationOrIdentityNoOp},
                triple_phase_coordinate_puzzle::{
                    TriplePhaseCoordinatePruneTable, TriplePhaseCoordinatePuzzle,
                },
            },
            idf_search::search_adaptations::SearchAdaptations,
            mask_pattern::apply_mask,
            pattern_validity_checker::{AlwaysValid, PatternValidityChecker},
            prune_table_trait::Depth,
            recursion_filter_trait::RecursionFilter,
        },
    },
    scramble::puzzles::{
        definitions::{square1_corners_kpattern, square1_edges_kpattern, square1_equator_kpattern},
        square1::wedges::{WedgeType, WEDGE_TYPE_LOOKUP},
    },
};

use super::{phase1::restrict_D_move, solve::Square1SearchPhase};

struct Phase2ShapeChecker;

impl PatternValidityChecker<KPuzzle> for Phase2ShapeChecker {
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
pub(crate) struct Phase2ShapeCoordinate {
    // Holds 8 possible values, namely the Cartesian product of:
    //
    // - Equator sliced (or not).
    // - U off by (1, 0) from solved (or not).
    // - D off by (-1, 0) from solved (or not).
    pub(crate) equator: KPattern,
}

impl SemanticCoordinate<KPuzzle> for Phase2ShapeCoordinate {
    fn try_new(_kpuzzle: &KPuzzle, full_pattern: &KPattern) -> Option<Self> {
        // TODO: this isn't a full validity check for scramble positions.
        if !Phase2ShapeChecker::is_valid(full_pattern) {
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

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct Phase2EdgesCoordinate {
    pub(crate) edges: KPattern,
}

// TODO: make it impossible to use these at runtime without validating each one at least once?
// const U_SQ_MOVE_CLASS_INDEX: MoveClassIndex = MoveClassIndex(0);
// const D_SQ_MOVE_CLASS_INDEX: MoveClassIndex = MoveClassIndex(1);
const SLASH_MOVE_CLASS_INDEX: MoveClassIndex = MoveClassIndex(2);

const MOVE_AMOUNT_MULTIPLE_FOR_90_DEGREES: i32 = 3;
const MOVE_AMOUNT_MULTIPLE_FOR_360_DEGREES: i32 = MOVE_AMOUNT_MULTIPLE_FOR_90_DEGREES * 4;

lazy_static! {
    // (-2, 0) takes a pattern from full cube shape into a square-square shape
    // where:
    //
    // - The puzzle stays square-square even if `/` is applied right after.
    // - Edges are exactly in the same order (according to the KPuzzle
    //   definition) as they are in the pattern before this move was applied.
    static ref EDGES_START_PATTERN_OFFSET: Move = parse_move!("U_SQ_2'");

    // Same as `EDGES_START_PATTERN_OFFSET` but for corners: (1, 0)
    static ref CORNERS_START_PATTERN_OFFSET: Move = parse_move!("U_SQ_");
}

impl SemanticCoordinate<KPuzzle> for Phase2EdgesCoordinate {
    fn try_new(_kpuzzle: &KPuzzle, full_pattern: &KPattern) -> Option<Self> {
        // TODO: this isn't a full validity check for scramble positions.
        if !Phase2ShapeChecker::is_valid(full_pattern) {
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

    fn remap_start_pattern(pattern: KPattern) -> KPattern {
        pattern.apply_move(&EDGES_START_PATTERN_OFFSET).unwrap()
    }

    // Keep only 90° moves.
    fn keep_move_during_enumeration(
        move_transformation_info: &MoveTransformationInfo<KPuzzle>,
    ) -> bool {
        move_transformation_info.move_class_index == SLASH_MOVE_CLASS_INDEX
            || move_transformation_info.r#move.amount % MOVE_AMOUNT_MULTIPLE_FOR_90_DEGREES == 0
    }

    // TODO: document the math behind this.
    fn remap_move_during_transformation_application<'a>(
        transformation: &'a FlatMoveIndex,
        search_generators_for_tpuzzle: &'a SearchGenerators<KPuzzle>,
    ) -> TransformationOrIdentityNoOp<'a, FlatMoveIndex> {
        let from_move_transformation_info = search_generators_for_tpuzzle.flat.at(*transformation);
        let frm_move_class_index = from_move_transformation_info.move_class_index;

        if frm_move_class_index == SLASH_MOVE_CLASS_INDEX {
            return TransformationOrIdentityNoOp::Transformation(transformation);
        }
        let Move { amount, .. } = from_move_transformation_info.r#move;
        if amount % MOVE_AMOUNT_MULTIPLE_FOR_90_DEGREES == 0 {
            return TransformationOrIdentityNoOp::Transformation(transformation);
        }

        let amount_delta = match amount.rem_euclid(MOVE_AMOUNT_MULTIPLE_FOR_90_DEGREES) {
            0 => 0,
            1 => 2,
            2 => -2,
            _ => panic!("Impossible remainder."),
        };

        let new_amount = (amount + amount_delta).rem_euclid(MOVE_AMOUNT_MULTIPLE_FOR_360_DEGREES);
        if new_amount == 0 {
            TransformationOrIdentityNoOp::IdentityNoOp
        } else {
            let multiples = search_generators_for_tpuzzle
                .by_move_class
                .at(from_move_transformation_info.move_class_index);
            // Amount 1 is at index 0.
            // TODO: make this more semantically fabulous
            TransformationOrIdentityNoOp::Transformation(
                &multiples[(new_amount - 1) as usize].flat_move_index,
            )
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct Phase2CornersCoordinate {
    pub(crate) corners: KPattern,
}

impl SemanticCoordinate<KPuzzle> for Phase2CornersCoordinate {
    fn try_new(_kpuzzle: &KPuzzle, full_pattern: &KPattern) -> Option<Self> {
        // TODO: this isn't a full validity check for scramble positions.
        if !Phase2ShapeChecker::is_valid(full_pattern) {
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

pub(crate) type Square1Phase2Puzzle = TriplePhaseCoordinatePuzzle<
    KPuzzle,
    Phase2ShapeCoordinate,
    Phase2EdgesCoordinate,
    Phase2CornersCoordinate,
>;

impl RecursionFilter<Square1Phase2Puzzle> for Square1Phase2Puzzle {
    fn keep_move(
        move_transformation_info: &MoveTransformationInfo<Square1Phase2Puzzle>,
        remaining_depth: Depth,
    ) -> bool {
        if remaining_depth > Depth(6) {
            restrict_D_move(move_transformation_info)
        } else {
            true
        }
    }
}

pub(crate) struct Square1Phase2SearchAdaptations {}

/// Explicitly specifies search adaptations for [`Square1Phase2Puzzle`].
impl SearchAdaptations<Square1Phase2Puzzle> for Square1Phase2SearchAdaptations {
    type PatternValidityChecker = AlwaysValid;
    type PruneTable = TriplePhaseCoordinatePruneTable<
        KPuzzle,
        Phase2ShapeCoordinate,
        Phase2EdgesCoordinate,
        Phase2CornersCoordinate,
    >;
    type RecursionFilter = Square1Phase2Puzzle;
}

impl Square1SearchPhase for Square1Phase2Puzzle {}
