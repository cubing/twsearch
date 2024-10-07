use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc,
};

use cubing::{
    alg::{Alg, AlgNode, Move, QuantumMove},
    kpuzzle::{KPattern, KPuzzle},
};
use serde::{Deserialize, Serialize};

use crate::_internal::{
    cli::options::{Generators, MetricEnum},
    CanonicalFSM, CanonicalFSMState, MoveClassIndex, PruneTable, RecursiveWorkTracker, SearchError,
    SearchGenerators, SearchLogger, CANONICAL_FSM_START_STATE,
};

use super::CheckPattern;

const MAX_SUPPORTED_SEARCH_DEPTH: usize = 500; // TODO: increase

#[allow(clippy::enum_variant_names)]
enum SearchRecursionResult {
    DoneSearching(),
    ContinueSearchingDefault(),
    ContinueSearchingExcludingCurrentMoveClass(),
}

struct SolutionPreviousMoves<'a> {
    latest_move: &'a Move,
    previous_moves: &'a SolutionMoves<'a>,
}

struct SolutionMoves<'a>(Option<&'a SolutionPreviousMoves<'a>>);

impl<'a> From<SolutionMoves<'a>> for Alg {
    fn from(value: SolutionMoves<'a>) -> Self {
        let nodes = value.get_alg_nodes();
        Alg { nodes }
    }
}

impl<'a> SolutionMoves<'a> {
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
    pub min_depth: Option<usize>,                            // inclusive
    pub max_depth: Option<usize>,                            // exclusive
    pub disallowed_initial_quanta: Option<Vec<QuantumMove>>, // TODO: Change this to `fsm_pre_moves` so we can compute disallowed initial FSM states.
    pub disallowed_final_quanta: Option<Vec<QuantumMove>>, // TODO: Find a way to represent this using disallowed final FSM states?
}

fn is_move_disallowed(r#move: &Move, disallowed_quanta: &Option<Vec<QuantumMove>>) -> bool {
    // TODO Use something like a `HashSet` to speed this up?
    if let Some(disallowed_quanta) = disallowed_quanta {
        for disallowed_quantum in disallowed_quanta {
            if r#move.quantum.as_ref() == disallowed_quantum {
                return true;
            }
        }
    }
    false
}

impl IndividualSearchOptions {
    pub fn get_min_num_solutions(&self) -> usize {
        self.min_num_solutions.unwrap_or(1)
    }
    pub fn get_min_depth(&self) -> usize {
        self.min_depth.unwrap_or(0)
    }
    pub fn get_max_depth(&self) -> usize {
        self.max_depth.unwrap_or(MAX_SUPPORTED_SEARCH_DEPTH)
    }
}

struct IndividualSearchData {
    individual_search_options: IndividualSearchOptions,
    recursive_work_tracker: RecursiveWorkTracker,
    num_solutions_sofar: usize,
    solution_sender: Sender<Option<Alg>>,
}

pub struct IDFSearchAPIData {
    pub search_generators: SearchGenerators,
    pub canonical_fsm: CanonicalFSM,
    pub kpuzzle: KPuzzle,
    pub target_pattern: KPattern,
    pub search_logger: Arc<SearchLogger>,
}

pub struct IDFSearch<T: CheckPattern> {
    api_data: Arc<IDFSearchAPIData>,
    prune_table: PruneTable<T>,
}

impl<T: CheckPattern> IDFSearch<T> {
    pub fn try_new(
        kpuzzle: KPuzzle,
        target_pattern: KPattern,
        generators: Generators,
        search_logger: Arc<SearchLogger>,
        metric: &MetricEnum,
        random_start: bool,
        min_prune_table_size: Option<usize>,
    ) -> Result<Self, SearchError> {
        let search_generators =
            SearchGenerators::try_new(&kpuzzle, &generators, metric, random_start)?;
        let canonical_fsm = CanonicalFSM::try_new(search_generators.clone())?; // TODO: avoid a clone
        let api_data = Arc::new(IDFSearchAPIData {
            search_generators,
            canonical_fsm,
            kpuzzle,
            target_pattern,
            search_logger: search_logger.clone(),
        });

        let prune_table = PruneTable::new(api_data.clone(), search_logger, min_prune_table_size); // TODO: make the prune table reusable across searches.
        Ok(Self {
            api_data,
            prune_table,
        })
    }

