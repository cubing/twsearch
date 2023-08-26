use std::collections::HashMap;

use crate::PackedKPattern;

pub struct GodsAlgorithmTable {
    finished: bool,
    depth_map: HashMap<PackedKPattern, /* depth */ usize>,

    bfs_queue: HashMap</* depth */ usize, Vec<PackedKPattern>>, // TODO: use a vector of references instead.
}

impl GodsAlgorithmTable {
    pub fn new() -> Self {
        Self {
            finished: false,
            depth_map: HashMap::new(),
            bfs_queue: HashMap::new(),
        }
    }

    pub fn fill(&self) {
        let depth = 0;
        while !self.finished {}
    }
}
