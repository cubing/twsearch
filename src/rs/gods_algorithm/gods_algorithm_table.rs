use std::{collections::HashMap, vec};

use cubing::alg::Move;

use crate::{PackedKPattern, PackedKPuzzle};

pub struct GodsAlgorithmTable {
    finished: bool,
    pattern_to_depth: HashMap<PackedKPattern, /* depth */ usize>,

    depth_to_patterns: Vec<Vec<PackedKPattern>>,
}

impl GodsAlgorithmTable {
    pub fn new(packed_kpuzzle: PackedKPuzzle, move_set: Vec<Move>) -> Self {
            finished: false,
            pattern_to_depth: HashMap::new(),
            depth_to_patterns: vec![vec![packed_kpuzzle.]],
        }
    }

    pub fn fill(&self) {
        let depth = 0;
        while !self.finished {}
    }
}
