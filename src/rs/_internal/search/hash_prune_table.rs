use std::sync::Arc;

use thousands::Separable;

use crate::_internal::canonical_fsm::canonical_fsm::{
    CanonicalFSMState, CANONICAL_FSM_START_STATE,
};
use crate::_internal::puzzle_traits::puzzle_traits::{
    HashablePatternPuzzle, SemiGroupActionPuzzle,
};
use crate::whole_number_newtype;

use super::iterative_deepening::iterative_deepening_search::IterativeDeepeningSearchAPIData;
use super::iterative_deepening::search_adaptations::StoredSearchAdaptationsWithoutPruneTable;
use super::prune_table_trait::{Depth, LegacyConstructablePruneTable, PruneTable};
use super::recursive_work_tracker::RecursiveWorkTracker;
use super::search_logger::SearchLogger;

whole_number_newtype!(DepthU8, u8);

type PruneTableEntryType = DepthU8;
// 0 is uninitialized, all other values are stored as 1+depth.
// This allows us to save initialization time by allowing table memory pages to start as "blank" (all 0).
const UNINITIALIZED_SENTINEL: PruneTableEntryType = DepthU8(0);
const INVALID_PATTERN_SENTINEL: PruneTableEntryType = DepthU8(u8::MAX); // TODO: avoid harcoding `u8` here.
const INVALID_PATTERN_DEPTH: PruneTableEntryType = DepthU8(INVALID_PATTERN_SENTINEL.0 - 1);
const MAX_PRUNE_TABLE_DEPTH: PruneTableEntryType = DepthU8(u8::MAX - 2); // TODO: avoid harcoding `u8` here.

const DEFAULT_MIN_PRUNE_TABLE_SIZE: usize = 1 << 20;

struct HashPruneTableImmutableData<TPuzzle: SemiGroupActionPuzzle> {
    // TODO
    search_api_data: Arc<IterativeDeepeningSearchAPIData<TPuzzle>>,
    search_adaptations_without_prune_table: StoredSearchAdaptationsWithoutPruneTable<TPuzzle>,
}
struct HashPruneTableMutableData<TPuzzle: SemiGroupActionPuzzle + HashablePatternPuzzle> {
    tpuzzle: TPuzzle,
    min_size: usize,               // power of 2
    max_size: Option<usize>,       // power of 2
    prune_table_size: usize,       // power of 2
    prune_table_index_mask: usize, // prune_table_size - 1
    current_pruning_depth: PruneTableEntryType,
    pattern_hash_to_depth: Vec<PruneTableEntryType>,
    recursive_work_tracker: RecursiveWorkTracker,
    search_logger: Arc<SearchLogger>,
}

impl<TPuzzle: SemiGroupActionPuzzle + HashablePatternPuzzle> HashPruneTableMutableData<TPuzzle> {
    fn hash_pattern(&self, pattern: &TPuzzle::Pattern) -> usize {
        // TODO: use modulo when the size is not a power of 2.
        self.tpuzzle.pattern_hash_u64(pattern) as usize & self.prune_table_index_mask
    }

    // Returns a heurstic depth for the given pattern.
    fn lookup(&self, pattern: &TPuzzle::Pattern) -> Depth {
        let pattern_hash = self.hash_pattern(pattern);
        let table_value = self.pattern_hash_to_depth[pattern_hash];
        if table_value == UNINITIALIZED_SENTINEL {
            Depth((self.current_pruning_depth.0 as usize) + 1)
        } else {
            Depth((*table_value as usize) - 1)
        }
    }

    fn set_if_uninitialized(&mut self, pattern: &TPuzzle::Pattern, depth: DepthU8) {
        let pattern_hash = self.hash_pattern(pattern);
        if self.pattern_hash_to_depth[pattern_hash] == UNINITIALIZED_SENTINEL
            || self.pattern_hash_to_depth[pattern_hash] == INVALID_PATTERN_SENTINEL
        {
            self.pattern_hash_to_depth[pattern_hash] = DepthU8(depth.0 + 1) // TODO: arithmetic on `Depth`
        };
    }

    fn set_invalid_depth(&mut self, pattern: &TPuzzle::Pattern) {
        self.set_if_uninitialized(pattern, INVALID_PATTERN_DEPTH)
    }
}

pub struct HashPruneTable<TPuzzle: SemiGroupActionPuzzle + HashablePatternPuzzle> {
    // We would store a `tpuzzle` here, but the one stored in `.mutable` is sufficient.
    immutable: HashPruneTableImmutableData<TPuzzle>,
    mutable: HashPruneTableMutableData<TPuzzle>,
}

impl<TPuzzle: SemiGroupActionPuzzle + HashablePatternPuzzle> HashPruneTable<TPuzzle> {
    // TODO: dedup with IterativeDeepeningSearch?
    // TODO: Store a reference to `search_api_data` so that you can't accidentally pass in the wrong `search_api_data`?
    fn recurse(
        immutable_data: &HashPruneTableImmutableData<TPuzzle>,
        mutable_data: &mut HashPruneTableMutableData<TPuzzle>,
        // TODO: Use a `PatternStack` to avoid allocations.
        current_pattern: &TPuzzle::Pattern,
        current_state: CanonicalFSMState,
        remaining_depth: PruneTableEntryType,
    ) {
        mutable_data.recursive_work_tracker.record_recursive_call();
        if remaining_depth == DepthU8(0) {
            mutable_data.set_if_uninitialized(current_pattern, remaining_depth);
            return;
        }
        for (move_class_index, move_transformation_multiples) in immutable_data
            .search_api_data
            .search_generators
            .by_move_class
            .iter()
        {
            let next_state = match immutable_data
                .search_api_data
                .canonical_fsm
                .next_state(current_state, move_class_index)
            {
                Some(next_state) => next_state,
                None => {
                    continue;
                }
            };

            for move_transformation_info in move_transformation_multiples {
                let Some(next_pattern) = mutable_data.tpuzzle.pattern_apply_transformation(
                    current_pattern,
                    &move_transformation_info.transformation,
                ) else {
                    continue;
                };

                if immutable_data
                    .search_adaptations_without_prune_table
                    .filter_pattern(&next_pattern)
                    .is_reject()
                {
                    mutable_data.set_invalid_depth(&next_pattern);
                    continue;
                }
                Self::recurse(
                    immutable_data,
                    mutable_data,
                    &next_pattern,
                    next_state,
                    remaining_depth - DepthU8(1),
                )
            }
        }
    }
}

