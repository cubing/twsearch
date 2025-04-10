use std::{
    fmt::Debug,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
};

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
};

// TODO: right now we return 0 solutions if we blow past this, should we return an explicit error,
// or panic instead?
const MAX_SUPPORTED_SEARCH_DEPTH: Depth = Depth(500); // TODO: increase

// TODO: use https://doc.rust-lang.org/std/ops/enum.ControlFlow.html as a wrapper instead?
#[allow(clippy::enum_variant_names)]
enum SearchRecursionResult {
    DoneSearching,
    ContinueSearchingDefault,
    ContinueSearchingExcludingCurrentMoveClass,
}

struct SolutionPreviousMoves<'a> {
    latest_move: &'a Move,
    previous_moves: &'a SolutionMoves<'a>,
}

#[derive(Clone)]
pub struct SolutionMoves<'a>(Option<&'a SolutionPreviousMoves<'a>>);

impl<'a> From<&SolutionMoves<'a>> for Alg {
    fn from(value: &SolutionMoves<'a>) -> Self {
        let nodes = value.get_alg_nodes();
        Alg { nodes }
    }
}

impl SolutionMoves<'_> {
    fn get_alg_nodes(&self) -> Vec<AlgNode> {
        match self.0 {
            Some(solution_previous_moves) => {
                let mut nodes = solution_previous_moves.previous_moves.get_alg_nodes();
                nodes.push(cubing::alg::AlgNode::MoveNode(
                    solution_previous_moves.latest_move.clone(),
                ));
                nodes
            }
            None => vec![],
        }
    }

    pub fn reverse_move_iter(&self) -> SolutionMovesReverseIterator {
        SolutionMovesReverseIterator {
            solution_moves: self,
        }
    }
}

pub struct SolutionMovesReverseIterator<'a> {
    solution_moves: &'a SolutionMoves<'a>,
}

impl<'a> Iterator for SolutionMovesReverseIterator<'a> {
    type Item = &'a Move;

    fn next(&mut self) -> Option<Self::Item> {
        let solution_previous_moves = self.solution_moves.0?;
        self.solution_moves = solution_previous_moves.previous_moves;
        Some(solution_previous_moves.latest_move)
    }
}

pub struct SearchSolutions {
    receiver: Receiver<Option<Alg>>,
    done: bool,
}

impl SearchSolutions {
    pub fn construct() -> (Sender<Option<Alg>>, Self) {
        // TODO: use `sync_channel` to control resumption?
        let (sender, receiver) = channel::<Option<Alg>>();
        (
            sender,
            Self {
                receiver,
                done: false,
            },
        )
    }
}

