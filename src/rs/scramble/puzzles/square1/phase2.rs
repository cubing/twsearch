use cubing::{
    alg::parse_alg,
    kpuzzle::{KPattern, KPuzzle},
};

use crate::{
    _internal::{
        canonical_fsm::search_generators::{FlatMoveIndex, MoveTransformationInfo},
        puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
        search::{
            coordinates::{
                phase_coordinate_puzzle::{
                    PhaseCoordinateConversionError, PhaseCoordinateIndex, PhaseCoordinatePuzzle,
                    SemanticCoordinate,
                },
                triple_phase_coordinate_puzzle::{
                    TriplePhaseCoordinatePruneTable, TriplePhaseCoordinatePuzzle,
                },
            },
            idf_search::search_adaptations::{DefaultSearchAdaptations, SearchAdaptations},
            mask_pattern::apply_mask,
            pattern_validity_checker::{AlwaysValid, PatternValidityChecker},
            prune_table_trait::{Depth, PruneTable},
            recursion_filter_trait::{RecursionFilter, RecursionFilterNoOp},
        },
    },
    scramble::{
        puzzles::{
            definitions::{
                square1_corners_kpattern, square1_edges_kpattern, square1_shape_kpattern,
                square1_unbandaged_kpuzzle,
            },
            square1::wedges::{WedgeType, WEDGE_TYPE_LOOKUP},
        },
        scramble_search::move_list_from_vec,
    },
};

use super::{phase1::restrict_D_move, solve::Square1SearchPhase};

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
pub(crate) struct Phase2ShapeCoordinate {
    pub(crate) shape: KPattern,
}

impl SemanticCoordinate<KPuzzle> for Phase2ShapeCoordinate {
    fn phase_name() -> &'static str {
        "Shape (Square-1 → phase 2)"
    }

    fn try_new(_kpuzzle: &KPuzzle, full_pattern: &KPattern) -> Option<Self> {
        // TODO: this isn't a full validity check for scramble positions.
        if !Phase2Checker::is_valid(full_pattern) {
            return None;
        }

        let phase_mask = square1_shape_kpattern(); // TODO: Store this with the coordinate lookup?
        let Ok(masked_pattern) = apply_mask(full_pattern, phase_mask) else {
            panic!("Mask application failed");
        };

        Some(Self {
            shape: masked_pattern,
        })
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct Phase2EdgesCoordinate {
    pub(crate) edges: KPattern,
}

impl SemanticCoordinate<KPuzzle> for Phase2EdgesCoordinate {
    fn phase_name() -> &'static str {
        "Edges (Square-1 → phase 2)"
    }

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
    pub(crate) corners: KPattern,
}

impl SemanticCoordinate<KPuzzle> for Phase2CornersCoordinate {
    fn phase_name() -> &'static str {
        "Corners (Square-1 → phase 2)"
    }

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

// TODO: rename to "shape" and move this above edges and corners
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct Phase2EquatorCoordinate {
    pub(crate) equator: KPattern,
}

impl SemanticCoordinate<KPuzzle> for Phase2EquatorCoordinate {
    fn phase_name() -> &'static str {
        "Equator (Square-1 → phase 2)"
    }

    fn try_new(_kpuzzle: &KPuzzle, full_pattern: &KPattern) -> Option<Self> {
        // TODO: this isn't a full validity check for scramble positions.
        if !Phase2Checker::is_valid(full_pattern) {
            return None;
        }

        let phase_mask = square1_shape_kpattern(); // TODO: Store this with the coordinate lookup?
        let Ok(masked_pattern) = apply_mask(full_pattern, phase_mask) else {
            panic!("Mask application failed");
        };

        Some(Self {
            equator: masked_pattern,
        })
    }
}

// TODO: generalize this, similar to how `TriplePhaseCoordinatePuzzle` does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Square1Phase2TripleCoordinate {
    equator: PhaseCoordinateIndex<PhaseCoordinatePuzzle<KPuzzle, Phase2EquatorCoordinate>>, // TODO: rename to "shape"
    edges: PhaseCoordinateIndex<PhaseCoordinatePuzzle<KPuzzle, Phase2EdgesCoordinate>>,
    corners: PhaseCoordinateIndex<PhaseCoordinatePuzzle<KPuzzle, Phase2CornersCoordinate>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Square1Phase2Transformation(FlatMoveIndex);

#[derive(Clone, Debug)]
pub struct Square1Phase2Puzzle {
    equator_puzzle: PhaseCoordinatePuzzle<KPuzzle, Phase2EquatorCoordinate>, // TODO: rename to "shape"
    edges_puzzle: PhaseCoordinatePuzzle<KPuzzle, Phase2EdgesCoordinate>,
    corners_puzzle: PhaseCoordinatePuzzle<KPuzzle, Phase2CornersCoordinate>,
}

