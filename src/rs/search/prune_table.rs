use std::{rc::Rc, time::Instant};

use thousands::Separable;

use crate::{CanonicalFSMState, MoveClassIndex, PackedKPattern, CANONICAL_FSM_START_STATE};

use super::search::IDFSearchAPIData;

type PruneTableEntryType = u8;
const UNINITIALIZED_DEPTH: PruneTableEntryType = 0xff;
const MAX_PRUNE_TABLE_DEPTH: PruneTableEntryType = UNINITIALIZED_DEPTH - 1;

const PRUNE_TABLE_INDEX_MASK: usize = 0xffffff;
const DEFAULT_PRUNE_TABLE_SIZE: usize = PRUNE_TABLE_INDEX_MASK + 1;

struct PruneTableImmutableData {
    search_api_data: Rc<IDFSearchAPIData>,
}
struct PruneTableMutableData {
    current_pruning_depth: PruneTableEntryType,
    pattern_hash_to_depth: Vec<PruneTableEntryType>,
    current_depth_num_recursive_calls: usize,
}

impl PruneTableMutableData {
    fn hash_pattern(&self, pattern: &PackedKPattern) -> usize {
        (pattern.hash() as usize) & PRUNE_TABLE_INDEX_MASK // TODO: use modulo when the size is not a power of 2.
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

pub struct PruneTable {
    immutable: PruneTableImmutableData,
    mutable: PruneTableMutableData,
}

impl PruneTable {
    pub fn new(search_api_data: Rc<IDFSearchAPIData>) -> Self {
        let mut prune_table = Self {
            immutable: PruneTableImmutableData { search_api_data },
            mutable: PruneTableMutableData {
                current_pruning_depth: 0,
                pattern_hash_to_depth: vec![UNINITIALIZED_DEPTH; DEFAULT_PRUNE_TABLE_SIZE],
                current_depth_num_recursive_calls: 0,
            },
        };
        prune_table.extend_for_search_depth(0);
        prune_table
    }

    // TODO: dedup with IDFSearch?
    // TODO: Store a reference to `search_api_data` so that you can't accidentally pass in the wrong `search_api_data`?
    pub fn extend_for_search_depth(&mut self, search_depth: usize) {
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
        if new_pruning_depth <= self.mutable.current_pruning_depth {
            return;
        }

        for depth in (self.mutable.current_pruning_depth + 1)..(new_pruning_depth + 1) {
            let start_time = Instant::now();
            self.mutable.current_depth_num_recursive_calls = 0;
            println!(
                "[Prune table][Pruning depth {}] Populating prune table…",
                depth
            );
            Self::recurse(
                &self.immutable,
                &mut self.mutable,
                &self.immutable.search_api_data.target_pattern,
                CANONICAL_FSM_START_STATE,
                depth,
            );
            println!(
                "[Prune table][Pruning depth {}] {} recursive calls ({:?})",
                depth,
                self.mutable
                    .current_depth_num_recursive_calls
                    .separate_with_underscores(),
                Instant::now() - start_time
            );
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
        mutable_data.current_depth_num_recursive_calls += 1;
        if remaining_depth == 0 {
            let pattern_hash = mutable_data.hash_pattern(current_pattern);
            if mutable_data.pattern_hash_to_depth[pattern_hash] == UNINITIALIZED_DEPTH {
                mutable_data.pattern_hash_to_depth[pattern_hash] = remaining_depth;
            };
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
