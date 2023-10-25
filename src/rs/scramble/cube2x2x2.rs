use cubing::alg::Alg;

use super::{
    definitions::cube2x2x2_packed_kpuzzle,
    randomize::{randomize_orbit_naive, OrbitOrientationConstraint, OrbitPermutationConstraint},
    scramble_search::{filtered_search, generators_from_vec_str},
};

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
