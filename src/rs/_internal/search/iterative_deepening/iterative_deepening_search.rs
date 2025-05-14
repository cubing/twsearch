use std::{cmp::max, fmt::Debug, sync::Arc};

use cubing::{
    alg::{Alg, AlgNode, Move},
    kpuzzle::{KPattern, KPuzzle},
};
use serde::{Deserialize, Serialize};

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
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::{
        hash_prune_table::HashPruneTable, pattern_stack::PatternStack,
        prune_table_trait::LegacyConstructablePruneTable,
    },
};

use super::{
    super::{
        prune_table_trait::Depth, recursive_work_tracker::RecursiveWorkTracker,
        search_logger::SearchLogger,
    },
    search_adaptations::{
        IndividualSearchAdaptations, StoredSearchAdaptations,
        StoredSearchAdaptationsWithoutPruneTable,
    },
    solution_moves::SolutionMoves,
};

// TODO: right now we return 0 solutions if we blow past this, should we return an explicit error,
// or panic instead?
const MAX_SUPPORTED_SEARCH_DEPTH: Depth = Depth(500); // TODO: increase

// TODO: use https://doc.rust-lang.org/std/ops/enum.ControlFlow.html as a wrapper instead?
#[allow(clippy::enum_variant_names)]
enum SearchRecursionResult {
    ContinueSearchingDefault,
    ContinueSearchingExcludingCurrentMoveClass,
    FoundSolution(Alg),
}

pub struct IterativeSearchCursor<'a, TPuzzle: SemiGroupActionPuzzle = KPuzzle> {
    search: &'a mut IterativeDeepeningSearch<TPuzzle>,
    individual_search_data: IndividualSearchData<TPuzzle>,
}

impl<TPuzzle: SemiGroupActionPuzzle> Iterator for IterativeSearchCursor<'_, TPuzzle> {
    type Item = Alg;

    fn next(&mut self) -> Option<Alg> {
        self.search
            .search_internal(&mut self.individual_search_data)
    }
}

pub struct OwnedIterativeSearchCursor<TPuzzle: SemiGroupActionPuzzle = KPuzzle> {
    search: IterativeDeepeningSearch<TPuzzle>,
    individual_search_data: IndividualSearchData<TPuzzle>,
}

impl<TPuzzle: SemiGroupActionPuzzle> Iterator for OwnedIterativeSearchCursor<TPuzzle> {
    type Item = Alg;

    fn next(&mut self) -> Option<Alg> {
        self.search
            .search_internal(&mut self.individual_search_data)
    }
}

// TODO: also handle "before" cases.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ContinuationCondition {
    None,
    // An empty `Vec` in the base case means a solution check shall be performed.
    At(Vec<Move>),
    // An empty `Vec` in the base case means a solution check shall not be performed.
    After(Vec<Move>),
}

impl Default for ContinuationCondition {
    fn default() -> Self {
        Self::None
    }
}

impl ContinuationCondition {
    fn min_depth(&self) -> Depth {
        match self {
            ContinuationCondition::None => Depth(0),
            ContinuationCondition::At(moves) => Depth(moves.len()),
            ContinuationCondition::After(moves) => Depth(moves.len()),
        }
    }
}

pub(crate) fn alg_from_moves(moves: &[Move]) -> Alg {
    let nodes = moves.iter().map(|m| AlgNode::MoveNode(m.clone())).collect();
    Alg { nodes }
}

pub(crate) fn alg_to_moves(alg: &Alg) -> Option<Vec<Move>> {
    let mut moves: Vec<Move> = vec![];
    for alg_node in &alg.nodes {
        let AlgNode::MoveNode(r#move) = alg_node else {
            return None;
        };
        moves.push(r#move.clone());
    }
    Some(moves)
}

impl Debug for ContinuationCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContinuationCondition::None => write!(f, "ContinuationCondition::None"),
            ContinuationCondition::At(moves) => {
                write!(
                    f,
                    "ContinuationCondition::At(parse_alg!({:?}))",
                    alg_from_moves(moves).to_string()
                )
            }
            ContinuationCondition::After(moves) => {
                write!(
                    f,
                    "ContinuationCondition::After(parse_alg!({:?}))",
                    alg_from_moves(moves).to_string()
                )
            }
        }
    }
}

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

