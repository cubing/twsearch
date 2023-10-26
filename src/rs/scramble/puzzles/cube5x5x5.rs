use cubing::alg::{Alg, AlgNode, Move};
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::_internal::{
    options::CustomGenerators, CanonicalFSM, MoveClassIndex, SearchGenerators,
    CANONICAL_FSM_START_STATE,
};

use super::definitions::cube5x5x5_packed_kpuzzle;

const NUM_5X5X5_RANDOM_MOVES: usize = 60;

pub fn scramble_5x5x5() -> Alg {
    let packed_kpuzzle = &cube5x5x5_packed_kpuzzle();
    let moves = [
        "U", "Uw", "L", "Lw", "F", "Fw", "R", "Rw", "B", "Bw", "D", "Dw",
    ]
    .iter()
    .map(|s| s.parse::<Move>().unwrap())
    .collect();
    let generators = SearchGenerators::try_new(
        packed_kpuzzle,
        &crate::_internal::options::Generators::Custom(CustomGenerators {
            moves,
            algs: vec![],
        }),
        &crate::_internal::options::MetricEnum::Hand,
        false,
    )
    .unwrap();
    let canonical_fsm = CanonicalFSM::try_new(generators.clone()).unwrap();
    let mut current_fsm_state = CANONICAL_FSM_START_STATE;
    let mut rng = thread_rng();
    let mut nodes = Vec::<AlgNode>::default();
    for _ in 0..NUM_5X5X5_RANDOM_MOVES {
        loop {
            let move_class_index = MoveClassIndex(rng.gen_range(0..generators.grouped.len()));
            let next = canonical_fsm.next_state(current_fsm_state, move_class_index);
            if let Some(next) = next {
                nodes.push(AlgNode::MoveNode(
                    generators.grouped[move_class_index.0]
                        .choose(&mut rng)
                        .unwrap()
                        .r#move
                        .clone(),
                ));
                current_fsm_state = next;
                break;
            };
        }
    }

    Alg { nodes }
}
