use cubing::{
    alg::{parse_alg, parse_move, Alg, Move},
    kpuzzle::{KPattern, KPuzzle, KPuzzleOrbitInfo},
};
use lazy_static::lazy_static;
use std::cmp::max;

use crate::{
    _internal::{
        canonical_fsm::search_generators::{FlatMoveIndex, MoveTransformationInfo},
        puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
        search::{
            coordinates::{
                graph_enumerated_derived_pattern_puzzle::{
                    DerivedPatternConversionError, DerivedPatternIndex,
                    GraphEnumeratedDerivedPatternPuzzle,
                },
                pattern_deriver::PatternDeriver,
            },
            filter::filtering_decision::FilteringDecision,
            iterative_deepening::search_adaptations::SearchAdaptations,
            mask_pattern::apply_mask,
            prune_table_trait::{Depth, PruneTable},
        },
    },
    scramble::{
        puzzles::{
            definitions::{
                square0_equatorless_kpuzzle, square1_shape_kpattern, square1_unbandaged_kpuzzle,
            },
            square1::wedges::{WedgeType, WEDGE_TYPE_LOOKUP},
        },
        scramble_search::move_list_from_vec,
    },
};

use super::wedges::get_phase2_shape_offsets;

fn phase2_filter_derived_pattern(pattern: &cubing::kpuzzle::KPattern) -> FilteringDecision {
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
            return FilteringDecision::Reject;
        }

        for slot_offset in [3, 6, 9] {
            let offset_value = unsafe {
                pattern
                    .packed_orbit_data()
                    .get_raw_piece_or_permutation_value(orbit_info, slot + slot_offset)
            };
            let offset_wedge_type = &WEDGE_TYPE_LOOKUP[offset_value as usize];

            if wedge_type != offset_wedge_type {
                return FilteringDecision::Reject;
            }
        }
    }

    FilteringDecision::Accept
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct Phase2ShapePattern {
    pub(crate) shape: KPattern,
}

// TODO: avoid the need for this intermediate struct
#[derive(Clone, Debug)]
struct Phase2ShapePatternDeriver {}

impl PatternDeriver<KPuzzle> for Phase2ShapePatternDeriver {
    type DerivedPattern = Phase2ShapePattern;