struct IndividualSearchData<TPuzzle: SemiGroupActionPuzzle> {
    search_pattern: TPuzzle::Pattern,
    individual_search_options: IndividualSearchOptions,
    recursive_work_tracker: RecursiveWorkTracker,
    num_solutions_sofar: usize,
    individual_search_adaptations: IndividualSearchAdaptations<TPuzzle>,
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

pub struct IterativeDeepeningSearchAPIData<TPuzzle: SemiGroupActionPuzzle> {
    pub search_generators: SearchGenerators<TPuzzle>,
    pub canonical_fsm: CanonicalFSM<TPuzzle>, // TODO: move this into `SearchAdaptations`
    pub tpuzzle: TPuzzle,
    pub target_patterns: Vec<TPuzzle::Pattern>,
    pub search_logger: Arc<SearchLogger>,
}

/// For information on [`StoredSearchAdaptations`], see the documentation for that trait.
pub struct IterativeDeepeningSearch<TPuzzle: SemiGroupActionPuzzle = KPuzzle> {
    pub api_data: Arc<IterativeDeepeningSearchAPIData<TPuzzle>>,
    pub stored_search_adaptations: StoredSearchAdaptations<TPuzzle>,
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

impl IterativeDeepeningSearch<KPuzzle> {
    // Shim for the old KPuzzle
    /// Constructs and populates `search_adaptations.prune_table` if it is not populated.
    pub fn try_new_kpuzzle_with_hash_prune_table_shim(
        tpuzzle: KPuzzle,
        generator_moves: Vec<Move>, // TODO: turn this back into `Generators`
        target_patterns: Vec<KPattern>,
        options: IterativeDeepeningSearchConstructionOptions,
        search_adaptations_without_prune_table: Option<
            StoredSearchAdaptationsWithoutPruneTable<KPuzzle>,
        >,
    ) -> Result<Self, SearchError> {
        Self::try_new_prune_table_construction_shim::<HashPruneTable<KPuzzle>>(
            tpuzzle,
            generator_moves,
            target_patterns,
            options,
            search_adaptations_without_prune_table,
        )
    }
}

impl<TPuzzle: SemiGroupActionPuzzle> IterativeDeepeningSearch<TPuzzle> {
    pub fn try_new_prune_table_construction_shim<
        TPruneTable: LegacyConstructablePruneTable<TPuzzle> + 'static,
    >(
        tpuzzle: TPuzzle,
        generator_moves: Vec<Move>, // TODO: turn this back into `Generators`
        target_patterns: Vec<TPuzzle::Pattern>,
        options: IterativeDeepeningSearchConstructionOptions,
        search_adaptations_without_prune_table: Option<
            StoredSearchAdaptationsWithoutPruneTable<TPuzzle>,
        >,
    ) -> Result<Self, SearchError> {
        let search_logger = options.search_logger.clone();
        let min_prune_table_size = options.min_prune_table_size;
        let max_prune_table_size = options.max_prune_table_size;
        let api_data = Self::legacy_construct_api_data(
            tpuzzle.clone(),
            generator_moves,
            target_patterns,
            options,
        )?;
        let search_adaptations_without_prune_table = match search_adaptations_without_prune_table {
            Some(search_adaptations_without_prune_table) => search_adaptations_without_prune_table,
            None => StoredSearchAdaptationsWithoutPruneTable {
                filter_move_transformation_fn: None,
                filter_pattern_fn: None,
            },
        };
        let prune_table = Box::new(TPruneTable::new(
            tpuzzle,
            api_data.clone(),
            search_logger,
            min_prune_table_size,
            max_prune_table_size,
            search_adaptations_without_prune_table.clone(),
        ));
        let search_adaptations = StoredSearchAdaptations {
            prune_table,
            filter_move_transformation_fn: search_adaptations_without_prune_table
                .filter_move_transformation_fn,
            filter_pattern_fn: search_adaptations_without_prune_table.filter_pattern_fn,
        };

        Self::try_new_internal(api_data, search_adaptations)
    }

