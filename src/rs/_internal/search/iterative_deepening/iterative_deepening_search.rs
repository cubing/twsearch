use std::{cmp::max, sync::Arc};

use crate::_internal::{
    canonical_fsm::{
        canonical_fsm::{
            CanonicalFSM, CanonicalFSMConstructionOptions, CanonicalFSMState,
            CANONICAL_FSM_START_STATE,
        },
        search_generators::SearchGenerators,
    },
    cli::args::MetricEnum,
    errors::SearchError,
    puzzle_traits::puzzle_traits::{HashablePatternPuzzle, SemiGroupActionPuzzle},
    search::{
        hash_prune_table::{HashPruneTable, HashPruneTableSizeBounds},
        pattern_stack::PatternStack,
        prune_table_trait::PruneTable,
    },
};
use cubing::{
    alg::{Alg, Move},
    kpuzzle::KPuzzle,
};

use super::{
    super::{prune_table_trait::Depth, search_logger::SearchLogger},
    continuation_condition::ContinuationCondition,
    individual_search::{IndividualSearchData, IndividualSearchOptions},
    search_adaptations::{IndividualSearchAdaptations, StoredSearchAdaptations},
    solution_moves::{alg_to_moves, SolutionMoves},
};

// TODO: use https://doc.rust-lang.org/std/ops/enum.ControlFlow.html as a wrapper instead?
#[allow(clippy::enum_variant_names)]
enum SearchRecursionResult {
    ContinueSearchingDefault,
    ContinueSearchingExcludingCurrentMoveClass,
    FoundSolution(Alg),
}

pub struct IterativeDeepeningSearchCursor<'a, TPuzzle: SemiGroupActionPuzzle = KPuzzle> {
    search: &'a mut IterativeDeepeningSearch<TPuzzle>,
    individual_search_data: IndividualSearchData<TPuzzle>,
}

impl<TPuzzle: SemiGroupActionPuzzle> Iterator for IterativeDeepeningSearchCursor<'_, TPuzzle> {
    type Item = Alg;

    fn next(&mut self) -> Option<Alg> {
        self.search
            .search_internal(&mut self.individual_search_data)
    }
}

pub struct OwnedIterativeDeepeningSearchCursor<TPuzzle: SemiGroupActionPuzzle = KPuzzle> {
    search: IterativeDeepeningSearch<TPuzzle>,
    individual_search_data: IndividualSearchData<TPuzzle>,
}

impl<TPuzzle: SemiGroupActionPuzzle> Iterator for OwnedIterativeDeepeningSearchCursor<TPuzzle> {
    type Item = Alg;

    fn next(&mut self) -> Option<Alg> {
        self.search
            .search_internal(&mut self.individual_search_data)
    }
}

pub struct ImmutableSearchData<TPuzzle: SemiGroupActionPuzzle> {
    pub tpuzzle: TPuzzle,
    pub search_generators: SearchGenerators<TPuzzle>,
    pub canonical_fsm: CanonicalFSM<TPuzzle>, // TODO: move this into `SearchAdaptations`
    pub target_patterns: Vec<TPuzzle::Pattern>,
    pub search_logger: Arc<SearchLogger>,
}

#[derive(Default)]
pub struct ImmutableSearchDataConstructionOptions {
    pub search_logger: Arc<SearchLogger>,
    pub canonical_fsm_construction_options: CanonicalFSMConstructionOptions,
}

impl<TPuzzle: SemiGroupActionPuzzle> ImmutableSearchData<TPuzzle> {
    pub fn try_from_common_options(
        tpuzzle: TPuzzle,
        search_generators: SearchGenerators<TPuzzle>, // TODO: turn this back into `Generators`
        target_patterns: Vec<TPuzzle::Pattern>,
        options: ImmutableSearchDataConstructionOptions,
    ) -> Result<Self, SearchError> {
        let canonical_fsm = CanonicalFSM::try_new(
            // TODO: avoid clones
            tpuzzle.clone(),
            search_generators.clone(),
            options.canonical_fsm_construction_options,
        )
        .map_err(|e| SearchError {
            description: e.to_string(),
        })?;

        Ok(ImmutableSearchData {
            search_generators,
            canonical_fsm,
            tpuzzle: tpuzzle.clone(),
            target_patterns,
            search_logger: options.search_logger.clone(),
        })
    }

