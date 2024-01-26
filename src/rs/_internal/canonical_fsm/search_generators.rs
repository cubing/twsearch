use std::collections::HashMap;

use cubing::alg::{Move, QuantumMove};
use rand::{seq::SliceRandom, thread_rng};

use crate::_internal::{
    cli::options::{Generators, MetricEnum},
    GenericPuzzle, GenericPuzzleCore, PuzzleError,
};

use super::GenericTransformationBuffer;

#[derive(Clone, Debug)]
pub struct BlankInverseInfo {}

#[derive(Clone, Debug)]
pub struct PopulatedInverseInfo<TPuzzle: GenericPuzzleCore> {
    pub inverse_transformation: TPuzzle::Transformation,
}

#[derive(Debug)]
pub struct MoveTransformationInfo<TPuzzle: GenericPuzzleCore> {
    pub r#move: Move,
    // move_class: MoveClass, // TODO: do we need this?
    // pub metric_turns: i32,
    pub transformation: TPuzzle::Transformation,
    // pub inverse_info: InverseInfo
}

// TODO: Why doesn't this work if derive `Clone` instead?
impl<TPuzzle: GenericPuzzleCore> Clone for MoveTransformationInfo<TPuzzle> {
    fn clone(&self) -> Self {
        Self {
            r#move: self.r#move.clone(),
            transformation: self.transformation.clone(),
        }
    }
}

pub type MoveTransformationMultiples<TPuzzle> = Vec<MoveTransformationInfo<TPuzzle>>;

#[derive(Debug)]
pub struct SearchGenerators<TPuzzle: GenericPuzzleCore> {
    // TODO: figure out the most reusable abstraction
    pub grouped: Vec<MoveTransformationMultiples<TPuzzle>>,
    pub flat: Vec<MoveTransformationInfo<TPuzzle>>, // TODO: avoid duplicate data
}

impl<TPuzzle: GenericPuzzleCore> Clone for SearchGenerators<TPuzzle> {
    fn clone(&self) -> Self {
        Self {
            grouped: self.grouped.clone(),
            flat: self.flat.clone(),
        }
    }
}

impl<TPuzzle: GenericPuzzleCore> SearchGenerators<TPuzzle> {
    pub fn from_grouped(
        grouped: Vec<MoveTransformationMultiples<TPuzzle>>,
        random_start: bool,
    ) -> Self {
        // let grouped_clone = grouped.iter().map(|group| group.map(|multiples|))
        let mut flat: Vec<MoveTransformationInfo<TPuzzle>> =
            grouped.iter().flatten().cloned().collect();
        if random_start {
            flat.shuffle(&mut thread_rng());
        }
        Self { grouped, flat }
    }
}

fn naïve_transformation_order<TPuzzle: GenericPuzzle>(
    identity_transformation: &TPuzzle::Transformation,
    transformation: &TPuzzle::Transformation,
) -> i32 {
    let mut order: i32 = 1;
    let mut current_transformation =
        GenericTransformationBuffer::<TPuzzle>::new(transformation.clone());
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

impl<TPuzzle: GenericPuzzle> SearchGenerators<TPuzzle> {
    pub fn try_new(
        tpuzzle: &TPuzzle,
        generators: &Generators,
        metric: &MetricEnum,
        random_start: bool,
    ) -> Result<SearchGenerators<TPuzzle>, PuzzleError> {
        let identity_transformation = TPuzzle::puzzle_identity_transformation(tpuzzle);

        let mut seen_quantum_moves = HashMap::<QuantumMove, Move>::new();

        let moves: Vec<&Move> = match generators {
            Generators::Default => TPuzzle::puzzle_definition_moves(tpuzzle),
            Generators::Custom(generators) => generators.moves.iter().collect(),
        };
        if let Generators::Custom(custom_generators) = generators {
            if !custom_generators.algs.is_empty() {
                eprintln!("WARNING: Alg generators are not implemented yet. Ignoring.");
            }
        };

        // TODO: actually calculate GCDs
        let mut grouped = Vec::<MoveTransformationMultiples<TPuzzle>>::default();
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
            let move_quantum_transformation =
                TPuzzle::puzzle_transformation_from_move(tpuzzle, &move_quantum).map_err(|e| {
                    PuzzleError {
                        description: e.to_string(), // TODO
                    }
                })?;
            let order = naïve_transformation_order::<TPuzzle>(
                &identity_transformation,
                &move_quantum_transformation,
            );

            let mut multiples = MoveTransformationMultiples::default(); // TODO: use order to set capacity.
            let move_transformation = TPuzzle::puzzle_transformation_from_move(tpuzzle, r#move)
                .map_err(|e| PuzzleError {
                    description: e.to_string(), // TODO
                })?;
            let mut move_multiple_transformation =
                GenericTransformationBuffer::<TPuzzle>::new(move_transformation.clone());

            match metric {
                MetricEnum::Hand => {
                    let mut amount: i32 = r#move.amount;
                    while move_multiple_transformation.current() != &identity_transformation {
                        let mut move_multiple = r#move.clone();
                        move_multiple.amount = canonicalize_center_amount(order, amount);
                        let transformation: &TPuzzle::Transformation =
                            move_multiple_transformation.current();
                        let transformation = transformation.clone();
                        let info = MoveTransformationInfo::<TPuzzle> {
                            r#move: move_multiple,
                            // metric_turns: 1, // TODO
                            transformation,
                        };
                        multiples.push(info.clone());

                        amount += r#move.amount;
                        move_multiple_transformation.apply_transformation(&move_transformation);
                    }
                }
                MetricEnum::Quantum => {
                    let transformation: &TPuzzle::Transformation =
                        move_multiple_transformation.current();
                    let is_self_inverse =
                        transformation == &TPuzzle::transformation_invert(transformation);
                    let transformation = transformation.clone();
                    let info = MoveTransformationInfo {
                        r#move: r#move.clone(),
                        // metric_turns: 1, // TODO
                        transformation,
                    };
                    multiples.push(info.clone());
                    if !is_self_inverse {
                        let transformation: &TPuzzle::Transformation =
                            move_multiple_transformation.current();
                        let transformation = transformation.clone();
                        let info = MoveTransformationInfo {
                            r#move: r#move.invert(),
                            // metric_turns: 1, // TODO
                            transformation,
                        };
                        multiples.push(info.clone());
                    }
                }
            }
            grouped.push(multiples);
        }
        let mut rng = thread_rng();
        if random_start {
            grouped.shuffle(&mut rng);
        }

        Ok(Self::from_grouped(grouped, random_start))
    }

    pub fn do_transformations_commute(
        t1: &TPuzzle::Transformation,
        t2: &TPuzzle::Transformation,
    ) -> bool {
        TPuzzle::transformation_apply_transformation(t1, t2)
            == TPuzzle::transformation_apply_transformation(t2, t1)
    }
}
