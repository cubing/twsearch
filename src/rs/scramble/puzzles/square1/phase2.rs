use cubing::{
    alg::{parse_alg, parse_move, Alg, Move},
    kpuzzle::{KPattern, KPuzzle},
};
use lazy_static::lazy_static;
use std::cmp::max;

use crate::{
    _internal::{
        canonical_fsm::search_generators::{FlatMoveIndex, MoveTransformationInfo},
        puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
        search::{
            coordinates::phase_coordinate_puzzle::{
                PhaseCoordinateConversionError, PhaseCoordinateIndex, PhaseCoordinatePuzzle,
                SemanticCoordinate,
            },
            idf_search::search_adaptations::{DefaultSearchAdaptations, SearchAdaptations},
            indexed_vec::IndexedVec,
            mask_pattern::apply_mask,
            pattern_validity_checker::{AlwaysValid, PatternValidityChecker},
            prune_table_trait::{Depth, PruneTable},
            recursion_filter_trait::RecursionFilterNoOp,
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

use super::wedges::{get_phase2_shape_offsets, Square1Phase2Offsets};

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

// TODO: generalize this, similar to how `TriplePhaseCoordinatePuzzle` does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Square1Phase2TripleCoordinate {
    shape: PhaseCoordinateIndex<PhaseCoordinatePuzzle<KPuzzle, Phase2ShapeCoordinate>>, // TODO: rename to "shape"
    edges: PhaseCoordinateIndex<PhaseCoordinatePuzzle<KPuzzle, Phase2EdgesCoordinate>>,
    corners: PhaseCoordinateIndex<PhaseCoordinatePuzzle<KPuzzle, Phase2CornersCoordinate>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Square1Phase2Transformation(FlatMoveIndex); // Index into search generators for the shape.

#[derive(Clone, Debug)]
pub struct Square1Phase2Puzzle {
    shape_puzzle: PhaseCoordinatePuzzle<KPuzzle, Phase2ShapeCoordinate>,
    shape_to_offsets: IndexedVec<
        PhaseCoordinateIndex<PhaseCoordinatePuzzle<KPuzzle, Phase2ShapeCoordinate>>,
        Square1Phase2Offsets,
    >,
    edges_puzzle: PhaseCoordinatePuzzle<KPuzzle, Phase2EdgesCoordinate>,
    corners_puzzle: PhaseCoordinatePuzzle<KPuzzle, Phase2CornersCoordinate>,
}

#[allow(non_snake_case)]
fn square1_tuple(U_SQ_amount: i32, D_SQ_amount: i32) -> Alg {
    lazy_static! {
        static ref U_SQ_: Move = parse_move!("U_SQ_");
        static ref D_SQ_: Move = parse_move!("D_SQ_");
    };

    Alg {
        nodes: vec![
            Move {
                quantum: U_SQ_.quantum.clone(),
                amount: U_SQ_amount,
            }
            .into(),
            Move {
                quantum: D_SQ_.quantum.clone(),
                amount: D_SQ_amount,
            }
            .into(),
        ],
    }
}

// An optimized representation of a Square-1 move. Note that the set "valid"
// `i32` values depends on the calling code, as different conventions are needed
// for different calculations.
#[derive(Copy, Clone, Debug)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
enum Square1Phase2Move {
    U_SQ_(i32),
    D_SQ_(i32),
    SLASH,
}

impl From<FlatMoveIndex> for Square1Phase2Move {
    fn from(value: FlatMoveIndex) -> Self {
        match value.0 {
            0 => Square1Phase2Move::U_SQ_(1),
            1 => Square1Phase2Move::U_SQ_(2),
            2 => Square1Phase2Move::U_SQ_(3),
            3 => Square1Phase2Move::U_SQ_(4),
            4 => Square1Phase2Move::U_SQ_(5),
            5 => Square1Phase2Move::U_SQ_(6),
            6 => Square1Phase2Move::U_SQ_(7),
            7 => Square1Phase2Move::U_SQ_(8),
            8 => Square1Phase2Move::U_SQ_(9),
            9 => Square1Phase2Move::U_SQ_(10),
            10 => Square1Phase2Move::U_SQ_(11),
            11 => Square1Phase2Move::D_SQ_(1),
            12 => Square1Phase2Move::D_SQ_(2),
            13 => Square1Phase2Move::D_SQ_(3),
            14 => Square1Phase2Move::D_SQ_(4),
            15 => Square1Phase2Move::D_SQ_(5),
            16 => Square1Phase2Move::D_SQ_(6),
            17 => Square1Phase2Move::D_SQ_(7),
            18 => Square1Phase2Move::D_SQ_(8),
            19 => Square1Phase2Move::D_SQ_(9),
            20 => Square1Phase2Move::D_SQ_(10),
            21 => Square1Phase2Move::D_SQ_(11),
            22 => Square1Phase2Move::SLASH,
            _ => panic!("Invalid move"), // TODO: is it faster to just return `SLASH` as the default?
        }
    }
}

// TODO: more type safety
impl Square1Phase2Move {
    // Accepts amounts from `0` to `11` (for `U_SQ_` or `D_SQ_` moves).
    fn to_edge_or_corner_transformation(self) -> Option<FlatMoveIndex> {
        Some(FlatMoveIndex(match self {
            Square1Phase2Move::U_SQ_(0) => return None,
            Square1Phase2Move::U_SQ_(1) => return None,
            Square1Phase2Move::U_SQ_(2) => return None,
            Square1Phase2Move::U_SQ_(3) => 0,
            Square1Phase2Move::U_SQ_(4) => 0,
            Square1Phase2Move::U_SQ_(5) => 0,
            Square1Phase2Move::U_SQ_(6) => 1,
            Square1Phase2Move::U_SQ_(7) => 1,
            Square1Phase2Move::U_SQ_(8) => 1,
            Square1Phase2Move::U_SQ_(9) => 2,
            Square1Phase2Move::U_SQ_(10) => 2,
            Square1Phase2Move::U_SQ_(11) => 2,
            Square1Phase2Move::D_SQ_(0) => return None,
            Square1Phase2Move::D_SQ_(1) => return None,
            Square1Phase2Move::D_SQ_(2) => return None,
            Square1Phase2Move::D_SQ_(3) => 3,
            Square1Phase2Move::D_SQ_(4) => 3,
            Square1Phase2Move::D_SQ_(5) => 3,
            Square1Phase2Move::D_SQ_(6) => 4,
            Square1Phase2Move::D_SQ_(7) => 4,
            Square1Phase2Move::D_SQ_(8) => 4,
            Square1Phase2Move::D_SQ_(9) => 5,
            Square1Phase2Move::D_SQ_(10) => 5,
            Square1Phase2Move::D_SQ_(11) => 5,
            Square1Phase2Move::SLASH => 6,
            _ => panic!("Invalid move"), // TODO: is it faster to just return `None` as the default?
        }))
    }
}

impl Square1Phase2Puzzle {
    pub fn new() -> Self {
        let kpuzzle = square1_unbandaged_kpuzzle();

        let shape_puzzle = {
            let full_generator_moves = move_list_from_vec(vec!["U_SQ_", "D_SQ_", "/"]);

            let start_pattern = square1_shape_kpattern()
                .apply_alg(&parse_alg!("(0, 0)"))
                .unwrap();
            PhaseCoordinatePuzzle::<KPuzzle, Phase2ShapeCoordinate>::new(
                kpuzzle.clone(),
                start_pattern,
                full_generator_moves,
            )
        };

        let mut shape_to_offsets = IndexedVec::<
            PhaseCoordinateIndex<PhaseCoordinatePuzzle<KPuzzle, Phase2ShapeCoordinate>>,
            Square1Phase2Offsets,
        >::default();
        for (_, pattern) in shape_puzzle.data.index_to_semantic_coordinate.iter() {
            shape_to_offsets.push(get_phase2_shape_offsets(&pattern.shape));
        }

        let reduced_generator_moves = move_list_from_vec(vec!["U_SQ_3", "D_SQ_3", "/"]);

        let edges_puzzle = {
            let start_pattern = square1_edges_kpattern()
                .apply_alg(&parse_alg!("(-2, 0)"))
                .unwrap();
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
                .unwrap();
            PhaseCoordinatePuzzle::<KPuzzle, Phase2CornersCoordinate>::new(
                kpuzzle.clone(),
                start_pattern,
                reduced_generator_moves,
            )
        };

        Self {
            shape_puzzle,
            shape_to_offsets,
            edges_puzzle,
            corners_puzzle,
        }
    }

    // TODO: make this more elegant
    // TODO: report errors for invalid patterns
    /// Currently assumes the input is square-square with left equator solved.
    pub fn full_pattern_to_phase_coordinate(
        &self,
        pattern: &KPattern,
    ) -> Result<Square1Phase2TripleCoordinate, PhaseCoordinateConversionError> {
        let offsets = get_phase2_shape_offsets(pattern);

        let Ok(shape) = self.shape_puzzle.full_pattern_to_phase_coordinate(pattern) else {
            return Err(PhaseCoordinateConversionError::InvalidSemanticCoordinate);
        };
        let Ok(edges) = self.edges_puzzle.full_pattern_to_phase_coordinate(
            &pattern
                .apply_alg(&square1_tuple(
                    offsets.edges_amount_U,
                    offsets.edges_amount_D,
                ))
                .unwrap(),
        ) else {
            return Err(PhaseCoordinateConversionError::InvalidSemanticCoordinate);
        };
        let Ok(corners) = self.corners_puzzle.full_pattern_to_phase_coordinate(
            &pattern
                .apply_alg(&square1_tuple(
                    offsets.corners_amount_U,
                    offsets.corners_amount_D,
                ))
                .unwrap(),
        ) else {
            return Err(PhaseCoordinateConversionError::InvalidSemanticCoordinate);
        };
        Ok(Square1Phase2TripleCoordinate {
            shape,
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
        self.shape_puzzle.move_order(r#move)
    }

    fn puzzle_transformation_from_move(
        &self,
        r#move: &cubing::alg::Move,
    ) -> Result<Self::Transformation, cubing::kpuzzle::InvalidAlgError> {
        Ok(Square1Phase2Transformation(
            self.shape_puzzle.puzzle_transformation_from_move(r#move)?,
        ))
    }

    fn do_moves_commute(
        &self,
        move1_info: &MoveTransformationInfo<Self>,
        move2_info: &MoveTransformationInfo<Self>,
    ) -> bool {
        let move1_info = self
            .shape_puzzle
            .data
            .search_generators_for_phase_coordinate_puzzle
            .by_move
            .get(&move1_info.r#move)
            .expect("TODO: invalid move lookup?");
        let move2_info = self
            .shape_puzzle
            .data
            .search_generators_for_phase_coordinate_puzzle
            .by_move
            .get(&move2_info.r#move)
            .expect("TODO: invalid move lookup?");
        self.shape_puzzle.do_moves_commute(move1_info, move2_info)
    }

    fn pattern_apply_transformation(
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Option<Self::Pattern> {
        // TODO: write down an explanation of the math.
        // TODO: cache the sharp math in a type-safe table instead of doing it every time.

        let offsets = self.shape_to_offsets.at(pattern.shape);

        let phase2_move = Square1Phase2Move::from(transformation_to_apply.0);

        let edges = {
            let edges_transformation = match phase2_move {
                Square1Phase2Move::U_SQ_(amount) => {
                    Square1Phase2Move::U_SQ_((amount - offsets.edges_amount_U).rem_euclid(12))
                }
                Square1Phase2Move::D_SQ_(amount) => {
                    Square1Phase2Move::D_SQ_((amount - offsets.edges_amount_D).rem_euclid(12))
                }
                Square1Phase2Move::SLASH => phase2_move,
            }
            .to_edge_or_corner_transformation();

            match edges_transformation {
                Some(edges_transformation) => self
                    .edges_puzzle
                    .pattern_apply_transformation(&pattern.edges, &edges_transformation)
                    .unwrap_or(pattern.edges),
                None => pattern.edges,
            }
        };

        let corners = {
            let corners_transformation = match phase2_move {
                Square1Phase2Move::U_SQ_(amount) => {
                    Square1Phase2Move::U_SQ_((amount - offsets.corners_amount_U).rem_euclid(12))
                }
                Square1Phase2Move::D_SQ_(amount) => {
                    Square1Phase2Move::D_SQ_((amount - offsets.corners_amount_D).rem_euclid(12))
                }
                Square1Phase2Move::SLASH => phase2_move,
            }
            .to_edge_or_corner_transformation();

            match corners_transformation {
                Some(corners_transformation) => self
                    .corners_puzzle
                    .pattern_apply_transformation(&pattern.corners, &corners_transformation)
                    .unwrap_or(pattern.corners),
                None => pattern.corners,
            }
        };

        Some(Self::Pattern {
            shape: self
                .shape_puzzle
                .pattern_apply_transformation(&pattern.shape, &transformation_to_apply.0)?,
            edges,
            corners,
        })
    }

    fn pattern_apply_transformation_into(
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    ) -> bool {
        let Some(pattern) = self.pattern_apply_transformation(pattern, transformation_to_apply)
        else {
            return false;
        };
        into_pattern.shape = pattern.shape;
        into_pattern.edges = pattern.edges;
        into_pattern.corners = pattern.corners;
        true
    }
}

#[derive(Clone)]
pub struct Square1Phase2PruneTable {
    tpuzzle: Square1Phase2Puzzle,
}

impl PruneTable<Square1Phase2Puzzle> for Square1Phase2PruneTable {
    fn new(
        tpuzzle: Square1Phase2Puzzle,
        _search_api_data: std::sync::Arc<
            crate::_internal::search::idf_search::idf_search::IDFSearchAPIData<Square1Phase2Puzzle>,
        >,
        _search_logger: std::sync::Arc<crate::_internal::search::search_logger::SearchLogger>,
        _min_size: Option<usize>,
    ) -> Self {
        Self { tpuzzle }
    }

    fn lookup(&self, pattern: &<Square1Phase2Puzzle as SemiGroupActionPuzzle>::Pattern) -> Depth {
        let shape_depth = *self
            .tpuzzle
            .shape_puzzle
            .data
            .exact_prune_table
            .at(pattern.shape);
        let edges_depth = *self
            .tpuzzle
            .edges_puzzle
            .data
            .exact_prune_table
            .at(pattern.edges);
        let corners_depth = *self
            .tpuzzle
            .corners_puzzle
            .data
            .exact_prune_table
            .at(pattern.corners);
        max(shape_depth, max(edges_depth, corners_depth))
    }

    fn extend_for_search_depth(&mut self, _search_depth: Depth, _approximate_num_entries: usize) {
        // no-p
    }
}

#[derive(Clone)]
pub struct Square1Phase2SearchAdaptations {}

/// Explicitly specifies search adaptations for [`Square1Phase2Puzzle`].
impl SearchAdaptations<Square1Phase2Puzzle> for Square1Phase2SearchAdaptations {
    type PatternValidityChecker = AlwaysValid;
    type PruneTable = Square1Phase2PruneTable;
    type RecursionFilter = RecursionFilterNoOp;
}

impl DefaultSearchAdaptations<Square1Phase2Puzzle> for Square1Phase2Puzzle {
    type Adaptations = Square1Phase2SearchAdaptations;
}
