use cubing::alg::{Alg, AlgNode, Move};
use rand::{thread_rng, Rng};

use crate::scramble::scramble_search::move_list_from_vec;

use super::{
    super::randomize::{
        randomize_orbit_naïve, OrbitOrientationConstraint, OrbitPermutationConstraint,
    },
    super::scramble_search::{filtered_search, generators_from_vec_str},
    definitions::tetraminx_kpuzzle,
};

pub fn scramble_pyraminx() -> Alg {
    let kpuzzle = tetraminx_kpuzzle();
    loop {
        let mut scramble_pattern = kpuzzle.default_pattern();

        let orbit_info = &kpuzzle.data.ordered_orbit_info[0];
        assert_eq!(orbit_info.name.0, "EDGES");
        randomize_orbit_naïve(
            &mut scramble_pattern,
            orbit_info,
            OrbitPermutationConstraint::SingleOrbitEvenParity,
            OrbitOrientationConstraint::OrientationsMustSumToZero,
        );

        let orbit_info = &kpuzzle.data.ordered_orbit_info[1];
        assert_eq!(orbit_info.name.0, "CORNERS");
        randomize_orbit_naïve(
            &mut scramble_pattern,
            orbit_info,
            OrbitPermutationConstraint::IdentityPermutation,
            OrbitOrientationConstraint::AnySum,
        );

        let tip_moves = move_list_from_vec(vec!["u", "l", "r", "b"]); // TODO: cache

        let mut rng = thread_rng();
        let generators = generators_from_vec_str(vec!["U", "L", "R", "B"]); // TODO: cache
        if let Some(scramble) = filtered_search(&scramble_pattern, generators, 4, Some(11), None) {
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
            let mut nodes = scramble.nodes;
            nodes.append(&mut alg_nodes);
            return Alg { nodes };
        }
    }
}
