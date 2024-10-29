use std::marker::PhantomData;
use std::sync::Arc;

use thousands::Separable;

use crate::_internal::puzzle_traits::SemiGroupActionPuzzle;
use crate::_internal::{
    CanonicalFSMState, MoveClassIndex, RecursiveWorkTracker, SearchLogger,
    CANONICAL_FSM_START_STATE,
};

use super::idf_search::IDFSearchAPIData;
use super::{PatternValidityChecker, PruneTable};

type PruneTableEntryType = u8;
// 0 is uninitialized, all other values are stored as 1+depth.
// This allows us to save initialization time by allowing table memory pages to start as "blank" (all 0).
const UNINITIALIZED_SENTINEL: PruneTableEntryType = 0;
const INVALID_PATTERN_SENTINEL: PruneTableEntryType = PruneTableEntryType::MAX;
const INVALID_PATTERN_DEPTH: PruneTableEntryType = INVALID_PATTERN_SENTINEL - 1;
const MAX_PRUNE_TABLE_DEPTH: PruneTableEntryType = PruneTableEntryType::MAX - 2;

const DEFAULT_MIN_PRUNE_TABLE_SIZE: usize = 1 << 20;

struct HashPruneTableImmutableData<TPuzzle: SemiGroupActionPuzzle> {
    target_pattern: TPuzzle::Pattern,
}
struct HashPruneTableMutableData<TPuzzle: SemiGroupActionPuzzle> {
    tpuzzle: TPuzzle,
    min_size: usize,               // power of 2
    prune_table_size: usize,       // power of 2
    prune_table_index_mask: usize, // prune_table_size - 1
    current_pruning_depth: PruneTableEntryType,
    pattern_hash_to_depth: Vec<PruneTableEntryType>,
    recursive_work_tracker: RecursiveWorkTracker,
    search_logger: Arc<SearchLogger>,
}

impl<TPuzzle: SemiGroupActionPuzzle> HashPruneTableMutableData<TPuzzle> {
    fn hash_pattern(&self, pattern: &TPuzzle::Pattern) -> usize {
        // TODO: use modulo when the size is not a power of 2.
        self.tpuzzle.pattern_hash_u64(pattern) as usize & self.prune_table_index_mask
    }

    // Returns a heurstic depth for the given pattern.
    fn lookup(&self, pattern: &TPuzzle::Pattern) -> usize {
        let pattern_hash = self.hash_pattern(pattern);
        let table_value = self.pattern_hash_to_depth[pattern_hash];
        if table_value == UNINITIALIZED_SENTINEL {
            (self.current_pruning_depth as usize) + 1
        } else {
            (table_value as usize) - 1
        }
    }

    fn set_if_uninitialized(&mut self, pattern: &TPuzzle::Pattern, depth: u8) {
        let pattern_hash = self.hash_pattern(pattern);
        if self.pattern_hash_to_depth[pattern_hash] == UNINITIALIZED_SENTINEL
            || self.pattern_hash_to_depth[pattern_hash] == INVALID_PATTERN_SENTINEL
        {
            self.pattern_hash_to_depth[pattern_hash] = depth + 1
        };
    }

    fn set_invalid_depth(&mut self, pattern: &TPuzzle::Pattern) {
        self.set_if_uninitialized(pattern, INVALID_PATTERN_DEPTH)
    }
}

pub struct HashPruneTable<
    TPuzzle: SemiGroupActionPuzzle,
    TPatternValidityChecker: PatternValidityChecker<TPuzzle>,
> {
    // We would store a `tpuzzle` here, but the one stored in `.mutable` is sufficient.
    immutable: HashPruneTableImmutableData<TPuzzle>,
    mutable: HashPruneTableMutableData<TPuzzle>,
    phantom_validity_checker: PhantomData<TPatternValidityChecker>,
}

