use std::collections::HashMap;

use cubing::alg::{Move, QuantumMove};

use crate::{
    _internal::{cli::options::MetricEnum, puzzle_traits::SemiGroupActionPuzzle, SearchError},
    index_type,
};

use super::MoveClassIndex;

index_type!(FlatMoveIndex, usize);

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
    // TODO: this should be `TPuzzle: SemiGroupActionPuzzle` but the Rust checker does not use bounds chcks.
    TPuzzle, // TODO = KPuzzle
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

impl<TPuzzle: SemiGroupActionPuzzle> SearchGenerators<TPuzzle> {
    pub fn try_new(
        tpuzzle: &TPuzzle,
        moves: Vec<&Move>,
        metric: &MetricEnum,
        random_start: bool,
    ) -> Result<SearchGenerators<TPuzzle>, SearchError> {
        let mut seen_moves = HashMap::<QuantumMove, Move>::new();

        // TODO: actually calculate GCDs
        let mut grouped = Vec::<MoveTransformationMultiples<TPuzzle>>::default();
        let mut flat = Vec::<MoveTransformationInfo<TPuzzle>>::default();
        let mut by_move =
            HashMap::<Move, (MoveClassIndex, MoveTransformationInfo<TPuzzle>)>::default();
        for (move_class_index, r#move) in moves.into_iter().enumerate() {
            let move_class_index = MoveClassIndex(move_class_index);
            if let Some(existing) = seen_moves.get(&r#move.quantum) {
                // TODO: deduplicate by quantum move.
                println!(
              "Warning: two moves with the same quantum move specified ({}, {}). This is usually redundant.",
              existing, r#move
          );
            } else {
                seen_moves.insert(r#move.quantum.as_ref().clone(), r#move.clone());
            }

            let Ok(order) = tpuzzle.move_order(r#move) else {
                return Err(SearchError {
                    description: format!("Could not calculate order for move quantum: {}", r#move),
                });
            };

            let mut multiples = MoveTransformationMultiples::default(); // TODO: use order to set capacity.

            // TODO: this should be an `Iterator` instead of a `Vec`, but this requires some type wrangling.
            let amount_iterator: Vec<i32> = match (metric, order) {
                (MetricEnum::Hand, order) => {
                    let original_amount = r#move.amount;
                    let mod_amount = order * original_amount;
                    let max_positive_amount = order / 2;
                    (original_amount..=mod_amount - original_amount)
                        .step_by(original_amount as usize)
                        .map(|amount| {
                            if amount > max_positive_amount {
                                amount - mod_amount
                            } else {
                                amount
                            }
                        })
                        .collect()
                }
                (MetricEnum::Quantum, 2 | 1) => vec![1],
                (MetricEnum::Quantum, _) => vec![1, -1],
            };

            // TODO: we've given up O(log(average move order)) performance here to make this
            // generic. If this is ever an issue, we can special-case more efficient
            // calculations.
            for amount in amount_iterator {
                let move_multiple = Move {
                    quantum: r#move.quantum.clone(),
                    amount,
                };
                let Ok(transformation) = tpuzzle.puzzle_transformation_from_move(&move_multiple)
                else {
                    return Err(SearchError {
                        description: format!(
                            "Could not get transformation for move multiple: {}",
                            move_multiple
                        ),
                    });
                };
                let info = MoveTransformationInfo {
                    r#move: move_multiple.clone(),
                    // metric_turns: 1, // TODO
                    transformation,
                    flat_move_index: FlatMoveIndex(flat.len()),
                };
                multiples.push(info.clone());
                flat.push(info.clone());
                by_move.insert(move_multiple, (move_class_index, info));
            }
            grouped.push(multiples);
        }
        // let mut rng = thread_rng();
        if random_start {
            eprintln!(
                "Randomization requires some code refactoring. Ignoring randomization paramter."
            );
            // grouped.shuffle(&mut rng);
            // flat.shuffle(&mut rng); // TODO: can we shuffle this without messing up
        }

        Ok(Self {
            by_move_class: grouped,
            flat,
            by_move,
        })
    }
}
