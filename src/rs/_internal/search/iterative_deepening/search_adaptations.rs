use std::sync::Arc;

use crate::_internal::{
    canonical_fsm::search_generators::MoveTransformationInfo,
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::{filter::filtering_decision::FilteringDecision, prune_table_trait::Depth},
};

use super::solution_moves::SolutionMoves;

#[derive(Clone)]
#[allow(clippy::type_complexity)] // TODO
pub struct StoredSearchAdaptations<TPuzzle: SemiGroupActionPuzzle> {
    // TODO: `HashPruneTable` doesn't call `filter_move_transformation_fn`.
    pub filter_move_transformation_fn:
        Option<Arc<dyn Fn(&MoveTransformationInfo<TPuzzle>, Depth) -> FilteringDecision>>,
    pub filter_pattern_fn: Option<Arc<dyn Fn(&TPuzzle::Pattern) -> FilteringDecision>>,
}

impl<TPuzzle: SemiGroupActionPuzzle> Default for StoredSearchAdaptations<TPuzzle> {
    fn default() -> Self {
        Self {
            filter_move_transformation_fn: None,
            filter_pattern_fn: None,
        }
    }
}

#[allow(clippy::type_complexity)] // TODO
pub struct IndividualSearchAdaptations<TPuzzle: SemiGroupActionPuzzle> {
    pub filter_search_solution_fn:
        Option<Arc<dyn Fn(&TPuzzle::Pattern, &SolutionMoves) -> FilteringDecision>>,
}

impl<TPuzzle: SemiGroupActionPuzzle> Default for IndividualSearchAdaptations<TPuzzle> {
    fn default() -> Self {
        Self {
            filter_search_solution_fn: Default::default(),
        }
    }
}

// TODO
unsafe impl<TPuzzle: SemiGroupActionPuzzle> Send for StoredSearchAdaptations<TPuzzle> {}
// TODO
unsafe impl<TPuzzle: SemiGroupActionPuzzle> Sync for StoredSearchAdaptations<TPuzzle> {}

impl<TPuzzle: SemiGroupActionPuzzle> StoredSearchAdaptations<TPuzzle> {
    pub fn filter_move_transformation(
        &self,
        candidate_move_transformation_info: &MoveTransformationInfo<TPuzzle>,
        remaining_depth: Depth,
    ) -> FilteringDecision {
        if let Some(filter_move_transformation_fn) = &self.filter_move_transformation_fn {
            filter_move_transformation_fn(candidate_move_transformation_info, remaining_depth)
        } else {
            FilteringDecision::Accept
        }
    }

    pub fn filter_pattern(&self, candidate_pattern: &TPuzzle::Pattern) -> FilteringDecision {
        if let Some(filter_pattern_fn) = &self.filter_pattern_fn {
            filter_pattern_fn(candidate_pattern)
        } else {
            FilteringDecision::Accept
        }
    }
}

impl<TPuzzle: SemiGroupActionPuzzle> IndividualSearchAdaptations<TPuzzle> {
    pub fn filter_search_solution(
        &self,
        candidate_solution_pattern: &TPuzzle::Pattern,
        candidate_solution_moves: &SolutionMoves,
    ) -> FilteringDecision {
        if let Some(filter_search_solution_fn) = &self.filter_search_solution_fn {
            filter_search_solution_fn(candidate_solution_pattern, candidate_solution_moves)
        } else {
            FilteringDecision::Accept
        }
    }
}
