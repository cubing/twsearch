use std::{process::exit, time::Instant};

use cubing::alg::{Alg, Move};

use crate::{
    CanonicalFSM, CanonicalFSMState, MoveClassIndex, PackedKPattern, PackedKPuzzle, SearchError,
    SearchMoveCache, CANONICAL_FSM_START_STATE,
};

type PruneTableEntryType = u8;
const UNINITIALIZED_DEPTH: PruneTableEntryType = 0xff;
const MAX_PRUNE_TABLE_DEPTH: PruneTableEntryType = UNINITIALIZED_DEPTH - 1;

const PRUNE_TABLE_INDEX_MASK: usize = 0xffff;
const DEFAULT_PRUNE_TABLE_SIZE: usize = PRUNE_TABLE_INDEX_MASK + 1;

const MAX_SEARCH_DEPTH: usize = 500; // TODO: increase

pub struct PruneTable<'a> {
    current_pruning_depth: PruneTableEntryType,
    idf_search: &'a IDFSearch,
    pattern_hash_to_depth: Vec<PruneTableEntryType>,
}

impl<'a> PruneTable<'a> {
    pub fn new(idf_search: &'a IDFSearch) -> Self {
        let mut prune_table = Self {
            current_pruning_depth: 0,
            idf_search,
            pattern_hash_to_depth: vec![UNINITIALIZED_DEPTH; DEFAULT_PRUNE_TABLE_SIZE],
        };
        prune_table.extend_for_search_depth(0);
        prune_table
    }

    // TODO: dedup with IDFSearch?
    pub fn extend_for_search_depth(&mut self, search_depth: usize) {
        let mut new_pruning_depth =
            std::convert::TryInto::<PruneTableEntryType>::try_into(search_depth / 2)
                .expect("Prune table depth exceeded available size");
        if new_pruning_depth >= MAX_PRUNE_TABLE_DEPTH {
            println!(
                "Prune table hit max depth, limiting to {}.",
                MAX_PRUNE_TABLE_DEPTH
            );
            new_pruning_depth = MAX_PRUNE_TABLE_DEPTH;
        }
        if new_pruning_depth <= self.current_pruning_depth {
            return;
        }

        let start_time = Instant::now();
        for depth in (self.current_pruning_depth + 1)..(new_pruning_depth + 1) {
            println!("Populating prune table to pruning depth: {}", depth);
            self.recurse(
                &self.idf_search.target_pattern,
                CANONICAL_FSM_START_STATE,
                depth,
            );
            println!(
                "Populating prune table took: {:?}",
                Instant::now() - start_time
            );
        }
        self.current_pruning_depth = new_pruning_depth
    }

    fn hash_pattern(&self, pattern: &PackedKPattern) -> usize {
        (pattern.hash() as usize) & PRUNE_TABLE_INDEX_MASK // TODO: use modulo when the size is not a power of 2.
    }

    // TODO: dedup with IDFSearch?
    fn recurse(
        &mut self,
        current_pattern: &PackedKPattern,
        current_state: CanonicalFSMState,
        remaining_depth: PruneTableEntryType,
    ) {
        if remaining_depth == 0 {
            let pattern_hash = self.hash_pattern(current_pattern);
            if self.pattern_hash_to_depth[pattern_hash] == UNINITIALIZED_DEPTH {
                self.pattern_hash_to_depth[pattern_hash] = remaining_depth;
            };
            return;
        }
        for (move_class_index, move_transformation_multiples) in
            self.idf_search.search_move_cache.grouped.iter().enumerate()
        {
            let next_state = match self
                .idf_search
                .canonical_fsm
                .next_state(current_state, MoveClassIndex(move_class_index))
            {
                Some(next_state) => next_state,
                None => {
                    continue;
                }
            };

            for move_transformation_info in move_transformation_multiples {
                self.recurse(
                    &current_pattern.apply_transformation(&move_transformation_info.transformation),
                    next_state,
                    remaining_depth - 1,
                )
            }
        }
    }