impl Square1Phase2Puzzle {
    pub fn new() -> Self {
        let kpuzzle = square1_unbandaged_kpuzzle();

        let equator_puzzle = {
            let full_generator_moves = move_list_from_vec(vec!["U_SQ_", "D_SQ_", "/"]);

            let start_pattern = square1_shape_kpattern()
                .apply_alg(&parse_alg!("(0, 0)"))
                .unwrap(); // TODO: do we need an adjustment for the equator?
            PhaseCoordinatePuzzle::<KPuzzle, Phase2EquatorCoordinate>::new(
                kpuzzle.clone(),
                start_pattern,
                full_generator_moves,
            )
        };

        let reduced_generator_moves = move_list_from_vec(vec!["U_SQ_3", "D_SQ_3", "/"]);

        let edges_puzzle = {
            let start_pattern = square1_edges_kpattern()
                .apply_alg(&parse_alg!("(-2, 0)"))
                .unwrap(); // TODO: do we need an adjustment for the equator?
            PhaseCoordinatePuzzle::<KPuzzle, Phase2EdgesCoordinate>::new(
                kpuzzle.clone(),
                start_pattern,
                reduced_generator_moves.clone(),
            )
        };

        // TODO: reuse the table from edges (with different move remappings)
        let corners_puzzle = {
            let start_pattern = square1_corners_kpattern()
                .apply_alg(&parse_alg!("(1, 0)"))
                .unwrap(); // TODO: do we need an adjustment for the equator?
            PhaseCoordinatePuzzle::<KPuzzle, Phase2CornersCoordinate>::new(
                kpuzzle.clone(),
                start_pattern,
                reduced_generator_moves,
            )
        };

        Self {
            equator_puzzle,
            edges_puzzle,
            corners_puzzle,
        }
    }

    // TODO: report errors for invalid patterns
    pub fn full_pattern_to_phase_coordinate(
        &self,
        pattern: &KPattern,
    ) -> Result<Square1Phase2TripleCoordinate, PhaseCoordinateConversionError> {
        let Ok(equator) = self
            .equator_puzzle
            .full_pattern_to_phase_coordinate(pattern)
        else {
            return Err(PhaseCoordinateConversionError::InvalidSemanticCoordinate);
        };
        let Ok(edges) = self.edges_puzzle.full_pattern_to_phase_coordinate(pattern) else {
            return Err(PhaseCoordinateConversionError::InvalidSemanticCoordinate);
        };
        let Ok(corners) = self
            .corners_puzzle
            .full_pattern_to_phase_coordinate(pattern)
        else {
            return Err(PhaseCoordinateConversionError::InvalidSemanticCoordinate);
        };
        Ok(Square1Phase2TripleCoordinate {
            equator,
            edges,
            corners,
        })
    }
}

impl SemiGroupActionPuzzle for Square1Phase2Puzzle {
    type Pattern = Square1Phase2TripleCoordinate;

    // TODO: use a unified definition of these flat moves.
    type Transformation = Square1Phase2Transformation;

    fn move_order(
        &self,
        r#move: &cubing::alg::Move,
    ) -> Result<crate::_internal::search::move_count::MoveCount, cubing::kpuzzle::InvalidAlgError>
    {
        todo!()
    }

    fn puzzle_transformation_from_move(
        &self,
        r#move: &cubing::alg::Move,
    ) -> Result<Self::Transformation, cubing::kpuzzle::InvalidAlgError> {
        todo!()
    }

    fn do_moves_commute(
        &self,
        move1_info: &MoveTransformationInfo<Self>,
        move2_info: &MoveTransformationInfo<Self>,
    ) -> bool {
        todo!()
    }

    fn pattern_apply_transformation(
        // TODO: this is a hack to allow `Phase2Puzzle` to access its tables, ideally we would avoid this.
        // Then again, this might turn out to be necessary for similar high-performance implementations.
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Option<Self::Pattern> {
        todo!()
    }

    fn pattern_apply_transformation_into(
        // TODO: this is a hack to allow `Phase2Puzzle` to access its tables, ideally we would avoid this.
        // Then again, this might turn out to be necessary for similar high-performance implementations.
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    ) -> bool {
        todo!()
    }
}

#[derive(Clone)]
pub struct Square1Phase2PruneTable {}

impl PruneTable<Square1Phase2Puzzle> for Square1Phase2PruneTable {
    fn new(
        tpuzzle: Square1Phase2Puzzle,
        search_api_data: std::sync::Arc<
            crate::_internal::search::idf_search::idf_search::IDFSearchAPIData<Square1Phase2Puzzle>,
        >,
        search_logger: std::sync::Arc<crate::_internal::search::search_logger::SearchLogger>,
        min_size: Option<usize>,
    ) -> Self {
        todo!()
    }

    fn lookup(&self, pattern: &<Square1Phase2Puzzle as SemiGroupActionPuzzle>::Pattern) -> Depth {
        todo!()
    }

    fn extend_for_search_depth(&mut self, search_depth: Depth, approximate_num_entries: usize) {
        todo!()
    }
}

#[derive(Clone)]
pub struct Square1Phase2SearchAdaptations {}

/// Explicitly specifies search adaptations for [`Square1Phase2Puzzle`].
impl SearchAdaptations<Square1Phase2Puzzle> for Square1Phase2SearchAdaptations {
    type PatternValidityChecker = AlwaysValid;
    type PruneTable = Square1Phase2PruneTable; // TODO: should this be a no-op?
    type RecursionFilter = RecursionFilterNoOp; // TODO: Square1Phase2Puzzle;
}

impl DefaultSearchAdaptations<Square1Phase2Puzzle> for Square1Phase2Puzzle {
    type Adaptations = Square1Phase2SearchAdaptations;
}
