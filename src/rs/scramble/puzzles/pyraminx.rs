use cubing::alg::{Alg, AlgNode, Move};
use rand::{thread_rng, Rng};

use crate::{
    _internal::MoveCount,
    scramble::{randomize::PieceZeroConstraint, scramble_search::move_list_from_vec},
};

use super::{
    super::randomize::{
        randomize_orbit_naïve, OrbitOrientationConstraint, OrbitPermutationConstraint,
    },
    super::scramble_search::{generators_from_vec_str, simple_filtered_search},
    definitions::tetraminx_kpuzzle,
};

pub fn scramble_pyraminx() -> Alg {
    let kpuzzle = tetraminx_kpuzzle();
    loop {
        let mut scramble_pattern = kpuzzle.default_pattern();

        randomize_orbit_naïve(
            &mut scramble_pattern,
            0,
            "EDGES",
            OrbitPermutationConstraint::SingleOrbitEvenParity,
            OrbitOrientationConstraint::OrientationsMustSumToZero,
            PieceZeroConstraint::AnyPositionAndOrientation,
        );

        randomize_orbit_naïve(
            &mut scramble_pattern,
            1,
            "CORNERS",
            OrbitPermutationConstraint::IdentityPermutation,
            OrbitOrientationConstraint::AnySum,
            PieceZeroConstraint::AnyPositionAndOrientation,
        );

        let tip_moves = move_list_from_vec(vec!["u", "l", "r", "b"]); // TODO: cache
        let mut rng = thread_rng();
        let mut alg_nodes: Vec<AlgNode> = vec![];
        for tip_move in tip_moves {
            let amount = rng.gen_range(-1..=1);
            if amount == 0 {
                continue;
            }
            alg_nodes.push(cubing::alg::AlgNode::MoveNode(Move {
                quantum: tip_move.quantum.clone(),
                amount,
            }))
        }

        let generators = generators_from_vec_str(vec!["U", "L", "R", "B"]); // TODO: cache
        if let Some(scramble) = simple_filtered_search(
            &scramble_pattern,
            generators,
            MoveCount(6 - alg_nodes.len()),
            Some(MoveCount(11 - alg_nodes.len())),
        ) {
            let mut nodes = scramble.nodes;
            nodes.append(&mut alg_nodes);
            return Alg { nodes };
        }
    }
}
