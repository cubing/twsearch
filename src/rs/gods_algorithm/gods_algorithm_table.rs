use std::{mem::swap, time::Instant, vec};

use thousands::Separable;

use cubing::alg::Move;

use crate::{
    gods_algorithm::factor_number::factor_number, CanonicalFSM, PackedKPattern, PackedKPuzzle,
    SearchError, SearchMoveCache, CANONICAL_FSM_START_STATE,
};

// type SearchDepth = usize; // TODO

use indicatif::{MultiProgress, ProgressBar, ProgressFinish, ProgressStyle};

use super::{bulk_queue::BulkQueue, queue_item::QueueItem};

pub struct GodsAlgorithmTable {
    completed: bool, // "completed" instead of "complete" to make an unambiguous adjective
                     // TODO: optionally populate the following:
                     // pattern_to_depth: HashMap<PackedKPattern, /* depth */ SearchDepth>,
}

impl GodsAlgorithmTable {
    pub fn new() -> Self {
        Self {
            completed: false,
            // pattern_to_depth: HashMap::new(),
        }
    }
}

impl Default for GodsAlgorithmTable {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GodsAlgorithmSearch {
    // params
    packed_kpuzzle: PackedKPuzzle,
    start_pattern: Option<PackedKPattern>,
    search_moves: SearchMoveCache,

    // state
    canonical_fsm: CanonicalFSM,
    table: GodsAlgorithmTable,
    previous_queue: BulkQueue<QueueItem>,
    current_queue: BulkQueue<QueueItem>,
    next_queue: BulkQueue<QueueItem>,

    multi_progress_bar: MultiProgress,
}

macro_rules! format_num {
    ($n:expr) => {
        $n.separate_with_underscores()
    };
}

impl GodsAlgorithmSearch {
    pub fn try_new(
        packed_kpuzzle: PackedKPuzzle,
        start_pattern: Option<PackedKPattern>,
        move_list: Vec<Move>,
    ) -> Result<Self, SearchError> {
        let search_moves = SearchMoveCache::try_new(&packed_kpuzzle, &move_list)?;
        let canonical_fsm = CanonicalFSM::try_new(search_moves.clone())?;
        Ok(Self {
            packed_kpuzzle,
            start_pattern,
            search_moves,
            canonical_fsm,
            table: GodsAlgorithmTable { completed: false },
            previous_queue: BulkQueue { list: vec![] },
            current_queue: BulkQueue { list: vec![] },
            next_queue: BulkQueue { list: vec![] },
            multi_progress_bar: MultiProgress::new(),
        })
    }

