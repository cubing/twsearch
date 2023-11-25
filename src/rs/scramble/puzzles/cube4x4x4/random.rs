use cubing::{alg::Alg, kpuzzle::KPattern};

use crate::scramble::{
    puzzles::definitions::cube4x4x4_kpuzzle,
    randomize::{randomize_orbit_naive, OrbitOrientationConstraint, OrbitPermutationConstraint},
};

pub fn random_4x4x4_pattern(hardcoded_scramble_alg_for_testing: Option<&Alg>) -> KPattern {
    dbg!("random_4x4x4_pattern");
    let kpuzzle = cube4x4x4_kpuzzle();
    let mut scramble_pattern = kpuzzle.default_pattern();

    match hardcoded_scramble_alg_for_testing {
        Some(hardcoded_scramble_alg_for_testing) => {
            let transformation = kpuzzle
                .transformation_from_alg(hardcoded_scramble_alg_for_testing)
                .unwrap();
            scramble_pattern = scramble_pattern.apply_transformation(&transformation);
        }
        None => {
            for orbit_info in &kpuzzle.data.ordered_orbit_info {
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
