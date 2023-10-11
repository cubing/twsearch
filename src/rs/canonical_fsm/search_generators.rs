use std::collections::HashMap;

use cubing::alg::{Move, QuantumMove};
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    PackedKPuzzle, PackedKTransformation, PackedKTransformationBuffer, SearchError,
    _internal::cli::{Generators, MetricEnum},
};

#[derive(Clone, Debug)]
pub struct MoveTransformationInfo {
    #[allow(dead_code)] // TODO
    pub r#move: Move,
    // move_class: MoveClass, // TODO: do we need this?
    // pub metric_turns: i32,
    pub transformation: PackedKTransformation,
    #[allow(dead_code)] // TODO
    pub inverse_transformation: PackedKTransformation,
}

pub type MoveTransformationMultiples = Vec<MoveTransformationInfo>;

#[derive(Clone, Debug)]
pub struct SearchGenerators {
    // TODO: figure out the most reusable abstraction
    pub grouped: Vec<MoveTransformationMultiples>,
    pub flat: Vec<MoveTransformationInfo>, // TODO: avoid duplicate data
}

fn transformation_order(
    identity_transformation: &PackedKTransformation,
    transformation: &PackedKTransformation,
) -> i32 {
    let mut order: i32 = 1;
    let mut current_transformation = PackedKTransformationBuffer::from(transformation.clone());
    while &current_transformation.current != identity_transformation {
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
        packed_kpuzzle: &PackedKPuzzle,
        generators: &Generators,
        metric: &MetricEnum,
        random_start: bool,
    ) -> Result<SearchGenerators, SearchError> {
        let identity_transformation =
            packed_kpuzzle
                .identity_transformation()
                .map_err(|e| SearchError {
                    description: e.to_string(), // TODO
                })?;

        let mut seen_quantum_moves = HashMap::<QuantumMove, Move>::new();

        let moves: Vec<&Move> = match generators {
            Generators::Default => {
                let def = packed_kpuzzle.data.kpuzzle.definition();
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

            let mut multiples = MoveTransformationMultiples::default(); // TODO: use order to set capacity.
            let move_transformation =
                packed_kpuzzle
                    .transformation_from_move(r#move)
                    .map_err(|e| SearchError {
                        description: e.to_string(), // TODO
                    })?;
            let mut move_multiple_transformation =
                PackedKTransformationBuffer::from(move_transformation.clone());

            match metric {
                MetricEnum::Hand => {
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
                }
                MetricEnum::Quantum => {
                    let info = MoveTransformationInfo {
                        r#move: r#move.clone(),
                        // metric_turns: 1, // TODO
                        transformation: move_multiple_transformation.current.clone(),
                        inverse_transformation: move_multiple_transformation.current.invert(),
                    };
                    let is_self_inverse = info.transformation == info.inverse_transformation;
                    multiples.push(info.clone());
                    flat.push(info);
                    if !is_self_inverse {
                        let info = MoveTransformationInfo {
                            r#move: r#move.invert(),
                            // metric_turns: 1, // TODO
                            transformation: move_multiple_transformation.current.invert(),
                            inverse_transformation: move_multiple_transformation.current.clone(),
                        };
                        multiples.push(info.clone());
                        flat.push(info);
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

        Ok(Self { grouped, flat })
    }
}
