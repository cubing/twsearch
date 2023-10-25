use cubing::alg::Alg;

use crate::{
    _internal::{IDFSearch, IndividualSearchOptions, PackedKPattern, PackedKPuzzle},
    scramble::{
        randomize::{basic_parity, BasicParity},
        scramble_search::{basic_idfs, idfs_with_target_pattern},
    },
};

use super::{
    super::randomize::{
        randomize_orbit_naive, OrbitOrientationConstraint, OrbitPermutationConstraint,
    },
    super::scramble_search::generators_from_vec_str,
    definitions::{cube3x3x3_centerless_packed_kpuzzle, cube3x3x3_g1_target_pattern},
};

pub struct Scramble3x3x3TwoPhase {
    packed_kpuzzle: PackedKPuzzle,

    filtering_idfs: IDFSearch,

    phase1_target_pattern: PackedKPattern,
    phase1_idfs: IDFSearch,

    phase2_idfs: IDFSearch,
}

impl Default for Scramble3x3x3TwoPhase {
    fn default() -> Self {
        let packed_kpuzzle = cube3x3x3_centerless_packed_kpuzzle();
        let generators = generators_from_vec_str(vec!["U", "L", "F", "R", "B", "D"]);
        let filtering_idfs = basic_idfs(&packed_kpuzzle, generators.clone(), Some(32));

        let phase1_target_pattern = cube3x3x3_g1_target_pattern();
        let phase1_idfs = idfs_with_target_pattern(
            &packed_kpuzzle,
            generators.clone(),
            phase1_target_pattern.clone(),
            Some(1 << 24),
        );

        let phase2_generators = generators_from_vec_str(vec!["U", "L2", "F2", "R2", "B2", "D"]);
        let phase2_idfs = idfs_with_target_pattern(
            &packed_kpuzzle,
            phase2_generators.clone(),
            packed_kpuzzle.default_pattern(),
            Some(1 << 24),
        );

        Self {
            packed_kpuzzle,
            filtering_idfs,

            phase1_target_pattern,
            phase1_idfs,

            phase2_idfs,
        }
    }
}

pub fn random_3x3x3_pattern() -> PackedKPattern {
    let packed_kpuzzle = cube3x3x3_centerless_packed_kpuzzle();
    let mut scramble_pattern = packed_kpuzzle.default_pattern();
    let orbit_info = &packed_kpuzzle.data.orbit_iteration_info[0];
    assert_eq!(orbit_info.name.0, "EDGES");
    let edge_order = randomize_orbit_naive(
        &mut scramble_pattern,
        orbit_info,
        OrbitPermutationConstraint::AnyPermutation,
        OrbitOrientationConstraint::OrientationsMustSumToZero,
    );
    let each_orbit_parity = basic_parity(&edge_order);
    let orbit_info = &packed_kpuzzle.data.orbit_iteration_info[1];
    assert_eq!(orbit_info.name.0, "CORNERS");
    randomize_orbit_naive(
        &mut scramble_pattern,
        orbit_info,
        match each_orbit_parity {
            BasicParity::Even => OrbitPermutationConstraint::SingleOrbitEvenParity,
            BasicParity::Odd => OrbitPermutationConstraint::SingleOrbitOddParity,
        },
        OrbitOrientationConstraint::OrientationsMustSumToZero,
    );
    scramble_pattern
}

impl Scramble3x3x3TwoPhase {
    pub fn solve_3x3x3_pattern(&mut self, pattern: &PackedKPattern) -> Alg {
        let phase1_alg = {
            let phase1_search_pattern = self.phase1_target_pattern.clone();
            for orbit_info in &self.packed_kpuzzle.data.orbit_iteration_info {
                for i in 0..orbit_info.num_pieces {
                    let old_piece = pattern
                        .packed_orbit_data
                        .get_packed_piece_or_permutation(orbit_info, i);
                    let old_piece_mapped = self
                        .phase1_target_pattern
                        .packed_orbit_data
                        .get_packed_piece_or_permutation(orbit_info, old_piece as usize);
                    phase1_search_pattern
                        .packed_orbit_data
                        .set_packed_piece_or_permutation(orbit_info, i, old_piece_mapped);
                    let ori = pattern
                        .packed_orbit_data
                        .get_packed_orientation(orbit_info, i);
                    phase1_search_pattern
                        .packed_orbit_data
                        .set_packed_orientation(orbit_info, i, ori);
                }
            }

            self.phase1_idfs
                .search(
                    &phase1_search_pattern,
                    IndividualSearchOptions {
                        min_num_solutions: Some(1),
                        min_depth: None,
                        max_depth: None,
                    },
                )
                .next()
                .unwrap()
        };

        let mut phase2_alg = {
            let phase2_search_pattern = pattern.apply_transformation(
                &self
                    .packed_kpuzzle
                    .transformation_from_alg(&phase1_alg)
                    .unwrap(),
            );
            self.phase2_idfs
                .search(
                    &phase2_search_pattern,
                    IndividualSearchOptions {
                        min_num_solutions: Some(1),
                        min_depth: None,
                        max_depth: None,
                    },
                )
                .next()
                .unwrap()
        };

        let mut nodes = phase1_alg.nodes;
        nodes.append(&mut phase2_alg.nodes);
        Alg { nodes }
    }

    // TODO: rely on the main search to find patterns at a low depth?
    pub fn is_valid_scramble_pattern(&mut self, pattern: &PackedKPattern) -> bool {
        self.filtering_idfs
            .search(
                pattern,
                IndividualSearchOptions {
                    min_num_solutions: Some(1),
                    min_depth: Some(0),
                    max_depth: Some(2),
                },
            )
            .next()
            .is_none()
    }

    pub fn scramble_3x3x3(&mut self) -> Alg {
        loop {
            let scramble_pattern = random_3x3x3_pattern();
            if !self.is_valid_scramble_pattern(&scramble_pattern) {
                continue;
            }
            return self.solve_3x3x3_pattern(&scramble_pattern);
        }
    }
}