    pub fn fill(&mut self) {
        let start_pattern = match &self.start_pattern {
            Some(start_pattern) => start_pattern.clone(),
            None => self.packed_kpuzzle.default_pattern(),
        };
        let start_item = QueueItem {
            canonical_fsm_state: CANONICAL_FSM_START_STATE,
            pattern: start_pattern,
        };
        self.next_queue.push(start_item);

        let mut current_depth = 0;
        let mut num_patterns_total = 0;

        let start_time = Instant::now();
        while !self.table.completed {
            let mut swap_queue = BulkQueue::default();
            swap(&mut self.next_queue, &mut swap_queue);
            swap(&mut self.current_queue, &mut swap_queue);
            swap(&mut self.previous_queue, &mut swap_queue);
            drop(swap_queue);

            let num_queue_patterns = self.current_queue.size();

            let progress_bar = ProgressBar::new(num_queue_patterns.try_into().unwrap());
            let progress_bar = self.multi_progress_bar.insert_from_back(0, progress_bar);
            let progress_bar = progress_bar.with_finish(ProgressFinish::AndLeave);
            // TODO share the progress bar style?
            let progress_bar_style = ProgressStyle::with_template(
                "{prefix:3} {bar:12.cyan/blue} {elapsed:.2} {wide_msg}",
            )
            .expect("Could not construct progress bar.");
            // .progress_chars("=> ");
            progress_bar.set_style(progress_bar_style);
            progress_bar.set_prefix(current_depth.to_string());

            let num_to_test_at_current_depth: usize =
                num_queue_patterns * self.search_moves.flat.len();
            let mut num_tested_at_current_depth = 0;
            for queue_item in self.current_queue.iter() {
                for move_class_index in &self.canonical_fsm.move_class_indices {
                    let moves_in_class = &self.search_moves.grouped[move_class_index.0];
                    let next_state = self
                        .canonical_fsm
                        .next_state(queue_item.canonical_fsm_state, *move_class_index);
                    let next_state = match next_state {
                        Some(next_state) => next_state,
                        None => {
                            num_tested_at_current_depth += moves_in_class.len();
                            continue;
                        }
                    };
                    for move_info in moves_in_class {
                        num_tested_at_current_depth += 1;
                        let new_pattern = queue_item
                            .pattern
                            .apply_transformation(&move_info.inverse_transformation);
                        let new_item = QueueItem {
                            canonical_fsm_state: next_state,
                            pattern: new_pattern.clone(),
                        };
                        self.next_queue.push(new_item);
                        // self.table
                        //     .pattern_to_depth
                        //     .insert(new_pattern, current_depth);

                        if num_tested_at_current_depth % 1000 == 0 {
                            progress_bar
                                .set_length(num_to_test_at_current_depth.try_into().unwrap());
                            progress_bar.set_position(num_tested_at_current_depth as u64);
                            progress_bar.set_message(format!(
                                "{} patterns ({} cumulative) — {} remaining candidates",
                                format_num!(self.current_queue.size()),
                                format_num!(num_patterns_total + self.current_queue.size()), // TODO: increment before
                                format_num!(
                                    num_to_test_at_current_depth
                                        - std::convert::TryInto::<usize>::try_into(
                                            num_tested_at_current_depth
                                        )
                                        .unwrap()
                                )
                            ))
                        }
                    }
                }
            }

            let mut swap_queue = BulkQueue::default();
            swap(&mut self.next_queue, &mut swap_queue);
            // TODO: why can't we consume and replace `self.next_queue` directly?
            progress_bar.set_message(format!(
                "{} patterns ({} cumulative) — merging…",
                format_num!(self.current_queue.size()),
                format_num!(num_patterns_total)
            ));
            self.next_queue = swap_queue.sort_and_dedup(&self.previous_queue, &self.current_queue);
            progress_bar.set_message(format!(
                "{} patterns ({} cumulative) — merged!",
                format_num!(self.current_queue.size()),
                format_num!(num_patterns_total)
            ));
            // dbg!(&self.previous_queue);
            // dbg!(&self.current_queue);
            // dbg!(&self.next_queue);

            let num_patterns_at_current_depth = self.current_queue.size();
            num_patterns_total += num_patterns_at_current_depth;
            {
                progress_bar.set_length(self.current_queue.size().try_into().unwrap());
                progress_bar.set_position(self.current_queue.size().try_into().unwrap());
                progress_bar.set_message(format!(
                    "{} patterns ({} cumulative)",
                    format_num!(num_patterns_at_current_depth),
                    format_num!(num_patterns_total)
                ))
            }

            if num_patterns_at_current_depth == 0 {
                progress_bar.finish_and_clear();
                self.table.completed = true;
            } else {
                progress_bar.finish();
            }
            current_depth += 1;
        }
        let max_depth = current_depth - 1;
        println!();
        println!();
        println!(
            "Found {} ({}) pattern{}.\nMaximum depth: {} moves\nTotal time elapsed: {:?}",
            format_num!(num_patterns_total),
            factor_number(num_patterns_total.try_into().unwrap()),
            if num_patterns_total == 1 { "" } else { "s" },
            max_depth,
            Instant::now() - start_time
        );
    }
}