    pub fn search(
        &mut self,
        search_pattern: &KPattern,
        mut individual_search_options: IndividualSearchOptions,
    ) -> SearchSolutions {
        // TODO: do validation more consistently.
        if let Some(min_depth) = individual_search_options.min_depth {
            if min_depth > MAX_SUPPORTED_SEARCH_DEPTH {
                self.api_data
                    .search_logger
                    .write_error("Min depth too large, capping at maximum.");
                individual_search_options.min_depth = Some(MAX_SUPPORTED_SEARCH_DEPTH);
            }
        }
        if let Some(max_depth) = individual_search_options.max_depth {
            if max_depth > MAX_SUPPORTED_SEARCH_DEPTH {
                self.api_data
                    .search_logger
                    .write_error("Max depth too large, capping at maximum.");
                individual_search_options.max_depth = Some(MAX_SUPPORTED_SEARCH_DEPTH);
            }
        }

        assert!(
            individual_search_options.get_max_depth() > individual_search_options.get_min_depth()
        );

        let (solution_sender, search_solutions) = SearchSolutions::construct();
        let mut individual_search_data = IndividualSearchData {
            individual_search_options,
            recursive_work_tracker: RecursiveWorkTracker::new(
                "Search".to_owned(),
                self.api_data.search_logger.clone(),
            ),
            num_solutions_sofar: 0,
            solution_sender,
        };

        let search_pattern = search_pattern.clone();

        for remaining_depth in individual_search_data
            .individual_search_options
            .get_min_depth()
            ..individual_search_data
                .individual_search_options
                .get_max_depth()
        {
            self.api_data.search_logger.write_info("----------------");
            self.prune_table.extend_for_search_depth(
                remaining_depth,
                individual_search_data
                    .recursive_work_tracker
                    .estimate_next_level_num_recursive_calls(),
            );
            individual_search_data
                .recursive_work_tracker
                .start_depth(remaining_depth, Some("Starting search…"));
            let recursion_result = self.recurse(
                &mut individual_search_data,
                &search_pattern,
                CANONICAL_FSM_START_STATE,
                remaining_depth,
                SolutionMoves(None),
            );
            individual_search_data
                .recursive_work_tracker
                .finish_latest_depth();
            if let SearchRecursionResult::DoneSearching() = recursion_result {
                break;
            }
        }
        search_solutions
    }

    fn recurse(
        &self,
        individual_search_data: &mut IndividualSearchData,
        current_pattern: &KPattern,
        current_state: CanonicalFSMState,
        remaining_depth: usize,
        solution_moves: SolutionMoves,
    ) -> SearchRecursionResult {
        if remaining_depth == 0 && !T::is_valid(current_pattern) {
            return SearchRecursionResult::ContinueSearchingDefault();
        }

        individual_search_data
            .recursive_work_tracker
            .record_recursive_call();
        if remaining_depth == 0 {
            if let Some(previous_moves) = solution_moves.0 {
                if is_move_disallowed(
                    previous_moves.latest_move,
                    &individual_search_data
                        .individual_search_options
                        .disallowed_final_quanta,
                ) {
                    return SearchRecursionResult::ContinueSearchingDefault();
                }
            }
            return if current_pattern == &self.api_data.target_pattern {
                individual_search_data.num_solutions_sofar += 1;
                let alg = Alg::from(solution_moves);
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
                    SearchRecursionResult::DoneSearching()
                } else {
                    SearchRecursionResult::ContinueSearchingDefault()
                }
            } else {
                SearchRecursionResult::ContinueSearchingDefault()
            };
        }
        let prune_table_depth = self.prune_table.lookup(current_pattern);
        if prune_table_depth > remaining_depth + 1 {
            return SearchRecursionResult::ContinueSearchingExcludingCurrentMoveClass();
        }
        if prune_table_depth > remaining_depth {
            return SearchRecursionResult::ContinueSearchingDefault();
        }
        for (move_class_index, move_transformation_multiples) in
            self.api_data.search_generators.grouped.iter().enumerate()
        {
            let next_state = match self
                .api_data
                .canonical_fsm
                .next_state(current_state, MoveClassIndex(move_class_index))
            {
                Some(next_state) => next_state,
                None => {
                    continue;
                }
            };

            for move_transformation_info in move_transformation_multiples {
                if current_state == CANONICAL_FSM_START_STATE
                    && is_move_disallowed(
                        &move_transformation_info.r#move,
                        &individual_search_data
                            .individual_search_options
                            .disallowed_initial_quanta,
                    )
                {
                    // TODO: is it always safe to `break` here?
                    continue;
                }

                let next_pattern =
                    &current_pattern.apply_transformation(&move_transformation_info.transformation);

                match self.recurse(
                    individual_search_data,
                    next_pattern,
                    next_state,
                    remaining_depth - 1,
                    SolutionMoves(Some(&SolutionPreviousMoves {
                        latest_move: &move_transformation_info.r#move,
                        previous_moves: &solution_moves,
                    })),
                ) {
                    SearchRecursionResult::DoneSearching() => {
                        return SearchRecursionResult::DoneSearching();
                    }
                    SearchRecursionResult::ContinueSearchingDefault() => {}
                    SearchRecursionResult::ContinueSearchingExcludingCurrentMoveClass() => {
                        break;
                    }
                }
            }
        }
        SearchRecursionResult::ContinueSearchingDefault()
    }
}
