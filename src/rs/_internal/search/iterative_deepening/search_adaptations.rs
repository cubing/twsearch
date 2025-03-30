use std::sync::Arc;

use crate::_internal::{
    canonical_fsm::search_generators::MoveTransformationInfo,
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::{filter::filtering_decision::FilteringDecision, prune_table_trait::Depth},
};

use super::{super::prune_table_trait::PruneTable, iterative_deepening_search::SolutionMoves};

// TODO: get rid of the need for this
#[derive(Clone)]
#[allow(clippy::type_complexity)] // TODO
pub struct StoredSearchAdaptationsWithoutPruneTable<TPuzzle: SemiGroupActionPuzzle> {
    pub filter_transformation_fn:
        Option<Arc<dyn Fn(&MoveTransformationInfo<TPuzzle>, Depth) -> FilteringDecision>>,
    pub filter_pattern_fn: Option<Arc<dyn Fn(&TPuzzle::Pattern) -> FilteringDecision>>,
}

#[allow(clippy::type_complexity)] // TODO
pub struct StoredSearchAdaptations<TPuzzle: SemiGroupActionPuzzle> {
    // We require a prune table to avoid accidentally constructing a super slow search. The caller can explicitly pass in a useless prune table if they want.
    pub prune_table: Box<dyn PruneTable<TPuzzle>>,
    // TODO: `HashPruneTable` doesn't call `filter_transformation_fn`.
    pub filter_transformation_fn:
        Option<Arc<dyn Fn(&MoveTransformationInfo<TPuzzle>, Depth) -> FilteringDecision>>,
    pub filter_pattern_fn: Option<Arc<dyn Fn(&TPuzzle::Pattern) -> FilteringDecision>>,
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
    pub fn filter_transformation(
        &self,
        candidate_move_transformation_info: &MoveTransformationInfo<TPuzzle>,
        remaining_depth: Depth,
    ) -> FilteringDecision {
        if let Some(filter_transformation_fn) = &self.filter_transformation_fn {
            filter_transformation_fn(candidate_move_transformation_info, remaining_depth)
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

impl<TPuzzle: SemiGroupActionPuzzle> StoredSearchAdaptationsWithoutPruneTable<TPuzzle> {
    // TODO: Remove this implementation
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
