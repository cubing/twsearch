use std::collections::HashMap;

use cubing::{
    alg::{Move, QuantumMove},
    kpuzzle::KTransformationBuffer,
};
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    _internal::{
        cli::options::{Generators, MetricEnum},
        GroupActionPuzzle, SearchError, SemiGroupActionPuzzle,
    },
    index_type,
};

use super::MoveClassIndex;

index_type!(FlatMoveIndex);

#[derive(Clone, Debug)]
pub struct MoveTransformationInfo<
    TPuzzle: SemiGroupActionPuzzle, // TODO = KPuzzle
> {
    #[allow(dead_code)] // TODO
    pub r#move: Move,
    // move_class: MoveClass, // TODO: do we need this?
    // pub metric_turns: i32,
    pub transformation: TPuzzle::Transformation,
    // #[allow(dead_code)] // TODO
    // pub inverse_transformation: TPuzzle::Transformation,
    pub flat_move_index: FlatMoveIndex,
}

pub type MoveTransformationMultiples<
    TPuzzle: SemiGroupActionPuzzle, // TODO = KPuzzle
> = Vec<MoveTransformationInfo<TPuzzle>>;

#[derive(Clone, Debug)]
pub struct SearchGenerators<
    TPuzzle: SemiGroupActionPuzzle, // TODO = KPuzzle
> {
    // TODO: figure out the most reusable abstraction
    pub by_move_class: Vec<MoveTransformationMultiples<TPuzzle>>,
    pub flat: Vec<MoveTransformationInfo<TPuzzle>>, // TODO: avoid duplicate data
    pub by_move: HashMap<Move, (MoveClassIndex, MoveTransformationInfo<TPuzzle>)>, // TODO: avoid duplicate data
}

// See: https://github.com/cubing/cubing.js/blob/145d0a7a3271a71fd1051c871bb170560561a24b/src/cubing/alg/simplify/options.ts#L15
fn canonicalize_center_amount(order: i32, amount: i32) -> i32 {
    let offset = (order - 1) / 2;
    (amount + offset).rem_euclid(order) - offset
}

impl<
        TPuzzle: GroupActionPuzzle, // TODO: Make this work for SemiGroupAction
    > SearchGenerators<TPuzzle>
{
    pub fn try_new(
        tpuzzle: &TPuzzle,
        generators: &Generators,
        metric: &MetricEnum,
        random_start: bool,
    ) -> Result<SearchGenerators<TPuzzle>, SearchError> {
        let identity_transformation = tpuzzle.puzzle_identity_transformation();

        let mut seen_quantum_moves = HashMap::<QuantumMove, Move>::new();

        let moves: Vec<&Move> = match generators {
            Generators::Default => tpuzzle.puzzle_definition_all_moves(),
            Generators::Custom(generators) => generators.moves.iter().collect(),
        };
        if let Generators::Custom(custom_generators) = generators {
            if !custom_generators.algs.is_empty() {
                eprintln!("WARNING: Alg generators are not implemented yet. Ignoring.");
            }
        };

        // TODO: actually calculate GCDs
        let mut grouped = Vec::<MoveTransformationMultiples<TPuzzle>>::default();
        let mut flat = Vec::<MoveTransformationInfo<TPuzzle>>::default();
        let mut by_move =
            HashMap::<Move, (MoveClassIndex, MoveTransformationInfo<TPuzzle>)>::default();
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
            let Ok(order) = tpuzzle.move_order(&move_quantum) else {
                return Err(SearchError {
                    description: format!(
                        "Could not calculate order for move quantum: {}",
                        move_quantum
                    ),
                });
            };

            let mut multiples = MoveTransformationMultiples::default(); // TODO: use order to set capacity.
            let move_transformation =
                tpuzzle
                    .puzzle_transformation_from_move(r#move)
                    .map_err(|e| SearchError {
                        description: e.to_string(), // TODO
                    })?;
            // let mut move_multiple_transformation =
            //     KTransformationBuffer::from(move_transformation.clone());

            let mut move_multiple_transformation = tpuzzle
                .puzzle_transformation_from_move(&Move {
                    quantum: r#move.quantum,
                    amount: r#move.amount * ,
                })
                .map_err(|e| SearchError {
                    description: e.to_string(), // TODO
                })?;

            let mut populate_fields = |r#move: Move, transformation: &TPuzzle::Transformation| {
                let info = MoveTransformationInfo {
                    r#move: r#move.clone(),
                    // metric_turns: 1, // TODO
                    transformation: transformation.clone(),
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