impl Iterator for SearchSolutions {
    type Item = Alg;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            let received = match self.receiver.recv() {
                Ok(received) => received,
                Err(_) => {
                    // TODO: this could be either a channel failure or no solutions found. We should find a way for the latter to avoid hitting this code path.
                    self.done = true;
                    return None;
                }
            };
            match received {
                Some(alg) => Some(alg),
                None => {
                    self.done = true;
                    None
                }
            }
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndividualSearchOptions {
    pub min_num_solutions: Option<usize>,
    pub min_depth_inclusive: Option<Depth>, // inclusive
    pub max_depth_exclusive: Option<Depth>, // exclusive
    pub canonical_fsm_pre_moves: Option<Vec<Move>>,
    pub canonical_fsm_post_moves: Option<Vec<Move>>,
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
    individual_search_options: IndividualSearchOptions,
    recursive_work_tracker: RecursiveWorkTracker,
    num_solutions_sofar: usize,
    solution_sender: Sender<Option<Alg>>,
    individual_search_adaptations: IndividualSearchAdaptations<TPuzzle>,
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
    pub canonical_fsm_construction_options: CanonicalFSMConstructionOptions,
}

impl Default for IterativeDeepeningSearchConstructionOptions {
    fn default() -> Self {
        Self {
            search_logger: Default::default(),
            metric: MetricEnum::Hand,
            random_start: Default::default(),
            min_prune_table_size: Default::default(),
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
}

impl<TPuzzle: SemiGroupActionPuzzle> IterativeDeepeningSearch<TPuzzle> {
    fn try_new_internal(
        api_data: Arc<IterativeDeepeningSearchAPIData<TPuzzle>>,
        search_adaptations: StoredSearchAdaptations<TPuzzle>,
    ) -> Result<Self, SearchError> {
        Ok(Self {
            api_data,
            stored_search_adaptations: search_adaptations,
        })
    }

    pub fn search_with_default_individual_search_adaptations(
        &mut self,
        search_pattern: &TPuzzle::Pattern,
        individual_search_options: IndividualSearchOptions,
    ) -> SearchSolutions {
        self.search(
            search_pattern,
            individual_search_options,
            Default::default(),
        )
    }

    pub fn search(
        &mut self,
        search_pattern: &TPuzzle::Pattern,
        mut individual_search_options: IndividualSearchOptions,
        individual_search_adaptations: IndividualSearchAdaptations<TPuzzle>,
    ) -> SearchSolutions {
        // TODO: do validation more consistently.
        if let Some(min_depth) = individual_search_options.min_depth_inclusive {
            if min_depth > MAX_SUPPORTED_SEARCH_DEPTH {
                self.api_data
                    .search_logger
                    .write_error("Min depth too large, capping at maximum.");
                individual_search_options.min_depth_inclusive = Some(MAX_SUPPORTED_SEARCH_DEPTH);
            }
        }
        if let Some(max_depth) = individual_search_options.max_depth_exclusive {
            if max_depth > MAX_SUPPORTED_SEARCH_DEPTH {
                self.api_data
                    .search_logger
                    .write_error("Max depth too large, capping at maximum.");
                individual_search_options.max_depth_exclusive = Some(MAX_SUPPORTED_SEARCH_DEPTH);
            }
        }

        let (solution_sender, search_solutions) = SearchSolutions::construct();
        let mut individual_search_data = IndividualSearchData {
            individual_search_options,
            recursive_work_tracker: RecursiveWorkTracker::new(
                "Search".to_owned(),
                self.api_data.search_logger.clone(),
            ),
            num_solutions_sofar: 0,
            solution_sender,
            individual_search_adaptations,
        };

        let search_pattern = search_pattern.clone();

        // TODO: combine `KPatternStack` with `SolutionMoves`?
        let mut pattern_stack = PatternStack::new(self.api_data.tpuzzle.clone(), search_pattern);
        for remaining_depth in *individual_search_data
            .individual_search_options
            .get_min_depth()
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
                &mut individual_search_data,
                &mut pattern_stack,
                initial_state,
                remaining_depth,
                SolutionMoves(None),
            );
            individual_search_data
                .recursive_work_tracker
                .finish_latest_depth();
            if let SearchRecursionResult::DoneSearching = recursion_result {
                break;
            }
        }
        search_solutions
    }

    fn recurse(
        &self,
        individual_search_data: &mut IndividualSearchData<TPuzzle>,
        pattern_stack: &mut PatternStack<TPuzzle>,
        current_state: CanonicalFSMState,
        remaining_depth: Depth,
        solution_moves: SolutionMoves,
    ) -> SearchRecursionResult {
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
                    SolutionMoves(Some(&SolutionPreviousMoves {
                        latest_move: &move_transformation_info.r#move,
                        previous_moves: &solution_moves,
                    })),
                );
                pattern_stack.pop();

                match recursive_result {
                    SearchRecursionResult::DoneSearching => {
                        return SearchRecursionResult::DoneSearching;
                    }
                    SearchRecursionResult::ContinueSearchingDefault => {}
                    SearchRecursionResult::ContinueSearchingExcludingCurrentMoveClass => {
                        break;
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
    ) -> SearchRecursionResult {
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
        individual_search_data
            .solution_sender
            .send(Some(alg))
            .expect("Internal error: could not send solution");
        if individual_search_data.num_solutions_sofar
            >= individual_search_data
                .individual_search_options
                .get_min_num_solutions()
        {
            individual_search_data
                .solution_sender
                .send(None)
                .expect("Internal error: could not send end of search");
            SearchRecursionResult::DoneSearching
        } else {
            SearchRecursionResult::ContinueSearchingDefault
        }
    }

    fn is_target_pattern(&self, current_pattern: &TPuzzle::Pattern) -> bool {
        // TODO: use a hash set instead (for when there is more than 1 target pattern)
        self.api_data.target_patterns.contains(current_pattern)
    }
}
