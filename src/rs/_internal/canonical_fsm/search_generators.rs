use std::collections::HashMap;

use cubing::{
    alg::{Move, QuantumMove},
    kpuzzle::{KPuzzle, KTransformation, KTransformationBuffer},
};
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    _internal::{
        cli::options::{Generators, MetricEnum},
        SearchError,
    },
    index_type,
};

use super::MoveClassIndex;

index_type!(FlatMoveIndex);

#[derive(Clone, Debug)]
pub struct MoveTransformationInfo {
    #[allow(dead_code)] // TODO
    pub r#move: Move,
    // move_class: MoveClass, // TODO: do we need this?
    // pub metric_turns: i32,
    pub transformation: KTransformation,
    #[allow(dead_code)] // TODO
    pub inverse_transformation: KTransformation,

    pub flat_move_index: FlatMoveIndex,
}

pub type MoveTransformationMultiples = Vec<MoveTransformationInfo>;

#[derive(Clone, Debug)]
pub struct SearchGenerators {
    // TODO: figure out the most reusable abstraction
    pub by_move_class: Vec<MoveTransformationMultiples>,
    pub flat: Vec<MoveTransformationInfo>, // TODO: avoid duplicate data
    pub by_move: HashMap<Move, (MoveClassIndex, MoveTransformationInfo)>, // TODO: avoid duplicate data
}

fn transformation_order(
    identity_transformation: &KTransformation,
    transformation: &KTransformation,
) -> i32 {
    let mut order: i32 = 1;
    let mut current_transformation = KTransformationBuffer::from(transformation.clone());
    while current_transformation.current() != identity_transformation {
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

impl SearchGenerators {
    pub fn try_new(
        kpuzzle: &KPuzzle,
        generators: &Generators,
        metric: &MetricEnum,
        random_start: bool,
    ) -> Result<SearchGenerators, SearchError> {
        let identity_transformation = kpuzzle.identity_transformation();

        let mut seen_quantum_moves = HashMap::<QuantumMove, Move>::new();

        let moves: Vec<&Move> = match generators {
            Generators::Default => {
                let def = kpuzzle.definition();
                let moves = def.moves.keys();
                if let Some(derived_moves) = &def.derived_moves {
                    moves.chain(derived_moves.keys()).collect()
                } else {
                    moves.collect()
                }
            }
            Generators::Custom(generators) => generators.moves.iter().collect(),
        };
        if let Generators::Custom(custom_generators) = generators {
            if !custom_generators.algs.is_empty() {
                eprintln!("WARNING: Alg generators are not implemented yet. Ignoring.");
            }
        };

        // TODO: actually calculate GCDs
        let mut grouped = Vec::<MoveTransformationMultiples>::default();
        let mut flat = Vec::<MoveTransformationInfo>::default();
        let mut by_move = HashMap::<Move, (MoveClassIndex, MoveTransformationInfo)>::default();
        for (move_class_index, r#move) in moves.into_iter().enumerate() {
            let move_class_index = MoveClassIndex(move_class_index);
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
            let move_quantum_transformation = kpuzzle
                .transformation_from_move(&move_quantum)
                .map_err(|e| SearchError {
                    description: e.to_string(), // TODO
                })?;
            let order =
                transformation_order(&identity_transformation, &move_quantum_transformation);

            let mut multiples = MoveTransformationMultiples::default(); // TODO: use order to set capacity.
            let move_transformation =
                kpuzzle
                    .transformation_from_move(r#move)
                    .map_err(|e| SearchError {
                        description: e.to_string(), // TODO
                    })?;
            let mut move_multiple_transformation =
                KTransformationBuffer::from(move_transformation.clone());

            let mut populate_fields = |r#move: Move, transformation: &KTransformation| {
                let info = MoveTransformationInfo {
                    r#move: r#move.clone(),
                    // metric_turns: 1, // TODO
                    transformation: transformation.clone(),
                    inverse_transformation: transformation.invert(),
                    flat_move_index: FlatMoveIndex(flat.len()),
                };
                multiples.push(info.clone());
                flat.push(info.clone());
                by_move.insert(r#move, (move_class_index, info));
            };

            match metric {
                MetricEnum::Hand => {
                    let mut amount: i32 = r#move.amount;
                    while move_multiple_transformation.current() != &identity_transformation {
                        let mut move_multiple = r#move.clone();
                        move_multiple.amount = canonicalize_center_amount(order, amount);
                        populate_fields(move_multiple, move_multiple_transformation.current());

                        amount += r#move.amount;
                        move_multiple_transformation.apply_transformation(&move_transformation);
                    }
                }
                MetricEnum::Quantum => {
                    let transformation = move_multiple_transformation.current();
                    populate_fields(r#move.clone(), transformation);

                    let inverse_transformation = transformation.invert();
                    if transformation != &inverse_transformation {
                        // TODO: avoid redundant calculations?
                        populate_fields(r#move.invert(), &inverse_transformation);
                    }
                }
            }
            grouped.push(multiples);
        }
        let mut rng = thread_rng();
        if random_start {
            grouped.shuffle(&mut rng);
            flat.shuffle(&mut rng);
        }

        Ok(Self {
            by_move_class: grouped,
            flat,
            by_move,
        })
    }
}
