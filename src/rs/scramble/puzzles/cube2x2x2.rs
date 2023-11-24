use cubing::{alg::Alg, puzzles::cube2x2x2_kpuzzle};

use super::{
    super::randomize::{
        randomize_orbit_naive, OrbitOrientationConstraint, OrbitPermutationConstraint,
    },
    super::scramble_search::{filtered_search, generators_from_vec_str},
};

pub fn scramble_2x2x2() -> Alg {
    let kpuzzle = cube2x2x2_kpuzzle();
    loop {
        let mut scramble_pattern = kpuzzle.default_pattern();
        let orbit_info = &kpuzzle.data.orbit_iteration_info[0];
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
