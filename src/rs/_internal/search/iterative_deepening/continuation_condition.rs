use std::fmt::Debug;

use cubing::alg::Move;
use serde::{Deserialize, Serialize};

use crate::_internal::search::{
    iterative_deepening::solution_moves::alg_from_moves, prune_table_trait::Depth,
};

// TODO: also handle "before" cases.
#[derive(Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum ContinuationCondition {
    #[default]
    None,
    // An empty `Vec` in the base case means a solution check shall be performed.
    At(Vec<Move>),
    // An empty `Vec` in the base case means a solution check shall not be performed.
    After(Vec<Move>),
}

impl ContinuationCondition {
    pub fn min_depth(&self) -> Depth {
        match self {
            ContinuationCondition::None => Depth(0),
            ContinuationCondition::At(moves) => Depth(moves.len()),
            ContinuationCondition::After(moves) => Depth(moves.len()),
        }
    }

    /// A return value of `None` indicates to avoid recursing.
    /// A return value of `Some(…)` indicates to recurse using the given (potential) move.
    pub fn recurse(&self, potential_move: &Move) -> Option<ContinuationCondition> {
        match self {
            ContinuationCondition::None => Some(ContinuationCondition::None),
            ContinuationCondition::At(moves) => {
                if let Some((first, rest)) = moves.split_first() {
                    if first == potential_move {
                        // eprintln!("Move: {}", first);
                        Some(ContinuationCondition::At(rest.to_vec()))
                    } else {
                        // eprintln!("skippin' {}", potential_move);
                        None
                    }
                } else {
                    // eprintln!("at empty {}", potential_move);
                    Some(ContinuationCondition::None)
                }
            }
            ContinuationCondition::After(moves) => {
                if let Some((first, rest)) = moves.split_first() {
                    if first == potential_move {
                        // eprintln!("Move: {}", first);
                        Some(ContinuationCondition::After(rest.to_vec()))
                    } else {
                        // eprintln!("skippin' {}", potential_move);
                        None
                    }
                } else {
                    // eprintln!("after empty {}", potential_move);
                    Some(ContinuationCondition::None)
                }
            }
        }
    }
}

impl Debug for ContinuationCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContinuationCondition::None => write!(f, "ContinuationCondition::None"),
            ContinuationCondition::At(moves) => {
                write!(
                    f,
                    "ContinuationCondition::At(parse_alg!({:?}))",
                    alg_from_moves(moves).to_string()
                )
            }
            ContinuationCondition::After(moves) => {
                write!(
                    f,
                    "ContinuationCondition::After(parse_alg!({:?}))",
                    alg_from_moves(moves).to_string()
                )
            }
        }
    }
}