    // Returns a heurstic depth for the given pattern.
    pub fn lookup(&self, pattern: &PackedKPattern) -> usize {
        let pattern_hash = self.hash_pattern(pattern);
        let table_value = self.pattern_hash_to_depth[pattern_hash];
        if table_value == UNINITIALIZED_DEPTH {
            (self.current_pruning_depth as usize) + 1
        } else {
            table_value as usize
        }
    }
}

struct IndividualSearchData<'a> {
    prune_table: &'a mut PruneTable<'a>, // TODO: store this on `IDFSearch`.
    current_depth_num_recursive_calls: usize,
}

pub struct IDFSearch {
    pub search_move_cache: SearchMoveCache,
    pub canonical_fsm: CanonicalFSM,
    pub packed_kpuzzle: PackedKPuzzle,
    pub target_pattern: PackedKPattern,
    pub scramble_pattern: PackedKPattern,
}

impl IDFSearch {
    pub fn try_new(
        packed_kpuzzle: PackedKPuzzle,
        target_pattern: PackedKPattern,
        move_list: Vec<Move>,
        scramble_pattern: PackedKPattern,
    ) -> Result<Self, SearchError> {
        let search_move_cache = SearchMoveCache::try_new(&packed_kpuzzle, &move_list)?;
        let canonical_fsm = CanonicalFSM::try_new(search_move_cache.clone())?; // TODO: avoid a clone
        Ok(Self {
            search_move_cache,
            canonical_fsm,
            packed_kpuzzle,
            target_pattern,
            scramble_pattern,
        })
    }

    pub fn search(&self) -> Result<(), SearchError> {
        let start_time = Instant::now();
        let mut prune_table = PruneTable::new(self); // TODO: make the prune table reusable across searches.
        let mut individual_search_data = IndividualSearchData {
            prune_table: &mut prune_table,
            current_depth_num_recursive_calls: 0,
        };

        for remaining_depth in 0..MAX_SEARCH_DEPTH {
            individual_search_data
                .prune_table
                .extend_for_search_depth(remaining_depth);
            individual_search_data.current_depth_num_recursive_calls = 0;

            println!("Searching to depth: {}", remaining_depth);
            if let Some(solution) = self.recurse(
                &mut individual_search_data,
                &self.scramble_pattern,
                CANONICAL_FSM_START_STATE,
                remaining_depth,
            ) {
                println!("Solution: {}", solution);
                println!("Found at depth: {}", remaining_depth);
                println!("Found in: {:?}", Instant::now() - start_time);
                exit(0);
            }
            println!(
                "Number of recursive calls at depth {}: {}",
                remaining_depth, individual_search_data.current_depth_num_recursive_calls
            );
        }
        Err(SearchError {
            description: "No solution".to_owned(),
        })
    }

    fn recurse(
        &self,
        individual_search_data: &mut IndividualSearchData,
        current_pattern: &PackedKPattern,
        current_state: CanonicalFSMState,
        remaining_depth: usize,
    ) -> Option<Alg> {
        individual_search_data.current_depth_num_recursive_calls += 1;
        if remaining_depth == 0 {
            return if current_pattern == &self.target_pattern {
                Some(Alg { nodes: vec![] })
            } else {
                None
            };
        }
        if individual_search_data.prune_table.lookup(current_pattern) > remaining_depth {
            return None;
        }
        for (move_class_index, move_transformation_multiples) in
            self.search_move_cache.grouped.iter().enumerate()
        {
            let next_state = match self
                .canonical_fsm
                .next_state(current_state, MoveClassIndex(move_class_index))
            {
                Some(next_state) => next_state,
                None => {
                    continue;
                }
            };

            for move_transformation_info in move_transformation_multiples {
                if let Some(solution_tail) = self.recurse(
                    individual_search_data,
                    &current_pattern.apply_transformation(&move_transformation_info.transformation),
                    next_state,
                    remaining_depth - 1,
                ) {
                    let mut solution_tail_nodes = solution_tail.nodes;
                    solution_tail_nodes.insert(
                        0,
                        cubing::alg::AlgNode::MoveNode(move_transformation_info.r#move.clone()),
                    );
                    return Some(Alg {
                        nodes: solution_tail_nodes,
                    });
                }
            }
        }
        None
    }
}
