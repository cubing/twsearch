use std::rc::Rc;

use thousands::Separable;

use crate::{
    CanonicalFSMState, MoveClassIndex, PackedKPattern, RecursiveWorkTracker,
    CANONICAL_FSM_START_STATE,
};

use super::search::IDFSearchAPIData;

type PruneTableEntryType = u8;
// 0 is uninitialized, all other values are stored as 1+depth.
// This allows us to save initialization time by allowing table memory pages to start as "blank" (all 0).
const UNINITIALIZED_DEPTH: PruneTableEntryType = 0;
const MAX_PRUNE_TABLE_DEPTH: PruneTableEntryType = PruneTableEntryType::MAX - 1;

const MIN_PRUNE_TABLE_SIZE: usize = 1 << 16;

struct PruneTableImmutableData {
    search_api_data: Rc<IDFSearchAPIData>,
}
struct PruneTableMutableData {
    prune_table_size: usize,       // power of 2
    prune_table_index_mask: usize, // prune_table_size - 1
    current_pruning_depth: PruneTableEntryType,
    pattern_hash_to_depth: Vec<PruneTableEntryType>,
    recursive_work_tracker: RecursiveWorkTracker,
}

impl PruneTableMutableData {
    fn hash_pattern(&self, pattern: &PackedKPattern) -> usize {
        (pattern.hash() as usize) & self.prune_table_index_mask // TODO: use modulo when the size is not a power of 2.
    }

    // Returns a heurstic depth for the given pattern.
    pub fn lookup(&self, pattern: &PackedKPattern) -> usize {
        let pattern_hash = self.hash_pattern(pattern);
        let table_value = self.pattern_hash_to_depth[pattern_hash];
        if table_value == UNINITIALIZED_DEPTH {
            (self.current_pruning_depth as usize) + 1
        } else {
            (table_value as usize) - 1
        }
    }

    pub fn set_if_uninitialized(&mut self, pattern: &PackedKPattern, depth: u8) {
        let pattern_hash = self.hash_pattern(pattern);
        if self.pattern_hash_to_depth[pattern_hash] == UNINITIALIZED_DEPTH {
            self.pattern_hash_to_depth[pattern_hash] = depth + 1
        };
    }
}

pub struct PruneTable {
    immutable: PruneTableImmutableData,
    mutable: PruneTableMutableData,
}

impl PruneTable {
    pub fn new(search_api_data: Rc<IDFSearchAPIData>) -> Self {
        let mut prune_table = Self {
            immutable: PruneTableImmutableData { search_api_data },
            mutable: PruneTableMutableData {
                prune_table_size: MIN_PRUNE_TABLE_SIZE,
                prune_table_index_mask: MIN_PRUNE_TABLE_SIZE - 1,
                current_pruning_depth: 0,
                pattern_hash_to_depth: vec![0; MIN_PRUNE_TABLE_SIZE],
                recursive_work_tracker: RecursiveWorkTracker::new("Prune table".to_owned()),
            },
        };
        prune_table.extend_for_search_depth(0, 1);
        prune_table
    }

    // TODO: dedup with IDFSearch?
    // TODO: Store a reference to `search_api_data` so that you can't accidentally pass in the wrong `search_api_data`?
    pub fn extend_for_search_depth(&mut self, search_depth: usize, approximate_num_entries: usize) {
        let mut new_pruning_depth =
            std::convert::TryInto::<PruneTableEntryType>::try_into(search_depth / 2)
                .expect("Prune table depth exceeded available size");
        if new_pruning_depth >= MAX_PRUNE_TABLE_DEPTH {
            println!(
                "[Prune table] Hit max depth, limiting to {}.",
                MAX_PRUNE_TABLE_DEPTH
            );
            new_pruning_depth = MAX_PRUNE_TABLE_DEPTH;
        }

        let new_prune_table_size = usize::max(
            usize::next_power_of_two(approximate_num_entries),
            MIN_PRUNE_TABLE_SIZE,
        );
        if new_prune_table_size == self.mutable.prune_table_size {
            if new_pruning_depth <= self.mutable.current_pruning_depth {
                return;
            }
        } else {
            self.mutable.pattern_hash_to_depth = vec![0; new_prune_table_size];
            self.mutable.prune_table_size = new_prune_table_size;
            self.mutable.prune_table_index_mask = new_prune_table_size - 1;
        }

        for depth in (self.mutable.current_pruning_depth + 1)..(new_pruning_depth + 1) {
            self.mutable.recursive_work_tracker.start_depth(
                depth as usize,
                &format!(
                    "Populating prune table with {} entries…",
                    self.mutable.prune_table_size.separate_with_underscores()
                ),
            );
            Self::recurse(
                &self.immutable,
                &mut self.mutable,
                &self.immutable.search_api_data.target_pattern,
                CANONICAL_FSM_START_STATE,
                depth,
            );
            self.mutable.recursive_work_tracker.finish_latest_depth();
        }
        self.mutable.current_pruning_depth = new_pruning_depth
    }

    // TODO: dedup with IDFSearch?
    // TODO: Store a reference to `search_api_data` so that you can't accidentally pass in the wrong `search_api_data`?
    fn recurse(
        immutable_data: &PruneTableImmutableData,
        mutable_data: &mut PruneTableMutableData,
        current_pattern: &PackedKPattern,
        current_state: CanonicalFSMState,
        remaining_depth: PruneTableEntryType,
    ) {
        mutable_data.recursive_work_tracker.record_recursive_call();
        if remaining_depth == 0 {
            mutable_data.set_if_uninitialized(current_pattern, remaining_depth);
            return;
        }
        for (move_class_index, move_transformation_multiples) in immutable_data
            .search_api_data
            .search_move_cache
            .grouped
            .iter()
            .enumerate()
        {
            let next_state = match immutable_data
                .search_api_data
                .canonical_fsm
                .next_state(current_state, MoveClassIndex(move_class_index))
            {
                Some(next_state) => next_state,
                None => {
                    continue;
                }
            };

            for move_transformation_info in move_transformation_multiples {
                Self::recurse(
                    immutable_data,
                    mutable_data,
                    &current_pattern.apply_transformation(&move_transformation_info.transformation),
                    next_state,
                    remaining_depth - 1,
                )
            }
        }
    }

    // Returns a heurstic depth for the given pattern.
    pub fn lookup(&self, pattern: &PackedKPattern) -> usize {
        self.mutable.lookup(pattern)
    }
}
