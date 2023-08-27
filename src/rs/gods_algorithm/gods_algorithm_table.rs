use std::{collections::HashMap, vec};

use cubing::alg::Move;

use crate::{ConversionError, PackedKPattern, PackedKPuzzle, PackedKTransformation};

type SearchDepth = usize;

pub struct GodsAlgorithmTable {
    completed: bool, // "completed" instead of "complete" to make an unambiguous adjective
    pattern_to_depth: HashMap<PackedKPattern, /* depth */ SearchDepth>,
}

impl GodsAlgorithmTable {
    pub fn new() -> Self {
        Self {
            completed: false,
            pattern_to_depth: HashMap::new(),
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
    cached_move_info_list: Vec<CachedMoveInfo>,

    // state
    table: GodsAlgorithmTable,
    depth_to_patterns: Vec<Vec<PackedKPattern>>, // TODO: `HashMap` instead of `Vec` for the other layer for sparse rep?
}

struct CachedMoveInfo {
    _move: Move,
    _transformation: PackedKTransformation,
    inverse_transformation: PackedKTransformation,
}

impl CachedMoveInfo {
    pub fn try_new(packed_kpuzzle: &PackedKPuzzle, r#move: Move) -> Result<Self, ConversionError> {
        Ok(Self {
            _move: r#move.clone(),
            _transformation: packed_kpuzzle.transformation_from_move(&r#move)?,
            inverse_transformation: packed_kpuzzle.transformation_from_move(&(r#move).invert())?, // TODO: invert the regular transformation directly.
        })
    }
}

impl GodsAlgorithmSearch {
    pub fn try_new(packed_kpuzzle: PackedKPuzzle, move_list: Vec<Move>) -> Result<Self, String> {
        let move_list: Result<Vec<CachedMoveInfo>, ConversionError> = move_list
            .into_iter()
            .map(|r#move| CachedMoveInfo::try_new(&packed_kpuzzle, r#move))
            .collect();
        let move_list = move_list.map_err(|e| e.to_string())?;
        let depth_to_patterns = vec![];
        Ok(Self {
            packed_kpuzzle,
            cached_move_info_list: move_list,
            table: GodsAlgorithmTable {
                completed: false,
                pattern_to_depth: HashMap::new(),
            },
            depth_to_patterns,
        })
    }

    pub fn fill(&mut self) {
        let default_pattern = self.packed_kpuzzle.default_pattern();
        self.table
            .pattern_to_depth
            .insert(default_pattern.clone(), 0);
        self.depth_to_patterns.push(vec![default_pattern]);

        let mut current_depth = 0;
        let mut num_patterns_total = 1;
        while !self.table.completed {
            let last_depth_patterns = &self.depth_to_patterns[current_depth];
            current_depth += 1;

            let mut patterns_at_current_depth = Vec::<PackedKPattern>::new();

            let mut num_patterns_at_current_depth = 0; // TODO: is it performant to just use `patterns_at_current_depth.len()`;
            for pattern in last_depth_patterns {
                for move_info in &self.cached_move_info_list {
                    let new_pattern =
                        pattern.apply_transformation(&move_info.inverse_transformation);
                    if self.table.pattern_to_depth.get(&new_pattern).is_some() {
                        continue;
                    }

                    patterns_at_current_depth.push(new_pattern.clone()); // TODO: manage lifetimes to allow pushing a reference instead.
                    self.table
                        .pattern_to_depth
                        .insert(new_pattern, current_depth);

                    num_patterns_at_current_depth += 1;
                    if num_patterns_at_current_depth % 10000 == 0 {
                        println!(
                            "Found {} total pattern{} so far.",
                            num_patterns_at_current_depth,
                            if num_patterns_at_current_depth == 1 {
                                ""
                            } else {
                                "s"
                            }
                        );
                    }
                }
            }
            self.depth_to_patterns.push(patterns_at_current_depth);

            num_patterns_total += num_patterns_at_current_depth;
            println!(
                "Found {} pattern{} at depth {} ({} at all depths so far).",
                num_patterns_at_current_depth,
                if num_patterns_at_current_depth == 1 {
                    ""
                } else {
                    "s"
                },
                current_depth,
                num_patterns_total
            );

            if num_patterns_at_current_depth == 0 {
                self.table.completed = true;
                continue;
            }
        }
        let max_depth = current_depth - 1;
        println!(
            "Found {} pattern{} with a maximum depth of {}.",
            num_patterns_total,
            if num_patterns_total == 1 { "" } else { "s" },
            max_depth
        );
    }
}