fn previous_power_of_two(n: usize) -> usize {
    if n.is_power_of_two() {
        n
    } else {
        // `.prev_power_of_two()` doesn't exist yet: https://internals.rust-lang.org/t/add-prev-power-of-two/14281
        (n / 2).next_power_of_two()
    }
}

impl<TPuzzle: SemiGroupActionPuzzle + HashablePatternPuzzle> LegacyConstructablePruneTable<TPuzzle>
    for HashPruneTable<TPuzzle>
{
    fn new(
        tpuzzle: TPuzzle,
        search_api_data: Arc<IterativeDeepeningSearchAPIData<TPuzzle>>,
        search_logger: Arc<SearchLogger>,
        min_size: Option<usize>,
        max_size: Option<usize>,
        search_adaptations_without_prune_table: StoredSearchAdaptationsWithoutPruneTable<TPuzzle>,
    ) -> Self {
        let min_size = match min_size {
            Some(min_size) => min_size.next_power_of_two(),
            None => DEFAULT_MIN_PRUNE_TABLE_SIZE,
        };
        // Note: we could return a max size, but there are some issues with calculating this statically due to the variable width of `usize`.
        let max_size = max_size.map(previous_power_of_two);
        let mut prune_table = Self {
            immutable: HashPruneTableImmutableData {
                search_api_data,
                search_adaptations_without_prune_table,
            },
            mutable: HashPruneTableMutableData {
                tpuzzle,
                min_size,
                max_size,
                prune_table_size: min_size,
                prune_table_index_mask: min_size - 1,
                current_pruning_depth: DepthU8(0),
                pattern_hash_to_depth: vec![DepthU8(0); min_size],
                recursive_work_tracker: RecursiveWorkTracker::new(
                    "Prune table".to_owned(),
                    search_logger.clone(),
                ),
                search_logger,
            },
        };
        prune_table.extend_for_search_depth(Depth(0), 1);
        prune_table
    }
}

impl<TPuzzle: SemiGroupActionPuzzle + HashablePatternPuzzle> PruneTable<TPuzzle>
    for HashPruneTable<TPuzzle>
{
    // Returns a heuristic depth for the given pattern.
    fn lookup(&self, pattern: &TPuzzle::Pattern) -> Depth {
        self.mutable.lookup(pattern)
    }

    // TODO: dedup with IterativeDeepeningSearch?
    // TODO: Store a reference to `search_api_data` so that you can't accidentally pass in the wrong `search_api_data`?
    fn extend_for_search_depth(&mut self, search_depth: Depth, approximate_num_entries: usize) {
        let mut new_pruning_depth = DepthU8(
            std::convert::TryInto::<u8>::try_into(search_depth.0 / 2)
                .expect("Prune table depth exceeded available size"),
        );
        if new_pruning_depth > MAX_PRUNE_TABLE_DEPTH {
            self.mutable.search_logger.write_warning(&format!(
                "[Prune table] Exceeded max depth, limiting to {:?}.",
                MAX_PRUNE_TABLE_DEPTH
            ));
            new_pruning_depth = MAX_PRUNE_TABLE_DEPTH;
        }

        let new_prune_table_size = usize::max(
            usize::next_power_of_two(approximate_num_entries),
            self.mutable.min_size,
        );
        let new_prune_table_size = match self.mutable.max_size {
            Some(max_size) => usize::min(new_prune_table_size, max_size),
            None => new_prune_table_size,
        };
        match new_prune_table_size.cmp(&self.mutable.prune_table_size) {
            std::cmp::Ordering::Less => {
                // Don't shrink the prune table.
                return;
            }
            std::cmp::Ordering::Equal => {
                if new_pruning_depth <= self.mutable.current_pruning_depth {
                    return;
                }
            }
            std::cmp::Ordering::Greater => {
                self.mutable.recursive_work_tracker.print_message(&format!(
                    "Increasing hash prune table size to {} entries…",
                    new_prune_table_size.separate_with_underscores()
                ));
                self.mutable.pattern_hash_to_depth = vec![DepthU8(0); new_prune_table_size];
                self.mutable.prune_table_size = new_prune_table_size;
                self.mutable.prune_table_index_mask = new_prune_table_size - 1;
                self.mutable.current_pruning_depth = DepthU8(0);
            }
        }

        for depth_as_u8 in (*self.mutable.current_pruning_depth + 1)..(*new_pruning_depth + 1) {
            let depth = DepthU8(depth_as_u8);
            self.mutable
                .recursive_work_tracker
                .start_depth(Depth(*depth as usize), None);
            for target_pattern in &self.immutable.search_api_data.target_patterns {
                Self::recurse(
                    &self.immutable,
                    &mut self.mutable,
                    target_pattern,
                    CANONICAL_FSM_START_STATE,
                    depth,
                );
            }
            self.mutable.recursive_work_tracker.finish_latest_depth();
        }
        self.mutable.current_pruning_depth = new_pruning_depth
    }
}
