use std::sync::Mutex;

use cubing::alg::Alg;
use lazy_static::lazy_static;

use crate::{
    _internal::{IDFSearch, IndividualSearchOptions, PackedKPattern, PackedKPuzzle},
    scramble::{
        puzzles::definitions::{cube4x4x4_packed_kpuzzle, cube4x4x4_phase1_target_pattern},
        scramble_search::{basic_idfs, idfs_with_target_pattern},
    },
};

use super::super::scramble_search::generators_from_vec_str;

pub struct Scramble4x4x4FourPhase {
    packed_kpuzzle: PackedKPuzzle,

    _filtering_idfs: IDFSearch,

    phase1_target_pattern: PackedKPattern,
    phase1_idfs: IDFSearch,
    // phase2_idfs: IDFSearch,
}

impl Default for Scramble4x4x4FourPhase {
    fn default() -> Self {
        let packed_kpuzzle = cube4x4x4_packed_kpuzzle();

        // TODO: support normalizing orientation/ignoring orientation/24 targets, so that this checks for unoriented distance to solved.
        let generators = generators_from_vec_str(vec![
            "Uw", "U", "Lw", "L", "Fw", "F", "Rw", "R", "Bw", "B", "Dw", "D",
        ]);
        let filtering_idfs = basic_idfs(&packed_kpuzzle, generators.clone(), Some(32));

        let phase1_target_pattern = cube4x4x4_phase1_target_pattern();
        dbg!(&phase1_target_pattern);
        let phase1_idfs = idfs_with_target_pattern(
            &packed_kpuzzle,
            generators.clone(),
            phase1_target_pattern.clone(),
            None,
        );

        Self {
            packed_kpuzzle,
            _filtering_idfs: filtering_idfs,
            phase1_target_pattern,
            phase1_idfs,
        }
    }
}

pub fn random_4x4x4_pattern() -> PackedKPattern {
    dbg!("random_4x4x4_pattern");
    let packed_kpuzzle = cube4x4x4_packed_kpuzzle();
    let mut scramble_pattern = packed_kpuzzle.default_pattern();

    // let transformation = packed_kpuzzle.transformation_from_alg(&"F' R' B2 D L' B D L2 F L2 F2 B' L2 U2 F2 U2 F' R2 L2 D' L2 Fw2 Rw2 R F' Uw2 U2 Fw2 F Uw2 L U2 R2 D2 Uw U F R F' Rw' Fw B Uw' L' Fw2 F2".parse::<Alg>().unwrap()).unwrap();
    let transformation = packed_kpuzzle
        .transformation_from_alg(&"d' r2' f' u D2' b B' F2'".parse::<Alg>().unwrap())
        .unwrap();
    scramble_pattern = scramble_pattern.apply_transformation(&transformation);

    // for orbit_info in &packed_kpuzzle.data.orbit_iteration_info {
    //     randomize_orbit_naive(
    //         &mut scramble_pattern,
    //         orbit_info,
    //         OrbitPermutationConstraint::AnyPermutation,
    //         OrbitOrientationConstraint::OrientationsMustSumToZero,
    //     );
    // }
    scramble_pattern
}

impl Scramble4x4x4FourPhase {
    pub(crate) fn solve_4x4x4_pattern(
        &mut self,
        pattern: &PackedKPattern, // TODO: avoid assuming a superpattern.
    ) -> Alg {
        dbg!("solve_4x4x4_pattern");
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
                    if orbit_info.name == "CORNERS".into() {
                        // TODO: handle this properly by taking into account orientation mod.
                        phase1_search_pattern
                            .packed_orbit_data
                            .set_packed_orientation(orbit_info, i, 3);
                    }
                    if orbit_info.name == "EDGES".into() {
                        // TODO: handle this properly by taking into account orientation mod.
                        phase1_search_pattern
                            .packed_orbit_data
                            .set_packed_orientation(orbit_info, i, 2);
                    }
                }
            }

            self.phase1_idfs
                .search(
                    &phase1_search_pattern,
                    IndividualSearchOptions {
                        min_num_solutions: Some(1),
                        min_depth: None,
                        max_depth: None,
                        disallowed_initial_quanta: None,
                        disallowed_final_quanta: None,
                    },
                )
                .next()
                .unwrap()
        };

        // let mut phase2_alg = {
        //     let phase2_search_pattern = pattern.apply_transformation(
        //         &self
        //             .packed_kpuzzle
        //             .transformation_from_alg(&phase1_alg)
        //             .unwrap(),
        //     );
        //     self.phase2_idfs
        //         .search(
        //             &phase2_search_pattern,
        //             IndividualSearchOptions {
        //                 min_num_solutions: Some(1),
        //                 min_depth: None,
        //                 max_depth: None,
        //                 disallowed_initial_quanta: None,
        //                 disallowed_final_quanta,
        //             },
        //         )
        //         .next()
        //         .unwrap()
        // };

        let nodes = phase1_alg.nodes;
        // nodes.append(&mut phase2_alg.nodes);
        Alg { nodes }
    }

    // TODO: rely on the main search to find patterns at a low depth?
    pub fn is_valid_scramble_pattern(&mut self, _pattern: &PackedKPattern) -> bool {
        eprintln!("WARNING: FILTERING DISABLED FOR TESTING"); // TODO
        true
        // self.filtering_idfs
        //     .search(
        //         pattern,
        //         IndividualSearchOptions {
        //             min_num_solutions: Some(1),
        //             min_depth: Some(0),
        //             max_depth: Some(2),
        //             disallowed_initial_quanta: None,
        //             disallowed_final_quanta: None,
        //         },
        //     )
        //     .next()
        //     .is_none()
    }

    pub(crate) fn scramble_4x4x4(&mut self) -> Alg {
        loop {
            let scramble_pattern = random_4x4x4_pattern();
            if !self.is_valid_scramble_pattern(&scramble_pattern) {
                continue;
            }
            return self.solve_4x4x4_pattern(&scramble_pattern);
        }
    }
}

// TODO: switch to `LazyLock` once that's stable: https://doc.rust-lang.org/nightly/std/cell/struct.LazyCell.html
lazy_static! {
    static ref SCRAMBLE4X4X4_FOUR_PHASE: Mutex<Scramble4x4x4FourPhase> =
        Mutex::new(Scramble4x4x4FourPhase::default());
}

pub fn scramble_4x4x4() -> Alg {
    SCRAMBLE4X4X4_FOUR_PHASE.lock().unwrap().scramble_4x4x4()
}