    pub fn legacy_try_new(
        tpuzzle: TPuzzle,
        generator_moves: Vec<Move>, // TODO: turn this back into `Generators`
        target_patterns: Vec<TPuzzle::Pattern>,
        options: IterativeDeepeningSearchConstructionOptions,
        search_adaptations: StoredSearchAdaptations<TPuzzle>,
    ) -> Result<Self, SearchError> {
        let api_data =
            Self::legacy_construct_api_data(tpuzzle, generator_moves, target_patterns, options)?;
        Self::try_new_internal(api_data, search_adaptations)
    }

    fn legacy_construct_api_data(
        tpuzzle: TPuzzle,
        generator_moves: Vec<Move>, // TODO: turn this back into `Generators`
        target_patterns: Vec<TPuzzle::Pattern>,
        options: IterativeDeepeningSearchConstructionOptions,
    ) -> Result<Arc<IterativeDeepeningSearchAPIData<TPuzzle>>, SearchError> {
        let search_generators = SearchGenerators::try_new(
            &tpuzzle,
            generator_moves,
            &options.metric,
            options.random_start,
        )?;

        let canonical_fsm = CanonicalFSM::try_new(
            // TODO: avoid clones
            tpuzzle.clone(),
            search_generators.clone(),
            options.canonical_fsm_construction_options,
        )
        .map_err(|e| SearchError {
            description: e.to_string(),
        })?;

        Ok(Arc::new(IterativeDeepeningSearchAPIData {
            search_generators,
            canonical_fsm,
            tpuzzle: tpuzzle.clone(),
            target_patterns,
            search_logger: options.search_logger.clone(),
        }))
    }

    fn try_new_internal(
        api_data: Arc<IterativeDeepeningSearchAPIData<TPuzzle>>,
        search_adaptations: StoredSearchAdaptations<TPuzzle>,
    ) -> Result<Self, SearchError> {
        Ok(Self {
            api_data,
            stored_search_adaptations: search_adaptations,
        })
    }

    /// Note that search is pull-based. You must call `.next()` (or invoke
    /// something that does) on the return value for the search to begine.
    pub fn search_with_default_individual_search_adaptations<'a>(
        &'a mut self,
        search_pattern: &TPuzzle::Pattern,
        individual_search_options: IndividualSearchOptions,
    ) -> IterativeSearchCursor<'a, TPuzzle> {
        self.search(
            search_pattern,
            individual_search_options,
            Default::default(),
        )
    }

    /// Note that search is pull-based. You must call `.next()` (or invoke
    /// something that does) on the return value for the search to begine.
    pub fn owned_search_with_default_individual_search_adaptations(
        self,
        search_pattern: &TPuzzle::Pattern,
        individual_search_options: IndividualSearchOptions,
    ) -> OwnedIterativeSearchCursor<TPuzzle> {
        self.owned_search(
            search_pattern,
            individual_search_options,
            Default::default(),
        )
    }

