use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc,
};

use cubing::alg::{Alg, AlgNode, Move, QuantumMove};
use serde::{Deserialize, Serialize};

use crate::_internal::{
    cli::options::{Generators, MetricEnum},
    CanonicalFSM, CanonicalFSMState, MoveClassIndex, PruneTable, PuzzleError, RecursiveWorkTracker,
    SearchGenerators, SearchLogger, CANONICAL_FSM_START_STATE,
};

use super::{GenericPuzzle, GenericPuzzleCore};

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
        // TODO: use `SyncSender` once the main work in a separate task.
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
            let rec = self.receiver.recv();
            let received = match rec {
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
    pub min_depth: Option<usize>,
    pub max_depth: Option<usize>,
    pub disallowed_initial_quanta: Option<Vec<QuantumMove>>, // TODO: Change this to `fsm_pre_moves` so we can compute disallowed initial FSM states.
    pub disallowed_final_quanta: Option<Vec<QuantumMove>>, // TODO: Find a way to represent this using disallowed final FSM states?
    pub phase2_debug: bool,                                // TODO: remove
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

// TODO: this is very hardcoded to facilitate current scramble searches; it can likely be improved.
pub(crate) trait AdditionalSolutionCondition<TPuzzle: GenericPuzzleCore> {
    fn should_accept_solution(
        &mut self, // TODO: un-mut?
        candidate_pattern: &TPuzzle::Pattern,
        candidate_alg: &Alg,
    ) -> bool;
}

// TODO: this is very hardcoded to facilitate current scramble searches; it can likely be improved.
pub(crate) trait ReplacementSolutionCondition<
    TPuzzle: GenericPuzzleCore,
    THeuristic: SearchHeuristic<TPuzzle>,
>
{
    fn should_accept_solution(
        &mut self, // TODO: un-mut?
        candidate_pattern: &TPuzzle::Pattern,
        candidate_alg: &THeuristic,
    ) -> bool;
}

pub(crate) enum SolutionCondition<
    TPuzzle: GenericPuzzleCore,
    THeuristic: SearchHeuristic<TPuzzle>, // TODO: avoid requiring this unless a replacement condition is needed.
> {
    Default,
    Additional(Box<dyn AdditionalSolutionCondition<TPuzzle>>), // TODO: handle this with backpressure on the iterator instead.
    Replacement(Box<dyn ReplacementSolutionCondition<TPuzzle, THeuristic>>),
}

// TODO: this is very hardcoded to facilitate current scramble searches; it can likely be improved.
enum IsSolvedConclusion {
    NotSolved,
    Solved(Alg),
}

struct IndividualSearchData<TPuzzle: GenericPuzzleCore, THeuristic: SearchHeuristic<TPuzzle>> {
    individual_search_options: IndividualSearchOptions,
    recursive_work_tracker: RecursiveWorkTracker,
    num_solutions_sofar: usize,
    solution_sender: Sender<Option<Alg>>,
    pub solution_condition: SolutionCondition<TPuzzle, THeuristic>,
    phase2_debug: bool, // TODO: remove
}

pub struct IDFSearchAPIData<TPuzzle: GenericPuzzleCore> {
    pub search_generators: SearchGenerators<TPuzzle>, // TODO: pass generic constraints down here.
    pub canonical_fsm: CanonicalFSM<TPuzzle>,
    pub tpuzzle: TPuzzle,
    pub target_pattern: TPuzzle::Pattern,
    pub search_logger: Arc<SearchLogger>,
}

pub(crate) trait SearchHeuristic<TPuzzle: GenericPuzzleCore> {
    fn extend_for_search_depth(&mut self, search_depth: usize, approximate_num_entries: usize);
    fn lookup(&self, pattern: &TPuzzle::Pattern) -> usize;
}

impl<TPuzzle: GenericPuzzleCore> SearchHeuristic<TPuzzle> for PruneTable<TPuzzle> {
    fn extend_for_search_depth(&mut self, search_depth: usize, approximate_num_entries: usize) {
        self.extend_for_search_depth(search_depth, approximate_num_entries)
    }

    fn lookup(&self, pattern: &<TPuzzle as GenericPuzzleCore>::Pattern) -> usize {
        self.lookup(pattern)
    }
}

pub(crate) struct IDFSearch<
    TPuzzle: GenericPuzzleCore,
    THeuristic: SearchHeuristic<TPuzzle> = PruneTable<TPuzzle>,
> {
    pub(crate) api_data: Arc<IDFSearchAPIData<TPuzzle>>,
    search_heuristic: THeuristic,
}

impl<TPuzzle: GenericPuzzle> IDFSearch<TPuzzle> {
    pub fn try_new(
        tpuzzle: TPuzzle,
        target_pattern: TPuzzle::Pattern,
        generators: Generators,
        search_logger: Arc<SearchLogger>,
        metric: &MetricEnum,
        random_start: bool,
        min_prune_table_size: Option<usize>,
    ) -> Result<Self, PuzzleError> {
        let search_generators =
            SearchGenerators::<TPuzzle>::try_new(&tpuzzle, &generators, metric, random_start)?;

        let canonical_fsm = CanonicalFSM::try_new(search_generators.clone())?; // TODO: avoid a clone

        let api_data = Arc::new(IDFSearchAPIData::<TPuzzle> {
            search_generators,
            canonical_fsm,
            tpuzzle,
            target_pattern,
            search_logger: search_logger.clone(),
        });
        let prune_table = PruneTable::new(api_data.clone(), search_logger, min_prune_table_size); // TODO: make the prune table reusable across searches.

        Self::try_new_core(api_data, prune_table)
    }
}

impl<TPuzzle: GenericPuzzleCore, THeuristic: SearchHeuristic<TPuzzle>>
    IDFSearch<TPuzzle, THeuristic>
{
    pub fn try_new_core(
        api_data: Arc<IDFSearchAPIData<TPuzzle>>,
        search_heuristic: THeuristic,
    ) -> Result<Self, PuzzleError> {
        Ok(Self {
            api_data,
            search_heuristic,
        })
    }

    pub fn search(
        &mut self,
        search_pattern: &TPuzzle::Pattern,
        individual_search_options: IndividualSearchOptions,
    ) -> SearchSolutions {
        self.search_with_additional_check(
            search_pattern,
            individual_search_options,
            SolutionCondition::Default,
        )
    }

    pub(crate) fn search_with_additional_check(
        &mut self,
        search_pattern: &TPuzzle::Pattern,
        mut individual_search_options: IndividualSearchOptions,
        solution_condition: SolutionCondition<TPuzzle, THeuristic>,
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

        let (solution_sender, search_solutions) = SearchSolutions::construct();
        let mut individual_search_data = IndividualSearchData {
            phase2_debug: individual_search_options.phase2_debug,
            individual_search_options,
            recursive_work_tracker: RecursiveWorkTracker::new(
                "Search".to_owned(),
                self.api_data.search_logger.clone(),
            ),
            num_solutions_sofar: 0,
            solution_sender,
            solution_condition,
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
            self.search_heuristic.extend_for_search_depth(
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
                false,
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
        individual_search_data: &mut IndividualSearchData<TPuzzle, THeuristic>,
        current_pattern: &TPuzzle::Pattern,
        current_state: CanonicalFSMState,
        remaining_depth: usize,
        last_pattern_is_maybe_unsolved: bool,
        solution_moves: SolutionMoves,
    ) -> SearchRecursionResult {
        individual_search_data
            .recursive_work_tracker
            .record_recursive_call();

        // Confusing name, just means "position is solved, except with false negatives in some cases".
        let mut pattern_is_maybe_unsolved = true;

        if remaining_depth == 0 {
            // dbg!(current_pattern);
            // dbg!(self.search_heuristic.lookup(current_pattern));

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
            let solved_conclusion: IsSolvedConclusion =
                match &mut individual_search_data.solution_condition {
                    SolutionCondition::Default => {
                        if current_pattern == &self.api_data.target_pattern {
                            let alg = Alg::from(solution_moves); // TODO: deduplicate?
                            IsSolvedConclusion::Solved(alg)
                        } else {
                            IsSolvedConclusion::NotSolved
                        }
                    }
                    SolutionCondition::Additional(solution_condition) => {
                        if current_pattern != &self.api_data.target_pattern {
                            IsSolvedConclusion::NotSolved
                        } else {
                            // TODO: avoid calculating the alg unless needed by the solution condition implementation?
                            let alg = Alg::from(solution_moves);
                            if solution_condition.should_accept_solution(current_pattern, &alg) {
                                IsSolvedConclusion::Solved(alg)
                            } else {
                                IsSolvedConclusion::NotSolved
                            }
                        }
                    }
                    SolutionCondition::Replacement(solution_condition) => {
                        // TODO: avoid calculating the alg unless needed by the solution condition implementation?
                        let alg = Alg::from(solution_moves);
                        if solution_condition
                            .should_accept_solution(current_pattern, &self.search_heuristic)
                        {
                            IsSolvedConclusion::Solved(alg)
                        } else {
                            IsSolvedConclusion::NotSolved
                        }
                    }
                };
            return if let IsSolvedConclusion::Solved(alg) = solved_conclusion {
                println!("send");

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
                    SearchRecursionResult::DoneSearching()
                } else {
                    SearchRecursionResult::ContinueSearchingDefault()
                }
            } else {
                SearchRecursionResult::ContinueSearchingDefault()
            };
        } else {
            #[allow(clippy::collapsible_if)]
            if matches!(
                individual_search_data.solution_condition,
                SolutionCondition::Additional(_)
            ) {
                if current_pattern == &self.api_data.target_pattern {
                    // println!("early solved!");
                    if !last_pattern_is_maybe_unsolved {
                        // println!("early double solved!");
                        // return SearchRecursionResult::ContinueSearchingDefault();
                    }

                    pattern_is_maybe_unsolved = false;
                }
            }
        }

        let prune_table_depth = self.search_heuristic.lookup(current_pattern);
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
                if current_state == CANONICAL_FSM_START_STATE
                    && individual_search_data.phase2_debug == true
                {
                    dbg!(move_transformation_info.r#move.to_string());
                    dbg!(&move_transformation_info.transformation);
                }
                match self.recurse(
                    individual_search_data,
                    &TPuzzle::pattern_apply_transformation(
                        &self.api_data.tpuzzle,
                        current_pattern,
                        &move_transformation_info.transformation,
                    ),
                    next_state,
                    remaining_depth - 1,
                    pattern_is_maybe_unsolved,
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