    fn derive_pattern(
        &self,
        source_puzzle_pattern: &<KPuzzle as SemiGroupActionPuzzle>::Pattern,
    ) -> Option<Self::DerivedPattern> {
        // TODO: this isn't a full validity check for scramble positions.
        if phase2_filter_derived_pattern(source_puzzle_pattern).is_reject() {
            return None;
        }

        let phase_mask = square1_shape_kpattern(); // TODO: Store this with the coordinate lookup?
        let Ok(masked_pattern) = apply_mask(source_puzzle_pattern, phase_mask) else {
            panic!("Mask application failed");
        };

        Some(Phase2ShapePattern {
            shape: masked_pattern,
        })
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct Square0EquatorlessPattern {
    pub(crate) pattern: KPattern,
}

const NUM_SQUARE0_EQUATORLESS_WEDGES: u8 = 8;
impl Square0EquatorlessPattern {
    // TODO: define this as a static method instead and just output a `KPattern` instead of a `Square0EquatorlessCoordinate`? Or ideally add a type-safe way for coordinates to us different masks/conversions into the same coordinates (and tables).
    fn from_edges_pattern(square1_edges_pattern: &KPattern) -> Self {
        lazy_static! {
            static ref orbit_info: &'static KPuzzleOrbitInfo =
                &square1_unbandaged_kpuzzle().data.ordered_orbit_info[0];
        };
        assert_eq!(orbit_info.name.0, "WEDGES"); // TODO: only do this at orbit info time, using the former pattern for retrieving an `OrbitInfo` from a definition.

        let mut square0_equatorless_pattern =
            square0_equatorless_kpuzzle().default_pattern().clone();

        let mut square0_equatorless_i = 0;
        for square1_i in 0..orbit_info.num_pieces {
            let piece = square1_edges_pattern.get_piece(&orbit_info, square1_i);
            let square0_piece = match piece {
                // TODO: unify this with `WEDGE_TYPE_LOOKUP`?
                0 => None,
                1 => None,
                2 => Some(0),
                3 => None,
                4 => None,
                5 => Some(1),
                6 => None,
                7 => None,
                8 => Some(2),
                9 => None,
                10 => None,
                11 => Some(3),
                12 => Some(4),
                13 => None,
                14 => None,
                15 => Some(5),
                16 => None,
                17 => None,
                18 => Some(6),
                19 => None,
                20 => None,
                21 => Some(7),
                22 => None,
                23 => None,
                _ => panic!("Invalid edge piece"),
            };
            if let Some(square0_piece) = square0_piece {
                square0_equatorless_pattern.set_piece(
                    &orbit_info,
                    square0_equatorless_i,
                    square0_piece,
                );
                square0_equatorless_i += 1;
            }
        }
        assert_eq!(square0_equatorless_i, NUM_SQUARE0_EQUATORLESS_WEDGES);
        Self {
            pattern: square0_equatorless_pattern,
        }
    }

    // TODO: define this as a static method instead and just output a `KPattern` instead of a `Square0EquatorlessCoordinate`? Or ideally add a type-safe way for coordinates to us different masks/conversions into the same coordinates (and tables).
    fn from_corners_pattern(square1_corners_pattern: &KPattern) -> Self {
        lazy_static! {
            static ref orbit_info: &'static KPuzzleOrbitInfo =
                &square1_unbandaged_kpuzzle().data.ordered_orbit_info[0];
        };
        assert_eq!(orbit_info.name.0, "WEDGES"); // TODO: only do this at orbit info time, using the former pattern for retrieving an `OrbitInfo` from a definition.

        let mut square0_equatorless_pattern =
            square0_equatorless_kpuzzle().default_pattern().clone();

        let mut square0_equatorless_i = 0;
        for square1_i in 0..orbit_info.num_pieces {
            let piece = square1_corners_pattern.get_piece(&orbit_info, square1_i);
            let square0_piece = match piece {
                // TODO: unify this with `WEDGE_TYPE_LOOKUP`?
                0 => Some(0),
                1 => None,
                2 => None,
                3 => Some(1),
                4 => None,
                5 => None,
                6 => Some(2),
                7 => None,
                8 => None,
                9 => Some(3),
                10 => None,
                11 => None,
                12 => None,
                13 => Some(4),
                14 => None,
                15 => None,
                16 => Some(5),
                17 => None,
                18 => None,
                19 => Some(6),
                20 => None,
                21 => None,
                22 => Some(7),
                23 => None,
                _ => panic!("Invalid corner piece"),
            };
            if let Some(square0_piece) = square0_piece {
                square0_equatorless_pattern.set_piece(
                    &orbit_info,
                    square0_equatorless_i,
                    square0_piece,
                );
                square0_equatorless_i += 1;
            }
        }
        assert_eq!(square0_equatorless_i, NUM_SQUARE0_EQUATORLESS_WEDGES);
        Self {
            pattern: square0_equatorless_pattern,
        }
    }
}

// TODO: avoid the need for this intermediate struct
#[derive(Clone, Debug)]
struct Square0EquatorlessPatternDeriver {}

impl PatternDeriver<KPuzzle> for Square0EquatorlessPatternDeriver {
    type DerivedPattern = Square0EquatorlessPattern;

    fn derive_pattern(
        &self,
        _source_puzzle_pattern: &<KPuzzle as SemiGroupActionPuzzle>::Pattern,
    ) -> Option<Self::DerivedPattern> {
        todo!()
    }

    // fn try_new(_kpuzzle: &KPuzzle, pattern: &KPattern) -> Option<Self> {
    //     Some(Self {
    //         pattern: pattern.clone(),
    //     })
    // }
}

// TODO: generalize this, similar to how `TriplePhaseCoordinatePuzzle` does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Square1Phase2TripleCoordinate {
    shape: DerivedPatternIndex<
        GraphEnumeratedDerivedPatternPuzzle<KPuzzle, Phase2ShapePatternDeriver>,
    >, // TODO: rename to "shape"
    edges: DerivedPatternIndex<
        GraphEnumeratedDerivedPatternPuzzle<KPuzzle, Square0EquatorlessPatternDeriver>,
    >,
    corners: DerivedPatternIndex<
        GraphEnumeratedDerivedPatternPuzzle<KPuzzle, Square0EquatorlessPatternDeriver>,
    >,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Square1Phase2Transformation(FlatMoveIndex); // Index into search generators for the shape.

#[derive(Clone, Debug)]
pub struct Square1Phase2Puzzle {
    shape_puzzle: GraphEnumeratedDerivedPatternPuzzle<KPuzzle, Phase2ShapePatternDeriver>,
    square0_equatorless_puzzle:
        GraphEnumeratedDerivedPatternPuzzle<KPuzzle, Square0EquatorlessPatternDeriver>,
}

#[allow(non_snake_case)]
fn square1_tuple(U_SQ_amount: i32, D_SQ_amount: i32) -> Alg {
    let U_SQ_ = parse_move!("U_SQ_");
    let D_SQ_ = parse_move!("D_SQ_");

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
    fn to_edge_transformation(self) -> Option<FlatMoveIndex> {
        Some(FlatMoveIndex(match self {
            Square1Phase2Move::U_SQ_(0) => return None,
            Square1Phase2Move::U_SQ_(1) => 0,
            Square1Phase2Move::U_SQ_(2) => return None,
            Square1Phase2Move::U_SQ_(3) => 0,
            Square1Phase2Move::U_SQ_(4) => 1,
            Square1Phase2Move::U_SQ_(5) => 0,
            Square1Phase2Move::U_SQ_(6) => 1,
            Square1Phase2Move::U_SQ_(7) => 2,
            Square1Phase2Move::U_SQ_(8) => 1,
            Square1Phase2Move::U_SQ_(9) => 2,
            Square1Phase2Move::U_SQ_(10) => return None,
            Square1Phase2Move::U_SQ_(11) => 2,
            Square1Phase2Move::D_SQ_(0) => return None,
            Square1Phase2Move::D_SQ_(1) => 3,
            Square1Phase2Move::D_SQ_(2) => return None,
            Square1Phase2Move::D_SQ_(3) => 3,
            Square1Phase2Move::D_SQ_(4) => 4,
            Square1Phase2Move::D_SQ_(5) => 3,
            Square1Phase2Move::D_SQ_(6) => 4,
            Square1Phase2Move::D_SQ_(7) => 5,
            Square1Phase2Move::D_SQ_(8) => 4,
            Square1Phase2Move::D_SQ_(9) => 5,
            Square1Phase2Move::D_SQ_(10) => return None,
            Square1Phase2Move::D_SQ_(11) => 5,
            Square1Phase2Move::SLASH => 6,
            _ => panic!("Invalid move"), // TODO: is it faster to just return `None` as the default?
        }))
    }

    // Accepts amounts from `0` to `11` (for `U_SQ_` or `D_SQ_` moves).
    fn to_corner_transformation(self) -> Option<FlatMoveIndex> {
        Some(FlatMoveIndex(match self {
            Square1Phase2Move::U_SQ_(0) => return None,
            Square1Phase2Move::U_SQ_(1) => return None,
            Square1Phase2Move::U_SQ_(2) => 0,
            Square1Phase2Move::U_SQ_(3) => 0,
            Square1Phase2Move::U_SQ_(4) => 0,
            Square1Phase2Move::U_SQ_(5) => 1,
            Square1Phase2Move::U_SQ_(6) => 1,
            Square1Phase2Move::U_SQ_(7) => 1,
            Square1Phase2Move::U_SQ_(8) => 2,
            Square1Phase2Move::U_SQ_(9) => 2,
            Square1Phase2Move::U_SQ_(10) => 2,
            Square1Phase2Move::U_SQ_(11) => return None,
            Square1Phase2Move::D_SQ_(0) => return None,
            Square1Phase2Move::D_SQ_(1) => return None,
            Square1Phase2Move::D_SQ_(2) => 3,
            Square1Phase2Move::D_SQ_(3) => 3,
            Square1Phase2Move::D_SQ_(4) => 3,
            Square1Phase2Move::D_SQ_(5) => 4,
            Square1Phase2Move::D_SQ_(6) => 4,
            Square1Phase2Move::D_SQ_(7) => 4,
            Square1Phase2Move::D_SQ_(8) => 5,
            Square1Phase2Move::D_SQ_(9) => 5,
            Square1Phase2Move::D_SQ_(10) => 5,
            Square1Phase2Move::D_SQ_(11) => return None,
            Square1Phase2Move::SLASH => 6,
            _ => panic!("Invalid move"), // TODO: is it faster to just return `None` as the default?
        }))
    }
}

impl Square1Phase2Puzzle {
    pub fn new() -> Self {
        let kpuzzle = square1_unbandaged_kpuzzle();

        let phase2_shape_pattern_deriver = Phase2ShapePatternDeriver {};

        let shape_puzzle = {
            let full_generator_moves = move_list_from_vec(vec!["U_SQ_", "D_SQ_", "/"]);

            let start_pattern = square1_shape_kpattern()
                .apply_alg(parse_alg!("(0, 0)"))
                .unwrap();
            GraphEnumeratedDerivedPatternPuzzle::<KPuzzle, Phase2ShapePatternDeriver>::new(
                kpuzzle.clone(),
                phase2_shape_pattern_deriver,
                start_pattern,
                full_generator_moves,
            )
        };

        let square0_equatorless_pattern_deriver = Square0EquatorlessPatternDeriver {};

        // TODO: when using `U_SQ_3` and `D_SQ_3` here (and in the def), `cubing.rs` runs into an error. So we use inconsistent moves for now.
        // let reduced_generator_moves = move_list_from_vec(vec!["U_SQ_3", "D_SQ_3", "/"]);
        let reduced_generator_moves = move_list_from_vec(vec!["U_SQ_", "D_SQ_", "/"]);
        let square0_equatorless_puzzle = {
            let kpuzzle = square0_equatorless_kpuzzle();
            GraphEnumeratedDerivedPatternPuzzle::<KPuzzle, Square0EquatorlessPatternDeriver>::new(
                kpuzzle.clone(),
                square0_equatorless_pattern_deriver,
                kpuzzle.default_pattern(),
                reduced_generator_moves,
            )
        };

        Self {
            shape_puzzle,
            square0_equatorless_puzzle,
        }
    }

    // TODO: make this more elegant
    // TODO: report errors for invalid patterns
    /// Currently assumes the input is square-square with left equator solved.
    pub fn full_pattern_to_phase_coordinate(
        &self,
        pattern: &KPattern,
    ) -> Result<Square1Phase2TripleCoordinate, DerivedPatternConversionError> {
        let offsets = get_phase2_shape_offsets(pattern);

        let Ok(shape) = self.shape_puzzle.full_pattern_to_derived_pattern(pattern) else {
            return Err(DerivedPatternConversionError::InvalidDerivedPattern);
        };
        let Ok(edges) = self
            .square0_equatorless_puzzle
            .full_pattern_to_derived_pattern(
                &Square0EquatorlessPattern::from_edges_pattern(
                    &pattern
                        .apply_alg(&square1_tuple(
                            offsets.edges_amount_U,
                            offsets.edges_amount_D,
                        ))
                        .unwrap(),
                )
                .pattern,
            )
        else {
            return Err(DerivedPatternConversionError::InvalidDerivedPattern);
        };
        let Ok(corners) = self
            .square0_equatorless_puzzle
            .full_pattern_to_derived_pattern(
                &Square0EquatorlessPattern::from_corners_pattern(
                    &pattern
                        .apply_alg(&square1_tuple(
                            offsets.corners_amount_U,
                            offsets.corners_amount_D,
                        ))
                        .unwrap(),
                )
                .pattern,
            )
        else {
            return Err(DerivedPatternConversionError::InvalidDerivedPattern);
        };
        let pattern = Square1Phase2TripleCoordinate {
            shape,
            edges,
            corners,
        };
        Ok(pattern)
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
            .search_generators_for_derived_pattern_puzzle
            .by_move
            .get(&move1_info.r#move)
            .expect("TODO: invalid move lookup?");
        let move2_info = self
            .shape_puzzle
            .data
            .search_generators_for_derived_pattern_puzzle
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

        let phase2_move = Square1Phase2Move::from(transformation_to_apply.0);

        let shape = self
            .shape_puzzle
            .pattern_apply_transformation(&pattern.shape, &transformation_to_apply.0)?;

        let edges = {
            let edges_transformation = match phase2_move {
                Square1Phase2Move::U_SQ_(amount) => {
                    Square1Phase2Move::U_SQ_((amount).rem_euclid(12))
                }
                Square1Phase2Move::D_SQ_(amount) => {
                    Square1Phase2Move::D_SQ_((amount).rem_euclid(12))
                }
                Square1Phase2Move::SLASH => phase2_move,
            }
            .to_edge_transformation();

            match edges_transformation {
                Some(edges_transformation) => self
                    .square0_equatorless_puzzle
                    .pattern_apply_transformation(&pattern.edges, &edges_transformation)
                    .unwrap_or(pattern.edges),
                None => pattern.edges,
            }
        };

        let corners = {
            let corners_transformation = match phase2_move {
                Square1Phase2Move::U_SQ_(amount) => {
                    Square1Phase2Move::U_SQ_((amount).rem_euclid(12))
                }
                Square1Phase2Move::D_SQ_(amount) => {
                    Square1Phase2Move::D_SQ_((amount).rem_euclid(12))
                }
                Square1Phase2Move::SLASH => phase2_move,
            }
            .to_corner_transformation();

            match corners_transformation {
                Some(corners_transformation) => self
                    .square0_equatorless_puzzle
                    .pattern_apply_transformation(&pattern.corners, &corners_transformation)
                    .unwrap_or(pattern.corners),
                None => pattern.corners,
            }
        };

        Some(Self::Pattern {
            shape,
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

impl Square1Phase2PruneTable {
    fn new(tpuzzle: Square1Phase2Puzzle) -> Self {
        Self { tpuzzle }
    }
}

impl PruneTable<Square1Phase2Puzzle> for Square1Phase2PruneTable {
    fn lookup(&self, pattern: &<Square1Phase2Puzzle as SemiGroupActionPuzzle>::Pattern) -> Depth {
        let shape_depth = self.tpuzzle.shape_puzzle.data.exact_prune_table[pattern.shape];
        let edges_depth = self
            .tpuzzle
            .square0_equatorless_puzzle
            .data
            .exact_prune_table[pattern.edges];
        let corners_depth = self
            .tpuzzle
            .square0_equatorless_puzzle
            .data
            .exact_prune_table[pattern.corners];
        max(shape_depth, max(edges_depth, corners_depth))
    }

    fn extend_for_search_depth(&mut self, _search_depth: Depth, _approximate_num_entries: usize) {
        // no-op
    }
}

// TODO: we currently take `square1_phase1_puzzle` as an argument to keep construction DRY. There's probably a better way to do this.
pub(crate) fn square1_phase2_search_adaptations(
    square1_phase2_puzzle: Square1Phase2Puzzle,
) -> SearchAdaptations<Square1Phase2Puzzle> {
    let prune_table = Box::new(Square1Phase2PruneTable::new(square1_phase2_puzzle));
    SearchAdaptations {
        prune_table,
        filter_transformation_fn: None,
        filter_pattern_fn: None,
        filter_search_solution_fn: None,
    }
}
