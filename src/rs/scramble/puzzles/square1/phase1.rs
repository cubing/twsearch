use std::{fmt::Debug, ops::Range, sync::Arc};

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
            coordinates::{
                graph_enumerated_derived_pattern_puzzle::GraphEnumeratedDerivedPatternPuzzle,
                graph_enumerated_derived_pattern_puzzle_prune_table::GraphEnumeratedDerivedPatternPuzzlePruneTable,
                pattern_deriver::PatternDeriver,
            },
            filter::filtering_decision::FilteringDecision,
            iterative_deepening::{
                search_adaptations::{IndividualSearchAdaptations, StoredSearchAdaptations},
                solution_moves::SolutionMoves,
            },
            mask_pattern::apply_mask,
            prune_table_trait::Depth,
        },
    },
    scramble::parity::BasicParity,
};

use super::{
    super::definitions::square1_square_square_shape_kpattern, parity::bandaged_wedge_parity,
    square1_shape_traversal_filter::shape_traversal_filter_pattern,
};

use lazy_static::lazy_static;

// Note that this entire struct consists of single coordinate.
// The fields themselves are more like "subcoordinates" rather than coordinates in themselves.
// TODO: Implement automatic coordinate composition?
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) struct Square1Phase1Pattern {
    masked_shape_pattern: KPattern,
    parity: BasicParity,
}

#[derive(Clone, Debug)]
pub(crate) struct Square1Phase1PatternDeriver {}

impl PatternDeriver for Square1Phase1PatternDeriver {
    type DerivedPattern = Square1Phase1Pattern;

    fn derive_pattern(&self, source_puzzle_pattern: &KPattern) -> Option<Self::DerivedPattern> {
        let phase_mask = &square1_square_square_shape_kpattern(); // TODO: Store this with the coordinate lookup?
        let Ok(masked_pattern) = apply_mask(source_puzzle_pattern, phase_mask) else {
            panic!("Mask application failed");
        };

        // TODO: this isn't a full validity check for scramble positions.
        if shape_traversal_filter_pattern(&masked_pattern).is_reject() {
            return None;
        }

        let parity = bandaged_wedge_parity(source_puzzle_pattern);
        Some(Square1Phase1Pattern {
            masked_shape_pattern: masked_pattern,
            parity,
        })
    }
}
pub(crate) type Square1Phase1Puzzle =
    GraphEnumeratedDerivedPatternPuzzle<KPuzzle, Square1Phase1PatternDeriver>;

// TODO: allow flipping this depending on whether this is for a scramble (backwards) or a solution (forwards)?
const D_SQ_MOVE_RESTRICTED_RANGE: Range<i32> = -3..3;

// This is exported so it can be reused by phase 2.
#[allow(non_snake_case)]
pub fn restrict_D_move(
    move_transformation_info: &MoveTransformationInfo<Square1Phase1Puzzle>,
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

fn filter_move_transformation(
    move_transformation_info: &MoveTransformationInfo<Square1Phase1Puzzle>,
    _remaining_depth: Depth,
) -> FilteringDecision {
    match restrict_D_move(move_transformation_info) {
        true => FilteringDecision::Accept,
        false => FilteringDecision::Reject,
    }
}

fn filter_search_solution(
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

// TODO: we currently take `square1_phase1_puzzle` as an argument to keep construction DRY. There's probably a better way to do this.
pub(crate) fn square1_phase1_stored_search_adaptations(
    square1_phase1_puzzle: GraphEnumeratedDerivedPatternPuzzle<
        KPuzzle,
        Square1Phase1PatternDeriver,
    >,
) -> StoredSearchAdaptations<Square1Phase1Puzzle> {
    let prune_table = Box::new(GraphEnumeratedDerivedPatternPuzzlePruneTable::new(
        square1_phase1_puzzle,
    ));
    StoredSearchAdaptations {
        prune_table,
        filter_move_transformation_fn: Some(Arc::new(Box::new(filter_move_transformation))),
        filter_pattern_fn: None,
    }
}

// TODO: we currently take `square1_phase1_puzzle` as an argument to keep construction DRY. There's probably a better way to do this.
pub(crate) fn square1_phase1_individual_search_adaptations(
) -> IndividualSearchAdaptations<Square1Phase1Puzzle> {
    IndividualSearchAdaptations {
        filter_search_solution_fn: Some(Arc::new(Box::new(filter_search_solution))),
    }
}
