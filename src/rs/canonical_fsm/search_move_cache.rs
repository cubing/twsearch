use std::collections::HashMap;

use cubing::alg::{Move, QuantumMove};

use crate::{PackedKPuzzle, PackedKTransformation, PackedKTransformationBuffer, SearchError};

#[derive(Clone, Debug)]
pub struct MoveTransformationInfo {
    #[allow(dead_code)] // TODO
    pub(crate) r#move: Move,
    // move_class: MoveClass, // TODO: do we need this?
    // pub(crate) metric_turns: i32,
    pub(crate) transformation: PackedKTransformation,
    #[allow(dead_code)] // TODO
    pub(crate) inverse_transformation: PackedKTransformation,
}

pub type MoveTransformationMultiples = Vec<MoveTransformationInfo>;

#[derive(Clone, Debug)]
pub struct SearchMoveCache {
    // TODO: figure out the most reusable abstraction
    pub(crate) grouped: Vec<MoveTransformationMultiples>,
    pub(crate) flat: Vec<MoveTransformationInfo>, // TODO: avoid duplicate data
}

fn transformation_order(
    identity_transformation: &PackedKTransformation,
    transformation: &PackedKTransformation,
) -> i32 {
    let mut order: i32 = 1;
    let mut current_transformation = PackedKTransformationBuffer::from(transformation.clone());
    println!("start");
    while &current_transformation.current != identity_transformation {
        println!("while");
        current_transformation.apply_transformation(transformation);
        order += 1;
    }
    order
}

// See: https://github.com/cubing/cubing.js/blob/145d0a7a3271a71fd1051c871bb170560561a24b/src/cubing/alg/simplify/options.ts#L15
fn canonicalize_center_amount(order: i32, amount: i32) -> i32 {
    let offset = (order - 1) / 2;
    (amount + offset).rem_euclid(order) - offset
}

impl SearchMoveCache {
    pub fn try_new(
        packed_kpuzzle: &PackedKPuzzle,
        moves: &Vec<Move>,
    ) -> Result<SearchMoveCache, SearchError> {
        let identity_transformation =
            packed_kpuzzle
                .identity_transformation()
                .map_err(|e| SearchError {
                    description: e.to_string(), // TODO
                })?;

        let mut seen_quantum_moves = HashMap::<QuantumMove, Move>::new();

        // TODO: actually calculate GCDs
        let mut grouped = Vec::<MoveTransformationMultiples>::default();
        let mut flat = Vec::<MoveTransformationInfo>::default();
        for r#move in moves {
            if let Some(existing) = seen_quantum_moves.get(&r#move.quantum) {
                // TODO: deduplicate by quantum move.
                println!(
              "Warning: two moves with the same quantum move specified ({}, {}). This is usually redundant.",
              existing, r#move
          );
            } else {
                seen_quantum_moves.insert(r#move.quantum.as_ref().clone(), r#move.clone());
            }

            let move_quantum = Move {
                quantum: r#move.quantum.clone(),
                amount: 1,
            };
            let move_quantum_transformation = packed_kpuzzle
                .transformation_from_move(&move_quantum)
                .map_err(|e| SearchError {
                    description: e.to_string(), // TODO
                })?;
            let order =
                transformation_order(&identity_transformation, &move_quantum_transformation);
            dbg!(order);

            let mut multiples = MoveTransformationMultiples::default(); // TODO: use order to set capacity.
            let move_transformation =
                packed_kpuzzle
                    .transformation_from_move(r#move)
                    .map_err(|e| SearchError {
                        description: e.to_string(), // TODO
                    })?;
            let mut move_multiple_transformation =
                PackedKTransformationBuffer::from(move_transformation.clone());
            let mut amount: i32 = r#move.amount;
            while move_multiple_transformation.current != identity_transformation {
                let mut move_multiple = r#move.clone();
                move_multiple.amount = canonicalize_center_amount(order, amount);
                let info = MoveTransformationInfo {
                    r#move: move_multiple,
                    // metric_turns: 1, // TODO
                    transformation: move_multiple_transformation.current.clone(),
                    inverse_transformation: move_multiple_transformation.current.invert(),
                };
                multiples.push(info.clone());
                flat.push(info);

                amount += r#move.amount;
                move_multiple_transformation.apply_transformation(&move_transformation);
            }
            grouped.push(multiples);
        }
        Ok(Self { grouped, flat })
    }
}
