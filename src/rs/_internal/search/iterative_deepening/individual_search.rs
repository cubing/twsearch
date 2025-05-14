use cubing::alg::Move;
use serde::{Deserialize, Serialize};

use crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle;

use super::{
    super::{prune_table_trait::Depth, recursive_work_tracker::RecursiveWorkTracker},
    continuation_condition::ContinuationCondition,
    iterative_deepening_search::IterativeDeepeningSearch,
    search_adaptations::IndividualSearchAdaptations,
};

// TODO: right now we return 0 solutions if we blow past this, should we return an explicit error,
// or panic instead?
const MAX_SUPPORTED_SEARCH_DEPTH: Depth = Depth(500); // TODO: increase

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndividualSearchOptions {
    // TODO: this doesn't do anything at the moment.
    pub min_num_solutions: Option<usize>,
    #[serde(rename = "minDepth")] // TODO
    pub min_depth_inclusive: Option<Depth>, // inclusive
    #[serde(rename = "maxDepth")] // TODO
    pub max_depth_exclusive: Option<Depth>, // exclusive
    pub canonical_fsm_pre_moves: Option<Vec<Move>>,
    pub canonical_fsm_post_moves: Option<Vec<Move>>,
    // Recursive calls use modified continuation conditions derived from this.
    // This is called the "root" continuation condition to distinguish it from
    // the recursive ones.
    //
    // TODO: support (de)serialization.
    /// Note that:
    /// - If the depth of (i.e. number of moves in) the condition exceeds `min_depth_inclusive`:
    ///     - The `root_continuation_condition` will be used to set the initial search depth.
    ///     - `min_depth_inclusive` is overwritten (i.e. efffectively ignored).
    /// - If the depth of (i.e. number of moves in) the condition is less than `min_depth_inclusive`.
    ///     - `min_depth_inclusive` will be used to set the initial search depth.
    ///     - The `root_continuation_condition` is overwritten (i.e. efffectively ignored).
    ///
    /// This allows resuming a search from a previous solution by just passing `ContinuationCondition::After(/* previous solution */)` without also passing a min depth.
    /// However, if the continuation condition corresponds to an intermediate search call rather than the base case the min depth must be specified.
    #[serde(skip_serializing, skip_deserializing)]
    pub root_continuation_condition: ContinuationCondition,
}

impl IndividualSearchOptions {
    pub fn get_min_num_solutions(&self) -> usize {
        self.min_num_solutions.unwrap_or(1)
    }
    pub fn get_min_depth(&self) -> Depth {
        self.min_depth_inclusive.unwrap_or(Depth(0))
    }
    pub fn get_max_depth(&self) -> Depth {
        self.max_depth_exclusive
            .unwrap_or(MAX_SUPPORTED_SEARCH_DEPTH)
    }
}

pub(crate) struct IndividualSearchData<TPuzzle: SemiGroupActionPuzzle> {
    pub(crate) search_pattern: TPuzzle::Pattern,
    pub(crate) individual_search_options: IndividualSearchOptions,
    pub(crate) recursive_work_tracker: RecursiveWorkTracker,
    pub(crate) num_solutions_sofar: usize,
    pub(crate) individual_search_adaptations: IndividualSearchAdaptations<TPuzzle>,
}

impl<TPuzzle: SemiGroupActionPuzzle> IndividualSearchData<TPuzzle> {
    /// Note that search is pull-based. You must call `.next()` (or invoke
    /// something that does) on the return value for the search to begine.
    pub fn new(
        search: &mut IterativeDeepeningSearch<TPuzzle>,
        search_pattern: &TPuzzle::Pattern,
        mut individual_search_options: IndividualSearchOptions,
        individual_search_adaptations: IndividualSearchAdaptations<TPuzzle>,
    ) -> Self {
        // TODO: do validation more consisten   tly.
        if let Some(min_depth) = individual_search_options.min_depth_inclusive {
            if min_depth > MAX_SUPPORTED_SEARCH_DEPTH {
                search
                    .api_data
                    .search_logger
                    .write_error("Min depth too large, capping at maximum.");
                individual_search_options.min_depth_inclusive = Some(MAX_SUPPORTED_SEARCH_DEPTH);
            }
        }
        if let Some(max_depth) = individual_search_options.max_depth_exclusive {
            if max_depth > MAX_SUPPORTED_SEARCH_DEPTH {
                search
                    .api_data
                    .search_logger
                    .write_error("Max depth too large, capping at maximum.");
                individual_search_options.max_depth_exclusive = Some(MAX_SUPPORTED_SEARCH_DEPTH);
            }
        }

        let search_pattern = search_pattern.clone();

        Self {
            search_pattern,
            individual_search_options,
            recursive_work_tracker: RecursiveWorkTracker::new(
                "Search".to_owned(),
                search.api_data.search_logger.clone(),
            ),
            num_solutions_sofar: 0,
            individual_search_adaptations,
        }
    }
}
