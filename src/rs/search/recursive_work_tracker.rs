use std::time::{Duration, Instant};

use thousands::Separable;

pub(crate) struct RecursiveWorkTracker {
    work_name: String,
    // TODO: support custom writes intead of sending to stdout/stderr
    latest_depth: usize,
    latest_depth_num_recursive_calls: usize,
    latest_depth_start_time: Instant,
    latest_depth_duration: Duration,
    latest_depth_finished: bool,

    previous_depth_num_recursive_calls: usize,
}

// TODO: use a logger intead of printing to stdout.
impl RecursiveWorkTracker {
    pub fn new(work_name: String) -> Self {
        Self {
            work_name,
            latest_depth: 0,
            previous_depth_num_recursive_calls: 0,
            latest_depth_start_time: Instant::now(),
            latest_depth_duration: Duration::ZERO,
            latest_depth_finished: true,
            latest_depth_num_recursive_calls: 0,
        }
    }

    pub fn start_depth(&mut self, depth: usize, message: &str) {
        self.latest_depth_start_time = Instant::now();

        self.latest_depth = depth;
        self.latest_depth_duration = Duration::ZERO;
        self.latest_depth_finished = false;

        self.previous_depth_num_recursive_calls = self.latest_depth_num_recursive_calls;
        self.latest_depth_num_recursive_calls = 0;

        println!(
            "[{}][Depth {}] {}",
            self.work_name, self.latest_depth, message,
        )
    }

    pub fn finish_latest_depth(&mut self) {
        if self.latest_depth_finished {
            eprintln!(
                "WARNING: tried to finish tracking work for depth {} multiple times.",
                self.latest_depth,
            );
        }
        self.latest_depth_duration = Instant::now() - self.latest_depth_start_time;
        let rate = (self.latest_depth_num_recursive_calls as f64
            / (self.latest_depth_duration).as_secs_f64()) as usize;
        println!(
            "[{}][Depth {}] {} recursive calls ({:?}) ({}Hz)",
            self.work_name,
            self.latest_depth,
            self.latest_depth_num_recursive_calls
                .separate_with_underscores(),
            self.latest_depth_duration,
            rate.separate_with_underscores()
        );
        self.latest_depth_finished = true;
    }

    pub fn record_recursive_call(&mut self) {
        self.latest_depth_num_recursive_calls += 1;
    }

    pub fn estimate_next_level_num_recursive_calls(&self) -> usize {
        if self.previous_depth_num_recursive_calls == 0 {
            return self.latest_depth_num_recursive_calls;
        }
        self.latest_depth_num_recursive_calls * self.latest_depth_num_recursive_calls
            / self.previous_depth_num_recursive_calls
    }
}
