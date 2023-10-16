use std::sync::OnceLock;

use cubing::{
    alg::{Alg, AlgNode, Move},
    puzzles::cube2x2x2_kpuzzle,
};
use rand::{thread_rng, Rng};

use crate::_internal::{CustomGenerators, Generators, PackedKPuzzle, PuzzleError};

use super::{
    definitions::tetraminx_kpuzzle,
    randomize::{randomize_orbit_naive, OrbitOrientationConstraint, OrbitPermutationConstraint},
    scramble_search::scramble_search,
    Event,
};

pub fn random_scramble_for_event(event: Event) -> Result<Alg, PuzzleError> {
    let err = Err(PuzzleError {
        description: format!("Scrambles are not implement for this event yet: {}", event),
    });
    match event {
        Event::Cube3x3x3Speedsolving => err,
        Event::Cube2x2x2Speedsolving => Ok(scramble_222()),
        Event::Cube4x4x4Speedsolving => err,
        Event::Cube5x5x5Speedsolving => err,
        Event::Cube6x6x6Speedsolving => err,
        Event::Cube7x7x7Speedsolving => err,
        Event::Cube3x3x3Blindfolded => err,
        Event::Cube3x3x3FewestMoves => err,
        Event::Cube3x3x3OneHanded => err,
        Event::ClockSpeedsolving => err,
        Event::MegaminxSpeedsolving => err,
        Event::PyraminxSpeedsolving => Ok(scramble_pyraminx()),
        Event::SkewbSpeedsolving => err,
        Event::Square1Speedsolving => err,
        Event::Cube4x4x4Blindfolded => err,
        Event::Cube5x5x5Blindfolded => err,
        Event::Cube3x3x3MultiBlind => err,
        Event::FTOSpeedsolving => err,
        Event::MasterTetraminxSpeedsolving => err,
        Event::KilominxSpeedsolving => err,
        Event::RediCubeSpeedsolving => err,
    }
}

fn move_list_from_vec(move_str_list: Vec<&str>) -> Vec<Move> {
    move_str_list
        .iter()
        .map(|move_str| move_str.parse::<Move>().unwrap())
        .collect()
}

fn generators_from_vec_str(move_str_list: Vec<&str>) -> Generators {
    crate::_internal::Generators::Custom(CustomGenerators {
        moves: move_list_from_vec(move_str_list),
        algs: vec![],
    })
}

static CUBE222_KPUZZLE_CELL: OnceLock<PackedKPuzzle> = OnceLock::new();
pub fn scramble_222() -> Alg {
    let packed_kpuzzle =
        CUBE222_KPUZZLE_CELL.get_or_init(|| PackedKPuzzle::try_from(cube2x2x2_kpuzzle()).unwrap());

    loop {
        let mut scramble_pattern = packed_kpuzzle.default_pattern();
        let orbit_info = &packed_kpuzzle.data.orbit_iteration_info[0];
        randomize_orbit_naive(
            &mut scramble_pattern,
            orbit_info,
            OrbitPermutationConstraint::AnyPermutation,
            OrbitOrientationConstraint::OrientationsMustSumToZero,
        );
        let generators = generators_from_vec_str(vec!["U", "L", "F", "R"]);
        if let Some(scramble) = scramble_search(&scramble_pattern, generators, 4, 11) {
            return scramble;
        }
    }
}

static TETRAMINX_KPUZZLE_CELL: OnceLock<PackedKPuzzle> = OnceLock::new();
pub fn scramble_pyraminx() -> Alg {
    let packed_kpuzzle = TETRAMINX_KPUZZLE_CELL
        .get_or_init(|| PackedKPuzzle::try_from(tetraminx_kpuzzle()).unwrap());

    loop {
        let mut scramble_pattern = packed_kpuzzle.default_pattern();

        let orbit_info = &packed_kpuzzle.data.orbit_iteration_info[0];
        assert_eq!(orbit_info.name.0, "EDGES");
        randomize_orbit_naive(
            &mut scramble_pattern,
            orbit_info,
            OrbitPermutationConstraint::SingleOrbitEvenParity,
            OrbitOrientationConstraint::OrientationsMustSumToZero,
        );

        let orbit_info = &packed_kpuzzle.data.orbit_iteration_info[1];
        assert_eq!(orbit_info.name.0, "CORNERS");
        randomize_orbit_naive(
            &mut scramble_pattern,
            orbit_info,
            OrbitPermutationConstraint::IdentityPermutation,
            OrbitOrientationConstraint::AnySum,
        );

        let tip_moves = move_list_from_vec(vec!["u", "l", "r", "b"]); // TODO: cache

        let mut rng = thread_rng();
        let generators = generators_from_vec_str(vec!["U", "L", "R", "B"]); // TODO: cache
        if let Some(scramble) = scramble_search(&scramble_pattern, generators, 4, 11) {
            let mut alg_nodes: Vec<AlgNode> = vec![];
            for tip_move in tip_moves {
                let amount = rng.gen_range(-1..2);
                if amount == 0 {
                    continue;
                }
                alg_nodes.push(cubing::alg::AlgNode::MoveNode(Move {
                    quantum: tip_move.quantum.clone(),
                    amount,
                }))
            }
            let mut nodes = scramble.nodes;
            nodes.append(&mut alg_nodes);
            return Alg { nodes };
        }
    }
}
