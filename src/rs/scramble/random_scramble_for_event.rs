use lazy_static::lazy_static;
use std::sync::Mutex;

use cubing::alg::{Alg, AlgNode, Move};
use rand::{thread_rng, Rng};

use crate::{_internal::PuzzleError, scramble::scramble_search::move_list_from_vec};

use super::{
    cube3x3x3::Scramble3x3x3TwoPhase,
    definitions::{cube2x2x2_packed_kpuzzle, tetraminx_packed_kpuzzle},
    randomize::{randomize_orbit_naive, OrbitOrientationConstraint, OrbitPermutationConstraint},
    scramble_search::{filtered_search, generators_from_vec_str},
    Event,
};

pub fn random_scramble_for_event(event: Event) -> Result<Alg, PuzzleError> {
    let err = Err(PuzzleError {
        description: format!("Scrambles are not implement for this event yet: {}", event),
    });
    match event {
        Event::Cube3x3x3Speedsolving => {
            Ok(SCRAMBLE3X3X3_TWO_PHASE.lock().unwrap().scramble_3x3x3())
        }
        Event::Cube2x2x2Speedsolving => Ok(scramble_2x2x2()),
        Event::Cube4x4x4Speedsolving => err,
        Event::Cube5x5x5Speedsolving => err,
        Event::Cube6x6x6Speedsolving => err,
        Event::Cube7x7x7Speedsolving => err,
        Event::Cube3x3x3Blindfolded => err,
        Event::Cube3x3x3FewestMoves => err,
        Event::Cube3x3x3OneHanded => Ok(SCRAMBLE3X3X3_TWO_PHASE.lock().unwrap().scramble_3x3x3()),
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

// TODO: switch to `LazyLock` once that's stable: https://doc.rust-lang.org/nightly/std/cell/struct.LazyCell.html
lazy_static! {
    static ref SCRAMBLE3X3X3_TWO_PHASE: Mutex<Scramble3x3x3TwoPhase> =
        Mutex::new(Scramble3x3x3TwoPhase::default());
}

pub fn scramble_2x2x2() -> Alg {
    let packed_kpuzzle = cube2x2x2_packed_kpuzzle();
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
        if let Some(scramble) = filtered_search(&scramble_pattern, generators, Some(4), Some(11)) {
            return scramble;
        }
    }
}

pub fn scramble_pyraminx() -> Alg {
    let packed_kpuzzle = tetraminx_packed_kpuzzle();
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
        if let Some(scramble) = filtered_search(&scramble_pattern, generators, Some(4), Some(11)) {
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
