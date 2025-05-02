use std::{sync::Arc, time::Duration};

use thousands::Separable;

use super::{prune_table_trait::Depth, search_logger::SearchLogger};

pub(crate) struct RecursiveWorkTracker {
    work_name: String,
    // TODO: support custom writes intead of sending to stdout/stderr
    latest_depth: Depth,
    latest_depth_num_recursive_calls: usize,
    latest_depth_start_time: instant::Instant,
    latest_depth_duration: Duration,
    latest_depth_finished: bool,

    previous_depth_num_recursive_calls: usize,

    search_logger: Arc<SearchLogger>,
}

impl RecursiveWorkTracker {
    pub fn new(work_name: String, search_logger: Arc<SearchLogger>) -> Self {
        Self {
            work_name,
            latest_depth: Depth(0),
            previous_depth_num_recursive_calls: 0,
            latest_depth_start_time: instant::Instant::now(),
            latest_depth_duration: Duration::ZERO,
            latest_depth_finished: true,
            latest_depth_num_recursive_calls: 0,
            search_logger,
        }
    }

    pub fn print_message(&self, message: &str) {
        self.search_logger
            .write_info(&format!("[{}] {}", self.work_name, message));
    }

    // Pass `None` as the message to avoid printing anything.
    pub fn start_depth(&mut self, depth: Depth, message: Option<&str>) {
        self.latest_depth_start_time = instant::Instant::now();

        self.latest_depth = depth;
        self.latest_depth_duration = Duration::ZERO;
        self.latest_depth_finished = false;

        self.previous_depth_num_recursive_calls = self.latest_depth_num_recursive_calls;
        self.latest_depth_num_recursive_calls = 0;

        if let Some(message) = message {
            self.search_logger.write_info(&format!(
                "[{}][Depth {:?}] {}",
                self.work_name, self.latest_depth, message,
            ));
        }
    }

    pub fn finish_latest_depth(&mut self) {
        if self.latest_depth_finished {
            self.search_logger.write_warning(&format!(
                "WARNING: tried to finish tracking work for depth {:?} multiple times.",
                self.latest_depth,
            ));
        }
        self.latest_depth_duration = instant::Instant::now() - self.latest_depth_start_time;
        let rate = (self.latest_depth_num_recursive_calls as f64
            / (self.latest_depth_duration).as_secs_f64()) as usize;
        self.search_logger.write_info(&format!(
            "[{}][Depth {:?}] {} recursive calls ({:?}) ({} calls/s)",
            self.work_name,
            self.latest_depth,
            self.latest_depth_num_recursive_calls
                .separate_with_underscores(),
            self.latest_depth_duration,
            rate.separate_with_underscores()
        ));
        self.latest_depth_finished = true;
    }

    pub fn record_recursive_call(&mut self) {
        self.latest_depth_num_recursive_calls += 1;
    }

    pub fn estimate_next_level_num_recursive_calls(&self) -> usize {
        if self.previous_depth_num_recursive_calls == 0 {
            return self.latest_depth_num_recursive_calls;
        }
        // TODO: do more sophisticated tracking to estimate when the branching factor heavily slows down.
        self.latest_depth_num_recursive_calls * self.latest_depth_num_recursive_calls
            / self.previous_depth_num_recursive_calls
    }
}
