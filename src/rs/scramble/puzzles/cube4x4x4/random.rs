use cubing::alg::Alg;

use crate::{
    _internal::PackedKPattern,
    scramble::{
        puzzles::definitions::cube4x4x4_packed_kpuzzle,
        randomize::{
            randomize_orbit_naive, OrbitOrientationConstraint, OrbitPermutationConstraint,
        },
    },
};

pub fn random_4x4x4_pattern(hardcoded_scramble_alg_for_testing: Option<&Alg>) -> PackedKPattern {
    dbg!("random_4x4x4_pattern");
    let packed_kpuzzle = cube4x4x4_packed_kpuzzle();
    let mut scramble_pattern = packed_kpuzzle.default_pattern();

    match hardcoded_scramble_alg_for_testing {
        Some(hardcoded_scramble_alg_for_testing) => {
            let transformation = packed_kpuzzle
                .transformation_from_alg(hardcoded_scramble_alg_for_testing)
                .unwrap();
            scramble_pattern = scramble_pattern.apply_transformation(&transformation);
        }
        None => {
            for orbit_info in &packed_kpuzzle.data.orbit_iteration_info {
                randomize_orbit_naive(
                    &mut scramble_pattern,
                    orbit_info,
                    OrbitPermutationConstraint::AnyPermutation,
                    OrbitOrientationConstraint::OrientationsMustSumToZero,
                );
            }
        }
    }
    scramble_pattern
}