impl<TPuzzle: SemiGroupActionPuzzle, TPatternValidityChecker: PatternValidityChecker<TPuzzle>>
    HashPruneTable<TPuzzle, TPatternValidityChecker>
{
    // TODO: dedup with IDFSearch?
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
        if remaining_depth == 0 {
            mutable_data.set_if_uninitialized(current_pattern, remaining_depth);
            return;
        }
        for (move_class_index, move_transformation_multiples) in immutable_data
            .search_api_data
            .search_generators
            .by_move_class
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
                let next_pattern = mutable_data.tpuzzle.pattern_apply_transformation(
                    current_pattern,
                    &move_transformation_info.transformation,
                );
                if !TPatternValidityChecker::is_valid(&next_pattern) {
                    mutable_data.set_invalid_depth(&next_pattern);
                    continue;
                }
                Self::recurse(
                    immutable_data,
                    mutable_data,
                    &next_pattern,
                    next_state,
                    remaining_depth - 1,
                )
            }
        }
    }
}

impl<TPuzzle: SemiGroupActionPuzzle, TPatternValidityChecker: PatternValidityChecker<TPuzzle>>
    PruneTable<TPuzzle> for HashPruneTable<TPuzzle, TPatternValidityChecker>
{
    fn new(
        tpuzzle: TPuzzle,
        search_api_data: Arc<IDFSearchAPIData<TPuzzle>>,
        search_logger: Arc<SearchLogger>,
        min_size: Option<usize>,
    ) -> Self {
        let min_size = match min_size {
            Some(min_size) => min_size.next_power_of_two(),
            None => DEFAULT_MIN_PRUNE_TABLE_SIZE,
        };
        let mut prune_table = Self {
            immutable: HashPruneTableImmutableData { search_api_data },
            mutable: HashPruneTableMutableData {
                tpuzzle,
                min_size,
                prune_table_size: min_size,
                prune_table_index_mask: min_size - 1,
                current_pruning_depth: 0,
                pattern_hash_to_depth: vec![0; min_size],
                recursive_work_tracker: RecursiveWorkTracker::new(
                    "Prune table".to_owned(),
                    search_logger.clone(),
                ),
                search_logger,
            },
            phantom_validity_checker: PhantomData,
        };
        prune_table.extend_for_search_depth(0, 1);
        prune_table
    }

    // Returns a heuristic depth for the given pattern.
    fn lookup(&self, pattern: &TPuzzle::Pattern) -> usize {
        self.mutable.lookup(pattern)
    }

    // TODO: dedup with IDFSearch?
    // TODO: Store a reference to `search_api_data` so that you can't accidentally pass in the wrong `search_api_data`?
    fn extend_for_search_depth(&mut self, search_depth: usize, approximate_num_entries: usize) {
        let mut new_pruning_depth =
            std::convert::TryInto::<PruneTableEntryType>::try_into(search_depth / 2)
                .expect("Prune table depth exceeded available size");
        if new_pruning_depth > MAX_PRUNE_TABLE_DEPTH {
            self.mutable.search_logger.write_warning(&format!(
                "[Prune table] Exceeded max depth, limiting to {}.",
                MAX_PRUNE_TABLE_DEPTH
            ));
            new_pruning_depth = MAX_PRUNE_TABLE_DEPTH;
        }

        let new_prune_table_size = usize::max(
            usize::next_power_of_two(approximate_num_entries),
            self.mutable.min_size,
        );
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
                    "Increasing prune table size to {} entries…",
                    new_prune_table_size.separate_with_underscores()
                ));
                self.mutable.pattern_hash_to_depth = vec![0; new_prune_table_size];
                self.mutable.prune_table_size = new_prune_table_size;
                self.mutable.prune_table_index_mask = new_prune_table_size - 1;
                self.mutable.current_pruning_depth = 0;
            }
        }

        for depth in (self.mutable.current_pruning_depth + 1)..(new_pruning_depth + 1) {
            self.mutable
                .recursive_work_tracker
                .start_depth(depth as usize, None);
            Self::recurse(
                &self.immutable,
                &mut self.mutable,
                &self.immutable.target_pattern,
                CANONICAL_FSM_START_STATE,
                depth,
            );
            self.mutable.recursive_work_tracker.finish_latest_depth();
        }
        self.mutable.current_pruning_depth = new_pruning_depth
    }
}
