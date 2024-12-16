use std::{
    fmt::Debug,
    marker::PhantomData,
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc,
    },
};

use cubing::{
    alg::{Alg, AlgNode, Move},
    kpuzzle::KPuzzle,
};
use serde::{Deserialize, Serialize};

use crate::_internal::{
    canonical_fsm::{
        canonical_fsm::{CanonicalFSM, CanonicalFSMState, CANONICAL_FSM_START_STATE},
        search_generators::SearchGenerators,
    },
    cli::args::MetricEnum,
    errors::SearchError,
    puzzle_traits::puzzle_traits::{HashablePatternPuzzle, SemiGroupActionPuzzle},
    search::pattern_stack::PatternStack,
};

use super::{
    check_pattern::{AlwaysValid, PatternValidityChecker},
    hash_prune_table::HashPruneTable,
    prune_table_trait::{Depth, PruneTable},
    recursive_work_tracker::RecursiveWorkTracker,
    search_logger::SearchLogger,
};

const MAX_SUPPORTED_SEARCH_DEPTH: Depth = Depth(500); // TODO: increase

// TODO: use https://doc.rust-lang.org/std/ops/enum.ControlFlow.html as a wrapper instead?
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

#[derive(Clone)]
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
    pub fn construct() -> (SyncSender<Option<Alg>>, Self) {
        // TODO: use `sync_channel` to control resumption?
        let (sender, receiver) = sync_channel::<Option<Alg>>(10);
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
        println!("nexting!");
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
            println!("received!");
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
    pub min_depth: Option<Depth>, // inclusive
    pub max_depth: Option<Depth>, // exclusive
    pub canonical_fsm_pre_moves: Option<Vec<Move>>,
    pub canonical_fsm_post_moves: Option<Vec<Move>>,
}

impl IndividualSearchOptions {
    pub fn get_min_num_solutions(&self) -> usize {
        self.min_num_solutions.unwrap_or(1)
    }
    pub fn get_min_depth(&self) -> Depth {
        self.min_depth.unwrap_or(Depth(0))
    }
    pub fn get_max_depth(&self) -> Depth {
        self.max_depth.unwrap_or(MAX_SUPPORTED_SEARCH_DEPTH)
    }
}

struct IndividualSearchData {
    individual_search_options: IndividualSearchOptions,
    recursive_work_tracker: RecursiveWorkTracker,
    num_solutions_sofar: usize,
    solution_sender: SyncSender<Option<Alg>>,
}

pub struct IDFSearchAPIData<TPuzzle: SemiGroupActionPuzzle> {
    pub search_generators: SearchGenerators<TPuzzle>,
    pub canonical_fsm: CanonicalFSM<TPuzzle>,
    pub tpuzzle: TPuzzle,
    pub target_pattern: TPuzzle::Pattern,
    pub search_logger: Arc<SearchLogger>,
}

pub trait SearchOptimizations<TPuzzle: SemiGroupActionPuzzle> {
    type PatternValidityChecker: PatternValidityChecker<TPuzzle>;
    type PruneTable: PruneTable<TPuzzle>;
}

pub struct NoSearchOptimizations<TPuzzle: HashablePatternPuzzle> {
    phantom_data: PhantomData<TPuzzle>,
}
impl<TPuzzle: HashablePatternPuzzle> SearchOptimizations<TPuzzle>
    for NoSearchOptimizations<TPuzzle>
{
    type PatternValidityChecker = AlwaysValid;
    type PruneTable = HashPruneTable<TPuzzle, Self::PatternValidityChecker>;
}

pub trait DefaultSearchOptimizations<TPuzzle: SemiGroupActionPuzzle> {
    type Optimizations: SearchOptimizations<TPuzzle>;
}

impl DefaultSearchOptimizations<KPuzzle> for KPuzzle {
    type Optimizations = NoSearchOptimizations<KPuzzle>;
}

pub struct IDFSearch<
    TPuzzle: SemiGroupActionPuzzle + DefaultSearchOptimizations<TPuzzle> = KPuzzle,
    Optimizations: SearchOptimizations<TPuzzle> = <TPuzzle as DefaultSearchOptimizations<
        TPuzzle,
    >>::Optimizations,
> {
    pub api_data: Arc<IDFSearchAPIData<TPuzzle>>,
    pub prune_table: Optimizations::PruneTable,
}

impl<
        TPuzzle: SemiGroupActionPuzzle + DefaultSearchOptimizations<TPuzzle>,
        Optimizations: SearchOptimizations<TPuzzle>,
    > IDFSearch<TPuzzle, Optimizations>
{
    pub fn try_new(
        tpuzzle: TPuzzle,
        target_pattern: TPuzzle::Pattern,
        generator_moves: Vec<Move>, // TODO: turn this back into `Generators`
        search_logger: Arc<SearchLogger>,
        metric: &MetricEnum,
        random_start: bool,
        min_prune_table_size: Option<usize>,
    ) -> Result<Self, SearchError> {
        let search_generators =
            SearchGenerators::try_new(&tpuzzle, generator_moves, metric, random_start)?;
        let canonical_fsm = CanonicalFSM::try_new(tpuzzle.clone(), search_generators.clone())?; // TODO: avoid a clone
        let api_data = Arc::new(IDFSearchAPIData {
            search_generators,
            canonical_fsm,
            tpuzzle: tpuzzle.clone(),
            target_pattern,
            search_logger: search_logger.clone(),
        });

        let prune_table = Optimizations::PruneTable::new(
            tpuzzle,
            api_data.clone(),
            search_logger,
            min_prune_table_size,
        ); // TODO: make the prune table reusable across searches.
        Ok(Self {
            api_data,
            prune_table,
        })
    }

    pub fn search(
        &mut self,
        search_pattern: &TPuzzle::Pattern,
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
                &mut individual_search_data,
                &mut pattern_stack,
                initial_state,
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
        pattern_stack: &mut PatternStack<TPuzzle>,
        current_state: CanonicalFSMState,
        remaining_depth: Depth,
        solution_moves: SolutionMoves,
    ) -> SearchRecursionResult {
        let current_pattern = pattern_stack.current_pattern();
        // TODO: apply invalid checks only to intermediate state (i.e. exclude remaining_depth == 0)?
        if !Optimizations::PatternValidityChecker::is_valid(current_pattern) {
            return SearchRecursionResult::ContinueSearchingDefault();
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
        let prune_table_depth = self.prune_table.lookup(current_pattern);
        if prune_table_depth > remaining_depth + Depth(1) {
            return SearchRecursionResult::ContinueSearchingExcludingCurrentMoveClass();
        }
        if prune_table_depth > remaining_depth {
            return SearchRecursionResult::ContinueSearchingDefault();
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
                current_state = match self
                    .api_data
                    .canonical_fsm
                    .next_state(current_state, move_class_index)
                {
                    Some(next_state) => next_state,
                    None => return None,
                }
            }
        }
        Some(current_state)
    }

    fn base_case(
        &self,
        individual_search_data: &mut IndividualSearchData,
        current_pattern: &TPuzzle::Pattern,
        current_state: CanonicalFSMState,
        solution_moves: SolutionMoves,
    ) -> SearchRecursionResult {
        if current_pattern != &self.api_data.target_pattern {
            return SearchRecursionResult::ContinueSearchingDefault();
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
                Alg::from(solution_moves)
            ));
            return SearchRecursionResult::ContinueSearchingDefault();
        }

        let alg = Alg::from(solution_moves);
        individual_search_data.num_solutions_sofar += 1;
        println!("Sending a phase solution!");
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
    }
}