    // Figure out an ergonimic way to remove this.
    pub fn try_from_common_options_with_auto_search_generators(
        tpuzzle: TPuzzle,
        generator_moves: Vec<Move>,
        target_patterns: Vec<TPuzzle::Pattern>,
        options: ImmutableSearchDataConstructionOptions,
    ) -> Result<Self, SearchError> {
        let search_generators =
            SearchGenerators::try_new(&tpuzzle, generator_moves.clone(), &MetricEnum::Hand, false)
                .unwrap();
        Self::try_from_common_options(tpuzzle, search_generators, target_patterns, options)
    }
}

/// For information on [`StoredSearchAdaptations`], see the documentation for that trait.
pub struct IterativeDeepeningSearch<TPuzzle: SemiGroupActionPuzzle = KPuzzle> {
    pub immutable_search_data: Arc<ImmutableSearchData<TPuzzle>>,
    pub stored_search_adaptations: StoredSearchAdaptations<TPuzzle>,
    // We require a prune table to avoid accidentally constructing a super slow search. The caller can explicitly pass in a useless prune table if they want.
    pub prune_table: Box<dyn PruneTable<TPuzzle>>,
}

pub struct IterativeDeepeningSearchConstructionOptions {
    pub search_logger: Arc<SearchLogger>,
    pub metric: MetricEnum,
    pub random_start: bool,
    pub min_prune_table_size: Option<usize>,
    pub max_prune_table_size: Option<usize>,
    pub canonical_fsm_construction_options: CanonicalFSMConstructionOptions,
}

impl Default for IterativeDeepeningSearchConstructionOptions {
    fn default() -> Self {
        Self {
            search_logger: Default::default(),
            metric: MetricEnum::Hand,
            random_start: Default::default(),
            min_prune_table_size: Default::default(),
            max_prune_table_size: Default::default(),
            canonical_fsm_construction_options: Default::default(),
        }
    }
}

// TODO: this is needed because the struct directly owns the prune table.
unsafe impl<TPuzzle: SemiGroupActionPuzzle> Send for IterativeDeepeningSearch<TPuzzle> {}
// TODO: this is needed because the struct directly owns the prune table.
unsafe impl<TPuzzle: SemiGroupActionPuzzle> Sync for IterativeDeepeningSearch<TPuzzle> {}

impl<TPuzzle: SemiGroupActionPuzzle + HashablePatternPuzzle + 'static>
    IterativeDeepeningSearch<TPuzzle>
{
    // Shim for the old KPuzzle
    /// Constructs and populates `search_adaptations.prune_table` if it is not populated.
    pub fn new_with_hash_prune_table<T: Into<Arc<ImmutableSearchData<TPuzzle>>>>(
        immutable_search_data: T,
        stored_search_adaptations: StoredSearchAdaptations<TPuzzle>,
        hash_prune_table_size_bounds: HashPruneTableSizeBounds,
    ) -> IterativeDeepeningSearch<TPuzzle> {
        let immutable_search_data = immutable_search_data.into();
        let prune_table = Box::new(HashPruneTable::new(
            immutable_search_data.clone(),
            stored_search_adaptations.clone(),
            hash_prune_table_size_bounds,
        ));
        Self::new(
            immutable_search_data,
            stored_search_adaptations,
            prune_table,
        )
    }
}

impl<TPuzzle: SemiGroupActionPuzzle> IterativeDeepeningSearch<TPuzzle> {
    pub fn new<T: Into<Arc<ImmutableSearchData<TPuzzle>>>>(
        immutable_search_data: T,
        stored_search_adaptations: StoredSearchAdaptations<TPuzzle>,
        prune_table: Box<dyn PruneTable<TPuzzle>>,
    ) -> Self {
        Self {
            immutable_search_data: immutable_search_data.into(),
            stored_search_adaptations,
            prune_table,
        }
    }

