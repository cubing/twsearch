use cubing::alg::{Alg, AlgNode, Move};
use rand::{thread_rng, Rng};

use crate::scramble::scramble_search::move_list_from_vec;

use super::{
    definitions::tetraminx_packed_kpuzzle,
    randomize::{randomize_orbit_naive, OrbitOrientationConstraint, OrbitPermutationConstraint},
    scramble_search::{filtered_search, generators_from_vec_str},
};

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
