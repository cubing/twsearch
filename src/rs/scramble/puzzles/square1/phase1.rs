use std::{fmt::Debug, ops::Range};

use cubing::{
    alg::{parse_move, Move},
    kpuzzle::{KPattern, KPuzzle},
};

use crate::{
    _internal::{
        canonical_fsm::{
            move_class_mask::MoveClassIndex, search_generators::MoveTransformationInfo,
        },
        puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
        search::{
            coordinates::graph_enumerated_derived_pattern_puzzle::{
                DerivedPattern, DerivedPatternPuzzlePruneTable, GraphEnumeratedDerivedPatternPuzzle,
            },
            filter::{
                filtering_decision::FilteringDecision,
                pattern_traversal_filter_trait::{
                    PatternTraversalFilter, PatternTraversalFilterNoOp,
                },
                search_solution_filter_trait::SearchSolutionFilter,
                transformation_traversal_filter_trait::TransformationTraversalFilter,
            },
            iterative_deepening::{
                iterative_deepening_search::SolutionMoves, search_adaptations::SearchAdaptations,
            },
            mask_pattern::apply_mask,
            prune_table_trait::Depth,
        },
    },
    scramble::randomize::BasicParity,
};

use super::{
    super::definitions::square1_square_square_shape_kpattern, parity::bandaged_wedge_parity,
    square1_scramble_finder::Square1SearchPhase,
    square1_shape_traversal_filter::Square1ShapeTraversalFilter,
};

use lazy_static::lazy_static;

// Note that this entire struct consists of single coordinate.
// The fields themselves are more like "subcoordinates" rather than coordinates in themselves.
// TODO: Implement automatic coordinate composition?
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct Square1Phase1Coordinate {
    masked_pattern: KPattern,
    parity: BasicParity,
}

impl DerivedPattern<KPuzzle> for Square1Phase1Coordinate {
    fn derived_pattern_name() -> &'static str {
        "U/D shape (Square-1 → phase 1)"
    }

    fn try_new(_kpuzzle: &KPuzzle, full_pattern: &KPattern) -> Option<Self> {
        let phase_mask = &square1_square_square_shape_kpattern(); // TODO: Store this with the coordinate lookup?
        let Ok(masked_pattern) = apply_mask(full_pattern, phase_mask) else {
            panic!("Mask application failed");
        };

        // TODO: this isn't a full validity check for scramble positions.
        if Square1ShapeTraversalFilter::filter_pattern(&masked_pattern).is_reject() {
            return None;
        }

        let parity = bandaged_wedge_parity(full_pattern);
        Some(Self {
            masked_pattern,
            parity,
        })
    }
}

pub(crate) type Square1Phase1Puzzle =
    GraphEnumeratedDerivedPatternPuzzle<KPuzzle, Square1Phase1Coordinate>;

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

impl TransformationTraversalFilter<Square1Phase1Puzzle> for Square1Phase1Puzzle {
    fn filter_transformation(
        move_transformation_info: &MoveTransformationInfo<Square1Phase1Puzzle>,
        _remaining_depth: Depth,
    ) -> FilteringDecision {
        match restrict_D_move(move_transformation_info) {
            true => FilteringDecision::Accept,
            false => FilteringDecision::Reject,
        }
    }
}

impl SearchSolutionFilter<Square1Phase1Puzzle> for Square1Phase1Puzzle {
    fn filter_solution(
        _pattern: &<Square1Phase1Puzzle as SemiGroupActionPuzzle>::Pattern,
        solution_moves: &SolutionMoves,
    ) -> FilteringDecision {
        for r#move in solution_moves.reverse_move_iter() {
            if r#move == parse_move!("/") {
                return FilteringDecision::Accept;
            }
            if r#move.amount > 2 || r#move.amount < 0 {
                return FilteringDecision::Reject;
            }
        }
        FilteringDecision::Accept
    }
}

pub(crate) struct Square1Phase1SearchAdaptations {}

/// Explicitly specifies search adaptations for [`Square1Phase1Puzzle`].
impl SearchAdaptations<Square1Phase1Puzzle> for Square1Phase1SearchAdaptations {
    type PatternTraversalFilter = PatternTraversalFilterNoOp;
    type PruneTable = DerivedPatternPuzzlePruneTable<KPuzzle, Square1Phase1Coordinate>;
    type TransformationTraversalFilter = Square1Phase1Puzzle;
    type SearchSolutionFilter = Square1Phase1Puzzle;
}

impl Square1SearchPhase for Square1Phase1Puzzle {}