    /// Note that search is pull-based. You must call `.next()` on the return
    /// value (or invoke something that does) for the search to begin/continue.
    pub fn search<'a>(
        &'a mut self,
        search_pattern: &TPuzzle::Pattern,
        individual_search_options: IndividualSearchOptions,
        individual_search_adaptations: IndividualSearchAdaptations<TPuzzle>,
    ) -> IterativeDeepeningSearchCursor<'a, TPuzzle> {
        let individual_search_data = IndividualSearchData::new(
            self,
            search_pattern,
            individual_search_options,
            individual_search_adaptations,
        );
        IterativeDeepeningSearchCursor {
            search: self,
            individual_search_data,
        }
    }

    /// Returns an an iterator that takes ownership of this
    /// `IterativeDeepeningSearch`.
    ///
    /// This allows creating an iterator that can be returned by itself instead
    /// of being returned together with its `IterativeDeepeningSearch` (which
    /// also involves careful lifetime annotations).
    ///
    /// Note that search is pull-based. You must call `.next()` (or invoke
    /// something that does) on the return value for the search to begine.
    pub fn owned_search(
        mut self,
        search_pattern: &TPuzzle::Pattern,
        individual_search_options: IndividualSearchOptions,
        individual_search_adaptations: IndividualSearchAdaptations<TPuzzle>,
    ) -> OwnedIterativeDeepeningSearchCursor<TPuzzle> {
        let individual_search_data = IndividualSearchData::new(
            &mut self,
            search_pattern,
            individual_search_options,
            individual_search_adaptations,
        );
        OwnedIterativeDeepeningSearchCursor {
            search: self,
            individual_search_data,
        }
    }

    // TODO: ideally the return should be represented by a fallible iterator (since it can fail caller-provided input deep in the stack).
    fn search_internal(
        &mut self,
        individual_search_data: &mut IndividualSearchData<TPuzzle>,
    ) -> Option<Alg> {
        // TODO: the `min_num_solutions` semantics need a redesign throughout all of `twsearch`.
        // if individual_search_data.num_solutions_sofar
        //     >= individual_search_data
        //         .individual_search_options
        //         .get_min_num_solutions()
        // {
        //     return None;
        // }

        let (initial_search_depth, initial_depth_continuation_condition) = {
            let options_min_depth = individual_search_data
                .individual_search_options
                .get_min_depth();
            let root_continuation_condition: ContinuationCondition = individual_search_data
                .individual_search_options
                .root_continuation_condition
                .clone();
            let root_continuation_depth = root_continuation_condition.min_depth();

            match options_min_depth.cmp(&root_continuation_depth) {
                std::cmp::Ordering::Less => {
                    self.immutable_search_data.search_logger.write_info(&format!(
                        "Increasing initial search depth from {:?} to {:?} based on the root continuation condition: {:?}",
                        options_min_depth, root_continuation_depth, root_continuation_condition
                    ));
                }
                std::cmp::Ordering::Equal => {}
                std::cmp::Ordering::Greater => {
                    if root_continuation_condition != ContinuationCondition::None {
                        self.immutable_search_data.search_logger.write_info(&format!(
                            "Note: the root continuation condition corresponds to an intermediate call rather than the base case because the specified min depth {:?} is larger (this is not an issue if it is expected): {:?}",
                            options_min_depth, root_continuation_condition
                        ));
                    }
                }
            }
            (
                max(options_min_depth, root_continuation_depth),
                root_continuation_condition,
            )
        };
        // Make `initial_depth_continuation_condition` mutable
        let mut initial_depth_continuation_condition = initial_depth_continuation_condition;

        // TODO: combine `KPatternStack` with `SolutionMoves`?
        let mut pattern_stack = PatternStack::new(
            self.immutable_search_data.tpuzzle.clone(),
            individual_search_data.search_pattern.clone(),
        );
        for remaining_depth in *initial_search_depth
            ..*individual_search_data
                .individual_search_options
                .get_max_depth()
        {
            let remaining_depth = Depth(remaining_depth);
            self.immutable_search_data
                .search_logger
                .write_info("----------------");

            self.prune_table.extend_for_search_depth(
                remaining_depth,
                individual_search_data
                    .recursive_work_tracker
                    .estimate_next_level_num_recursive_calls(),
            );
            individual_search_data
                .recursive_work_tracker
                .start_depth(remaining_depth, Some("Starting search…"));
            let initial_state = self
                .apply_optional_fsm_moves(
                    CANONICAL_FSM_START_STATE,
                    &individual_search_data
                        .individual_search_options
                        .canonical_fsm_pre_moves,
                )
                .expect("TODO: invalid canonical FSM pre-moves.");
            let recursion_result = self.recurse(
                individual_search_data,
                &mut pattern_stack,
                initial_state,
                remaining_depth,
                SolutionMoves::default(),
                initial_depth_continuation_condition,
            );
            individual_search_data
                .recursive_work_tracker
                .finish_latest_depth();
            if let SearchRecursionResult::FoundSolution(alg) = recursion_result {
                // TODO: should we avoid writing into `root_continuation_condition`?
                individual_search_data
                    .individual_search_options
                    .root_continuation_condition =
                    ContinuationCondition::After(alg_to_moves(&alg).unwrap());
                return Some(alg);
            }
            initial_depth_continuation_condition = ContinuationCondition::None;
        }

        None
    }

    fn recurse(
        &self,
        individual_search_data: &mut IndividualSearchData<TPuzzle>,
        pattern_stack: &mut PatternStack<TPuzzle>,
        current_state: CanonicalFSMState,
        remaining_depth: Depth,
        solution_moves: SolutionMoves,
        continuation_condition: ContinuationCondition,
    ) -> SearchRecursionResult {
        // eprintln!("========");
        // eprintln!(
        //     "{}",
        //     Alg {
        //         nodes: solution_moves.snapshot_alg_nodes()
        //     }
        // );
        // dbg!(&continuation_condition);
        let mut continuation_condition = continuation_condition;
        let current_pattern = pattern_stack.current_pattern();
        // TODO: apply invalid checks only to intermediate state (i.e. exclude remaining_depth == 0)?
        if self
            .stored_search_adaptations
            .filter_pattern(current_pattern)
            .is_reject()
        {
            return SearchRecursionResult::ContinueSearchingDefault;
        }

        individual_search_data
            .recursive_work_tracker
            .record_recursive_call();
        if remaining_depth == Depth(0) {
            return self.base_case(
                individual_search_data,
                current_pattern,
                current_state,
                solution_moves,
                continuation_condition,
            );
        }
        let prune_table_depth = self.prune_table.lookup(current_pattern);
        if prune_table_depth > remaining_depth + Depth(1) {
            return SearchRecursionResult::ContinueSearchingExcludingCurrentMoveClass;
        }
        if prune_table_depth > remaining_depth {
            return SearchRecursionResult::ContinueSearchingDefault;
        }

        for (move_class_index, move_transformation_multiples) in self
            .immutable_search_data
            .search_generators
            .by_move_class
            .iter()
        {
            let Some(next_state) = self
                .immutable_search_data
                .canonical_fsm
                .next_state(current_state, move_class_index)
            else {
                continue;
            };

            for move_transformation_info in move_transformation_multiples {
                if continuation_condition != ContinuationCondition::None {
                    self.immutable_search_data
                        .search_logger
                        .write_extra(&format!(
                            "{} → {} ? ({:?})",
                            Alg {
                                nodes: solution_moves.snapshot_alg_nodes()
                            },
                            &move_transformation_info.r#move.to_string(),
                            &continuation_condition,
                        ));
                }
                // We check the continuation condition here so that we can
                // resume a search from a continuation condition even if the
                // move is not an accepted by the move transformation filter.
                // TDOO: does this check impact perf?
                let Some(recursive_continuation_condition) =
                    continuation_condition.recurse(&move_transformation_info.r#move)
                else {
                    continue;
                };
                // If we made it here, we're off the starting blocks.
                continuation_condition = ContinuationCondition::None;

                if self
                    .stored_search_adaptations
                    .filter_move_transformation(move_transformation_info, remaining_depth)
                    .is_reject()
                {
                    continue;
                }

                if !pattern_stack.push(&move_transformation_info.transformation) {
                    continue;
                }

                let recursive_result = self.recurse(
                    individual_search_data,
                    pattern_stack,
                    next_state,
                    remaining_depth - Depth(1),
                    (&solution_moves.push(&move_transformation_info.r#move)).into(),
                    recursive_continuation_condition,
                );
                // eprintln!("←←←←←←←←");
                pattern_stack.pop();

                match recursive_result {
                    SearchRecursionResult::ContinueSearchingDefault => {}
                    SearchRecursionResult::ContinueSearchingExcludingCurrentMoveClass => {
                        break;
                    }
                    SearchRecursionResult::FoundSolution(alg) => {
                        return SearchRecursionResult::FoundSolution(alg)
                    }
                }
            }
        }
        SearchRecursionResult::ContinueSearchingDefault
    }

    // Returns `None` if the moves cannot be applied, else returns the result of applying the moves.
    fn apply_optional_fsm_moves(
        &self,
        start_state: CanonicalFSMState,
        moves: &Option<Vec<Move>>,
    ) -> Option<CanonicalFSMState> {
        let mut current_state = start_state;
        if let Some(moves) = moves {
            for r#move in moves {
                let move_class_index = self
                    .immutable_search_data
                    .search_generators
                    .by_move
                    .get(r#move)
                    .expect("move!")
                    .move_class_index;
                current_state = self
                    .immutable_search_data
                    .canonical_fsm
                    .next_state(current_state, move_class_index)?
            }
        }
        Some(current_state)
    }

    fn base_case(
        &self,
        individual_search_data: &mut IndividualSearchData<TPuzzle>,
        current_pattern: &TPuzzle::Pattern,
        current_state: CanonicalFSMState,
        solution_moves: SolutionMoves,
        continuation_condition: ContinuationCondition,
    ) -> SearchRecursionResult {
        match continuation_condition {
            ContinuationCondition::None => {}
            ContinuationCondition::At(vec) => {
                if !vec.is_empty() {
                    // TODO: this can change if we expand change the base case
                    // code to run on intermediate nodes (which may be important
                    // for search with non-uniform metrics).
                    self.immutable_search_data.search_logger.write_warning("Encountered a non-empty `ContinuationCondition::At` during a base case. This could indicate a bug in the calling code.");
                    return SearchRecursionResult::ContinueSearchingDefault;
                }
            }
            ContinuationCondition::After(_) => {
                return SearchRecursionResult::ContinueSearchingDefault
            }
        }

        if !self.is_target_pattern(current_pattern) {
            return SearchRecursionResult::ContinueSearchingDefault;
        }

        if individual_search_data
            .individual_search_adaptations
            .filter_search_solution(current_pattern, &solution_moves)
            .is_reject()
        {
            return SearchRecursionResult::ContinueSearchingDefault;
        }

        if self
            .apply_optional_fsm_moves(
                current_state,
                &individual_search_data
                    .individual_search_options
                    .canonical_fsm_post_moves,
            )
            .is_none()
        {
            self.immutable_search_data
                .search_logger
                .write_info(&format!(
                    "Rejecting potential solution for invalid end moves: {}",
                    Alg::from(&solution_moves)
                ));
            return SearchRecursionResult::ContinueSearchingDefault;
        }

        let alg = Alg::from(&solution_moves);
        individual_search_data.num_solutions_sofar += 1;
        SearchRecursionResult::FoundSolution(alg)
    }

    fn is_target_pattern(&self, current_pattern: &TPuzzle::Pattern) -> bool {
        // TODO: use a hash set instead (for when there is more than 1 target pattern)
        self.immutable_search_data
            .target_patterns
            .contains(current_pattern)
    }
}
