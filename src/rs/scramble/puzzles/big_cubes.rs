use std::sync::OnceLock;

use cubing::{
    alg::{Alg, AlgNode, Move},
    kpuzzle::KPuzzle,
};
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::_internal::{
    canonical_fsm::{
        canonical_fsm::{CanonicalFSM, CANONICAL_FSM_START_STATE},
        move_class_mask::MoveClassIndex,
        search_generators::SearchGenerators,
    },
    cli::args::MetricEnum,
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
};

use super::{
    definitions::{cube5x5x5_kpuzzle, cube6x6x6_kpuzzle, cube7x7x7_kpuzzle},
    static_move_list::{add_random_suffixes_from, static_parsed_list, static_parsed_opt_list},
};

const NUM_5X5X5_RANDOM_MOVES: usize = 60;
const NUM_6X6X6_RANDOM_MOVES: usize = 80;
const NUM_7X7X7_RANDOM_MOVES: usize = 100;

struct ScrambleInfo<TPuzzle: SemiGroupActionPuzzle> {
    generators: SearchGenerators<TPuzzle>,
    canonical_fsm: CanonicalFSM<TPuzzle>,
}

impl<TPuzzle: SemiGroupActionPuzzle> ScrambleInfo<TPuzzle> {
    pub fn new(tpuzzle: &TPuzzle, moves: Vec<Move>) -> Self {
        let generators =
            SearchGenerators::try_new(tpuzzle, moves, &MetricEnum::Hand, false).unwrap();
        let canonical_fsm =
            CanonicalFSM::new(tpuzzle.clone(), generators.clone(), Default::default());
        Self {
            generators,
            canonical_fsm,
        }
    }
}

static CUBE5X5X5_SCRAMBLE_INFO_CELL: OnceLock<ScrambleInfo<KPuzzle>> = OnceLock::new();
pub fn scramble_5x5x5() -> Alg {
    let scramble_info = CUBE5X5X5_SCRAMBLE_INFO_CELL.get_or_init(|| {
        ScrambleInfo::new(
            cube5x5x5_kpuzzle(),
            static_parsed_list(&[
                "U", "Uw", //
                "L", "Lw", //
                "F", "Fw", //
                "R", "Rw", //
                "B", "Bw", //
                "D", "Dw", //
            ]),
        )
    });
    scramble_big_cube(scramble_info, NUM_5X5X5_RANDOM_MOVES)
}

pub fn scramble_5x5x5_bld() -> Alg {
    let s1 = static_parsed_opt_list(&["", "3Rw", "3Rw2", "3Rw'", "3Fw", "3Fw'"]);
    let s2 = static_parsed_opt_list(&["", "3Uw", "3Uw2", "3Uw'"]);
    add_random_suffixes_from(scramble_5x5x5(), [s1, s2])
}

static CUBE6X6X6_SCRAMBLE_INFO_CELL: OnceLock<ScrambleInfo<KPuzzle>> = OnceLock::new();
pub fn scramble_6x6x6() -> Alg {
    let scramble_info = CUBE6X6X6_SCRAMBLE_INFO_CELL.get_or_init(|| {
        ScrambleInfo::new(
            cube6x6x6_kpuzzle(),
            static_parsed_list(&[
                "U", "Uw", "3Uw", //
                "L", "Lw", // Avoid adjacent moves that combine into a cube rotation.
                "F", "Fw", "3Fw", //
                "R", "Rw", "3Rw", //
                "B", "Bw", // Avoid adjacent moves that combine into a cube rotation.
                "D", "Dw", // Avoid adjacent moves that combine into a cube rotation.
            ]),
        )
    });
    scramble_big_cube(scramble_info, NUM_6X6X6_RANDOM_MOVES)
}

static CUBE7X7X7_SCRAMBLE_INFO_CELL: OnceLock<ScrambleInfo<KPuzzle>> = OnceLock::new();
pub fn scramble_7x7x7() -> Alg {
    let scramble_info = CUBE7X7X7_SCRAMBLE_INFO_CELL.get_or_init(|| {
        ScrambleInfo::new(
            cube7x7x7_kpuzzle(),
            static_parsed_list(&[
                "U", "Uw", "3Uw", //
                "L", "Lw", "3Lw", //
                "F", "Fw", "3Fw", //
                "R", "Rw", "3Rw", //
                "B", "Bw", "3Bw", //
                "D", "Dw", "3Dw", //
            ]),
        )
    });
    scramble_big_cube(scramble_info, NUM_7X7X7_RANDOM_MOVES)
}

fn scramble_big_cube(scramble_info: &ScrambleInfo<KPuzzle>, num_random_moves: usize) -> Alg {
    // TODO: globally cache generators and `canonical_fsm` for each puzzle.
    let mut current_fsm_state = CANONICAL_FSM_START_STATE;
    let mut rng = thread_rng();
    let mut nodes = Vec::<AlgNode>::default();
    for _ in 0..num_random_moves {
        // TODO: we can forward-cache the valid move classes for each state instead of rejection sampling.
        loop {
            let move_class_index =
                MoveClassIndex(rng.gen_range(0..scramble_info.generators.by_move_class.len()));
            let next = scramble_info
                .canonical_fsm
                .next_state(current_fsm_state, move_class_index);
            if let Some(next) = next {
                nodes.push(AlgNode::MoveNode(
                    scramble_info.generators.by_move_class[move_class_index]
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
