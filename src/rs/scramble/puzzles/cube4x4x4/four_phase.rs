use cubing::{
    alg::{Alg, Pause},
    kpuzzle::{KPattern, KPuzzle},
};

use url::Url;

use crate::{
    _internal::{IDFSearch, IndividualSearchOptions},
    scramble::{
        puzzles::{
            cube4x4x4::{
                phase2::{
                    pattern_to_phase2_pattern, remap_piece_for_phase1_or_phase2_search_pattern,
                    Phase2AdditionalSolutionCondition,
                },
                phase2_symmetry::Phase2SymmetryTables,
                random::random_4x4x4_pattern,
            },
            definitions::{
                cube4x4x4_packed_kpuzzle, cube4x4x4_phase1_target_pattern,
                cube4x4x4_phase2_target_pattern,
            },
        },
        scramble_search::{basic_idfs, idfs_with_target_pattern},
    },
};

use super::{
    super::super::scramble_search::generators_from_vec_str,
    super::definitions::cube4x4x4_with_wing_parity_packed_kpuzzle,
};

pub(crate) struct Scramble4x4x4FourPhase {
    packed_kpuzzle: KPuzzle,

    _filtering_idfs: IDFSearch,

    phase1_target_pattern: KPattern,
    phase1_idfs: IDFSearch,

    phase2_idfs: IDFSearch,
}

impl Default for Scramble4x4x4FourPhase {
    fn default() -> Self {
        let packed_kpuzzle = cube4x4x4_packed_kpuzzle();
        let phase2_packed_kpuzzle = cube4x4x4_with_wing_parity_packed_kpuzzle();

        let phase1_generators = generators_from_vec_str(vec![
            "Uw", "U", "Lw", "L", "Fw", "F", "Rw", "R", "Bw", "B", "Dw", "D",
        ]);
        // TODO: support normalizing orientation/ignoring orientation/24 targets, so that this checks for unoriented distance to solved.
        let filtering_idfs = basic_idfs(&packed_kpuzzle, phase1_generators.clone(), Some(32));

        let phase1_target_pattern = cube4x4x4_phase1_target_pattern();
        // dbg!(&phase1_target_pattern);
        let phase1_idfs = idfs_with_target_pattern(
            &packed_kpuzzle,
            phase1_generators.clone(),
            phase1_target_pattern.clone(),
            None,
        );

        let phase2_generators =
            generators_from_vec_str(vec!["Uw2", "U", "L", "F", "Rw", "R", "B", "Dw2", "D"]);
        let phase2_center_target_pattern = cube4x4x4_phase2_target_pattern();
        // dbg!(&phase2_center_target_pattern);
        let phase2_idfs = idfs_with_target_pattern(
            &phase2_packed_kpuzzle,
            phase2_generators.clone(),
            phase2_center_target_pattern.clone(),
            None,
        );

        Self {
            packed_kpuzzle,
            _filtering_idfs: filtering_idfs,
            phase1_target_pattern,
            phase1_idfs,
            phase2_idfs,
        }
    }
}

impl Scramble4x4x4FourPhase {
    pub(crate) fn solve_4x4x4_pattern(
        &mut self,
        main_search_pattern: &KPattern, // TODO: avoid assuming a superpattern.
    ) -> Alg {
        let mut x = Phase2SymmetryTables::new(self.packed_kpuzzle.clone());
        x.init_choose_tables();
        x.init_move_tables();
        x.init_prune_table();
        let phase1_alg = {
            let mut phase1_search_pattern = self.phase1_target_pattern.clone();
            for orbit_info in &self.packed_kpuzzle.data.orbit_iteration_info {
                for i in 0..orbit_info.num_pieces {
                    remap_piece_for_phase1_or_phase2_search_pattern(
                        orbit_info,
                        main_search_pattern,
                        &self.phase1_target_pattern,
                        &mut phase1_search_pattern,
                        i,
                    );
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

        dbg!(&phase1_alg.to_string());

        let mut phase2_alg = {
            // TODO: unify with phase 1 (almost identical code)
            let phase2_search_pattern = pattern_to_phase2_pattern(main_search_pattern);
            let phase2_search_pattern = phase2_search_pattern.apply_alg(&phase1_alg).unwrap();

            let phase2_search_full_pattern = main_search_pattern.apply_alg(&phase1_alg).unwrap();

            let additional_solution_condition = Phase2AdditionalSolutionCondition {
                kpuzzle: self.packed_kpuzzle.clone(),
                phase2_search_full_pattern,
                _debug_num_checked: 0,
                _debug_num_centers_rejected: 0,
                _debug_num_total_rejected: 0,
                _debug_num_basic_parity_rejected: 0,
                _debug_num_known_pair_orientation_rejected: 0,
                _debug_num_edge_parity_rejected: 0,
            };

            self.phase2_idfs
                .search_with_additional_check(
                    &phase2_search_pattern,
                    IndividualSearchOptions::default(),
                    Some(Box::new(additional_solution_condition)),
                )
                .next()
                .unwrap()
        };

        let mut nodes = phase1_alg.nodes;
        nodes.push(cubing::alg::AlgNode::PauseNode(Pause {}));
        nodes.append(&mut phase2_alg.nodes);
        nodes.push(cubing::alg::AlgNode::PauseNode(Pause {}));
        Alg { nodes }
    }

    // TODO: rely on the main search to find patterns at a low depth?
    pub fn is_valid_scramble_pattern(&mut self, _pattern: &KPattern) -> bool {
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
            let hardcoded_scramble_alg_for_testing ="F' R' B2 D L' B D L2 F L2 F2 B' L2 U2 F2 U2 F' R2 L2 D' L2 Fw2 Rw2 R F' Uw2 U2 Fw2 F Uw2 L U2 R2 D2 Uw U F R F' Rw' Fw B Uw' L' Fw2 F2".parse::<Alg>().unwrap();
            // let hardcoded_scramble_alg_for_testing = "2R u".parse::<Alg>().unwrap();
            // let hardcoded_scramble_alg_for_testing =
            //     "r U2 x r U2 r U2 r' U2 l U2 r' U2 r U2 r' U2 r'"
            //         .parse::<Alg>()
            //         .unwrap();
            // let hardcoded_scramble_alg_for_testing =
            //     "Uw2 Fw2 U' L2 F2 L' Uw2 Fw2 U D' L' U2 R' Fw D' Rw2 F' L2 Uw' //Fw L U' R2 Uw Fw"
            //         .parse::<Alg>()
            //         .unwrap();
            let scramble_pattern = random_4x4x4_pattern(Some(&hardcoded_scramble_alg_for_testing));

            if !self.is_valid_scramble_pattern(&scramble_pattern) {
                continue;
            }
            dbg!(hardcoded_scramble_alg_for_testing.to_string());
            let solution_alg = self.solve_4x4x4_pattern(&scramble_pattern);
            println!(
                "{}",
                twizzle_link(&hardcoded_scramble_alg_for_testing, &solution_alg)
            );
            return solution_alg;
        }
    }
}

// TODO: remove `url` crate when removing this.
pub fn twizzle_link(scramble: &Alg, solution: &Alg) -> String {
    let mut url = Url::parse("https://alpha.twizzle.net/edit/").unwrap();
    url.query_pairs_mut()
        .append_pair("setup-alg", &scramble.to_string());
    url.query_pairs_mut()
        .append_pair("alg", &solution.to_string());
    url.query_pairs_mut().append_pair("puzzle", "4x4x4");
    url.to_string()
}
