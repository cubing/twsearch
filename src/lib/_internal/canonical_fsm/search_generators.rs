use std::collections::HashMap;

use cubing::{
    alg::{Move, QuantumMove},
    kpuzzle::InvalidAlgError,
};

use crate::{
    _internal::{
        cli::args::MetricEnum,
        errors::SearchError,
        puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
        search::{indexed_vec::IndexedVec, move_count::MoveCount},
    },
    whole_number_newtype,
};

use super::move_class_mask::MoveClassIndex;

whole_number_newtype!(FlatMoveIndex, usize);

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
    pub move_class_index: MoveClassIndex,
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
    pub by_move_class: IndexedVec<MoveClassIndex, MoveTransformationMultiples<TPuzzle>>,
    pub flat: IndexedVec<FlatMoveIndex, MoveTransformationInfo<TPuzzle>>, // TODO: avoid duplicate data
    pub by_move: HashMap<Move, MoveTransformationInfo<TPuzzle>>, // TODO: avoid duplicate data
}

impl<TPuzzle: SemiGroupActionPuzzle> SearchGenerators<TPuzzle> {
    pub fn try_new(
        tpuzzle: &TPuzzle,
        generator_moves: Vec<Move>,
        metric: &MetricEnum,
        random_start: bool,
    ) -> Result<SearchGenerators<TPuzzle>, SearchError> {
        let mut seen_moves = HashMap::<QuantumMove, Move>::new();

        // TODO: actually calculate GCDs
        let mut by_move_class =
            IndexedVec::<MoveClassIndex, MoveTransformationMultiples<TPuzzle>>::default();
        let mut flat = IndexedVec::<FlatMoveIndex, MoveTransformationInfo<TPuzzle>>::default();
        let mut by_move = HashMap::<Move, MoveTransformationInfo<TPuzzle>>::default();
        for (move_class_index, r#move) in generator_moves.into_iter().enumerate() {
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

            let Ok(order) = tpuzzle.move_order(&r#move) else {
                return Err(SearchError {
                    description: format!("Could not calculate order for move quantum: {}", r#move),
                });
            };

            let mut multiples = MoveTransformationMultiples::default(); // TODO: use order to set capacity.

            // TODO: this should be an `Iterator` instead of a `Vec`, but this requires some type wrangling.
            let amount_iterator: Vec<i32> = match (metric, order) {
                (MetricEnum::Hand, order) => {
                    let original_amount = r#move.amount;
                    let mod_amount = (order.0 as i32) * original_amount;
                    let max_positive_amount = (order.0 as i32) / 2;
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
                (MetricEnum::Quantum, MoveCount(2) | MoveCount(1)) => vec![1],
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
                    move_class_index,
                };
                multiples.push(info.clone());
                flat.push(info.clone());
                by_move.insert(move_multiple, info);
            }
            by_move_class.push(multiples);
        }
        // let mut rng = thread_rng();
        if random_start {
            eprintln!(
                "Randomization requires some code refactoring. Ignoring randomization parameter."
            );
            // grouped.shuffle(&mut rng);
            // flat.shuffle(&mut rng); // TODO: can we shuffle this without messing up
        }

        Ok(Self {
            by_move_class,
            flat,
            by_move,
        })
    }

    #[allow(clippy::type_complexity)] // TODO
    pub fn transfer_move_classes<
        TargetTPuzzle: SemiGroupActionPuzzle<Transformation = FlatMoveIndex>,
    >(
        &self,
        target_puzzle_transformation_from_move: fn(
            r#move: &Move,
            by_move: &HashMap<Move, MoveTransformationInfo<TPuzzle>>,
        ) -> Result<
            TargetTPuzzle::Transformation,
            InvalidAlgError,
        >,
    ) -> Result<SearchGenerators<TargetTPuzzle>, InvalidAlgError> {
        let mut by_move = HashMap::<Move, MoveTransformationInfo<TargetTPuzzle>>::default();
        for (r#move, info) in &self.by_move {
            let transformation =
                target_puzzle_transformation_from_move(&info.r#move, &self.by_move)?;
            by_move.insert(
                r#move.clone(),
                MoveTransformationInfo::<TargetTPuzzle> {
                    r#move: info.r#move.clone(),
                    transformation,
                    flat_move_index: info.flat_move_index,
                    move_class_index: info.move_class_index,
                },
            );
        }

        let mut flat =
            IndexedVec::<FlatMoveIndex, MoveTransformationInfo<TargetTPuzzle>>::default();
        for info in &self.flat.0 {
            flat.push(by_move.get(&info.r#move).unwrap().clone())
        }

        let mut by_move_class =
            IndexedVec::<MoveClassIndex, MoveTransformationMultiples<TargetTPuzzle>>::default();
        for move_class_source in &self.by_move_class.0 {
            let mut move_class = Vec::default();
            for info in move_class_source {
                move_class.push(by_move.get(&info.r#move).unwrap().clone())
            }
            by_move_class.push(move_class)
        }

        Ok(SearchGenerators::<TargetTPuzzle> {
            by_move_class,
            flat,
            by_move,
        })
    }
}
