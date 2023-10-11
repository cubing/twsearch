use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc,
};

use cubing::alg::{Alg, AlgNode, Move};

use crate::{
    CanonicalFSM, CanonicalFSMState, MoveClassIndex, PackedKPattern, PackedKPuzzle, PruneTable,
    RecursiveWorkTracker, SearchError, SearchGenerators, SearchLogger,
    _internal::cli::{Generators, MetricEnum},
    CANONICAL_FSM_START_STATE,
};

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
            match Some(self.receiver.recv().expect(
                "Internal error: could not determine next search solution or end of search.",
            )) {
                Some(alg) => alg,
                None => {
                    self.done = true;
                    None
                }
            }
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct IndividualSearchOptions {
    pub min_num_solutions: Option<usize>,
    pub min_depth: Option<usize>,
    pub max_depth: Option<usize>,
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
    pub packed_kpuzzle: PackedKPuzzle,
    pub target_pattern: PackedKPattern,
    pub search_logger: Arc<SearchLogger>,
}

pub struct IDFSearch {
    api_data: Arc<IDFSearchAPIData>,
    prune_table: PruneTable,
}

impl IDFSearch {
    pub fn try_new(
        packed_kpuzzle: PackedKPuzzle,
        target_pattern: PackedKPattern,
        generators: Generators,
        search_logger: Arc<SearchLogger>,
        metric: &MetricEnum,
        random_start: bool,
    ) -> Result<Self, SearchError> {
        let search_generators =
            SearchGenerators::try_new(&packed_kpuzzle, &generators, metric, random_start)?;
        let canonical_fsm = CanonicalFSM::try_new(search_generators.clone())?; // TODO: avoid a clone
        let api_data = Arc::new(IDFSearchAPIData {
            search_generators,
            canonical_fsm,
            packed_kpuzzle,
            target_pattern,
            search_logger: search_logger.clone(),
        });

        let prune_table = PruneTable::new(api_data.clone(), search_logger); // TODO: make the prune table reusable across searches.
        Ok(Self {
            api_data,
            prune_table,
        })
    }

    pub fn search(
        mut self,
        search_pattern: &PackedKPattern,
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

        for remaining_depth in
            individual_search_options.get_min_depth()..individual_search_options.get_max_depth()
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
        current_pattern: &PackedKPattern,
        current_state: CanonicalFSMState,
        remaining_depth: usize,
        solution_moves: SolutionMoves,
    ) -> SearchRecursionResult {
        individual_search_data
            .recursive_work_tracker
            .record_recursive_call();
        if remaining_depth == 0 {
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
                match self.recurse(
                    individual_search_data,
                    &current_pattern.apply_transformation(&move_transformation_info.transformation),
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