    /// Note that search is pull-based. You must call `.next()` (or invoke
    /// something that does) on the return value for the search to begine.
    pub fn search<'a>(
        &'a mut self,
        search_pattern: &TPuzzle::Pattern,
        individual_search_options: IndividualSearchOptions,
        individual_search_adaptations: IndividualSearchAdaptations<TPuzzle>,
    ) -> IterativeSearchCursor<'a, TPuzzle> {
        let individual_search_data = IndividualSearchData::new(
            self,
            search_pattern,
            individual_search_options,
            individual_search_adaptations,
        );
        IterativeSearchCursor {
            search: self,
            individual_search_data,
        }
    }

    /// Note that search is pull-based. You must call `.next()` (or invoke
    /// something that does) on the return value for the search to begine.
    pub fn owned_search(
        mut self,
        search_pattern: &TPuzzle::Pattern,
        individual_search_options: IndividualSearchOptions,
        individual_search_adaptations: IndividualSearchAdaptations<TPuzzle>,
    ) -> OwnedIterativeSearchCursor<TPuzzle> {
        let individual_search_data = IndividualSearchData::new(
            &mut self,
            search_pattern,
            individual_search_options,
            individual_search_adaptations,
        );
        OwnedIterativeSearchCursor {
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
                    self.api_data.search_logger.write_info(&format!(
                        "Increasing initial search depth from {:?} to {:?} based on the root continuation condition: {:?}",
                        options_min_depth, root_continuation_depth, root_continuation_condition
                    ));
                }
                std::cmp::Ordering::Equal => {}
                std::cmp::Ordering::Greater => {
                    if root_continuation_condition != ContinuationCondition::None {
                        self.api_data.search_logger.write_info(&format!(
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
            self.api_data.tpuzzle.clone(),
            individual_search_data.search_pattern.clone(),
        );
        for remaining_depth in *initial_search_depth
            ..*individual_search_data
                .individual_search_options
                .get_max_depth()
        {
            let remaining_depth = Depth(remaining_depth);
            self.api_data.search_logger.write_info("----------------");

            self.stored_search_adaptations
                .prune_table
                .extend_for_search_depth(
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
        let prune_table_depth = self
            .stored_search_adaptations
            .prune_table
            .lookup(current_pattern);
        if prune_table_depth > remaining_depth + Depth(1) {
            return SearchRecursionResult::ContinueSearchingExcludingCurrentMoveClass;
        }
        if prune_table_depth > remaining_depth {
            return SearchRecursionResult::ContinueSearchingDefault;
        }

        for (move_class_index, move_transformation_multiples) in
            self.api_data.search_generators.by_move_class.iter()
        {
            let Some(next_state) = self
                .api_data
                .canonical_fsm
                .next_state(current_state, move_class_index)
            else {
                continue;
            };

            for move_transformation_info in move_transformation_multiples {
                // We check the continuation condition here so that we can
                // resume a search from a continuation condition even if the
                // move is not an accepted by the move transformation filter.
                // TDOO: does this check impact perf?
                if continuation_condition != ContinuationCondition::None {
                    self.api_data.search_logger.write_info(&format!(
                        "{} → {} ? ({:?})",
                        Alg {
                            nodes: solution_moves.snapshot_alg_nodes()
                        },
                        &move_transformation_info.r#move.to_string(),
                        &continuation_condition,
                    ));
                }
                let Some(recursive_continuation_condition) = self.recursive_continuation_condition(
                    &continuation_condition,
                    &move_transformation_info.r#move,
                ) else {
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

    /// A return value of `None` indicates to avoid recursing.
    /// A return value of `Some(…)` indicates to recurse using the given (potential) move.
    fn recursive_continuation_condition(
        &self,
        continuation_condition: &ContinuationCondition,
        potential_move: &Move,
    ) -> Option<ContinuationCondition> {
        match continuation_condition {
            ContinuationCondition::None => Some(ContinuationCondition::None),
            ContinuationCondition::At(moves) => {
                if let Some((first, rest)) = moves.split_first() {
                    if first == potential_move {
                        // eprintln!("Move: {}", first);
                        Some(ContinuationCondition::At(rest.to_vec()))
                    } else {
                        // eprintln!("skippin' {}", potential_move);
                        None
                    }
                } else {
                    // eprintln!("at empty {}", potential_move);
                    Some(ContinuationCondition::None)
                }
            }
            ContinuationCondition::After(moves) => {
                if let Some((first, rest)) = moves.split_first() {
                    if first == potential_move {
                        // eprintln!("Move: {}", first);
                        Some(ContinuationCondition::After(rest.to_vec()))
                    } else {
                        // eprintln!("skippin' {}", potential_move);
                        None
                    }
                } else {
                    // eprintln!("after empty {}", potential_move);
                    Some(ContinuationCondition::None)
                }
            }
        }
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
                    .api_data
                    .search_generators
                    .by_move
                    .get(r#move)
                    .expect("move!")
                    .move_class_index;
                current_state = self
                    .api_data
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
                    self.api_data.search_logger.write_warning("Encountered a non-empty `ContinuationCondition::At` during a base case. This could indicate a bug in the calling code.");
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
            self.api_data.search_logger.write_info(&format!(
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
        self.api_data.target_patterns.contains(current_pattern)
    }
}
