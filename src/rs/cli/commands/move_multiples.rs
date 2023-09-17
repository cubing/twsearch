use std::collections::HashMap;

use cubing::alg::{Move, QuantumMove};

use twsearch::{PackedKPuzzle, PackedKTransformationBuffer, SearchError};

use twsearch::PackedKTransformation;

#[derive(Debug)]
pub struct MoveInfo {
    #[allow(dead_code)] // TODO
    pub(crate) r#move: Move,
    // move_class: MoveClass, // TODO: do we need this?
    // pub(crate) metric_turns: i32,
    pub(crate) transformation: PackedKTransformation,
    #[allow(dead_code)] // TODO
    pub(crate) inverse_transformation: PackedKTransformation,
}

pub type MoveMultiples = Vec<MoveInfo>;

pub fn moves_into_multiples_group(
    packed_kpuzzle: &PackedKPuzzle,
    moves: &Vec<Move>,
) -> Result<MoveMultiplesGroup, SearchError> {
    let identity_transformation =
        packed_kpuzzle
            .identity_transformation()
            .map_err(|e| SearchError {
                description: e.to_string(), // TODO
            })?;

    let mut seen_quantum_moves = HashMap::<QuantumMove, Move>::new();

    // TODO: actually calculate GCDs
    let mut move_multiples_group = MoveMultiplesGroup::default();
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

        // let f = |move_lcm| -> Result<Vec<MoveInfo>, CommandError> {
        let mut multiples = MoveMultiples::default(); // TODO: use order to set capacity.
        let move_transformation = packed_kpuzzle
            .transformation_from_move(r#move)
            .map_err(|e| SearchError {
                description: e.to_string(), // TODO
            })?;
        let mut move_multiple_transformation =
            PackedKTransformationBuffer::from(move_transformation.clone());
        let mut amount: i32 = r#move.amount;
        while move_multiple_transformation.current != identity_transformation {
            let mut move_multiple = r#move.clone();
            move_multiple.amount = amount;
            multiples.push(MoveInfo {
                r#move: move_multiple,
                // metric_turns: 1, // TODO
                transformation: move_multiple_transformation.current.clone(),
                inverse_transformation: move_multiple_transformation.current.invert(),
            });

            amount += r#move.amount;
            move_multiple_transformation.apply_transformation(&move_transformation);
        }
        move_multiples_group.push(multiples);
        // };
    }
    Ok(move_multiples_group)
}

pub type MoveMultiplesGroup = Vec<MoveMultiples>;
