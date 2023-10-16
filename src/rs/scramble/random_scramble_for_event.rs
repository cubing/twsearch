use lazy_static::lazy_static;
use std::sync::Mutex;

use cubing::alg::{Alg, AlgNode, Move};
use rand::{thread_rng, Rng};

use crate::{
    _internal::{
        CustomGenerators, Generators, IDFSearch, IndividualSearchOptions, PackedKPattern,
        PackedKPuzzle, PuzzleError,
    },
    scramble::{
        randomize::{basic_parity, BasicParity},
        scramble_search::{basic_idfs, idfs_with_target_pattern},
    },
};

use super::{
    definitions::{
        cube2x2x2_packed_kpuzzle, cube3x3x3_centerless_packed_kpuzzle, cube3x3x3_g1_target_pattern,
        tetraminx_packed_kpuzzle,
    },
    randomize::{randomize_orbit_naive, OrbitOrientationConstraint, OrbitPermutationConstraint},
    scramble_search::filtered_search,
    Event,
};

pub fn random_scramble_for_event(event: Event) -> Result<Alg, PuzzleError> {
    let err = Err(PuzzleError {
        description: format!("Scrambles are not implement for this event yet: {}", event),
    });
    match event {
        Event::Cube3x3x3Speedsolving => {
            Ok(SCRAMBLE3X3X3_TWO_PHASE.lock().unwrap().scramble_3x3x3())
        }
        Event::Cube2x2x2Speedsolving => Ok(scramble_2x2x2()),
        Event::Cube4x4x4Speedsolving => err,
        Event::Cube5x5x5Speedsolving => err,
        Event::Cube6x6x6Speedsolving => err,
        Event::Cube7x7x7Speedsolving => err,
        Event::Cube3x3x3Blindfolded => err,
        Event::Cube3x3x3FewestMoves => err,
        Event::Cube3x3x3OneHanded => Ok(SCRAMBLE3X3X3_TWO_PHASE.lock().unwrap().scramble_3x3x3()),
        Event::ClockSpeedsolving => err,
        Event::MegaminxSpeedsolving => err,
        Event::PyraminxSpeedsolving => Ok(scramble_pyraminx()),
        Event::SkewbSpeedsolving => err,
        Event::Square1Speedsolving => err,
        Event::Cube4x4x4Blindfolded => err,
        Event::Cube5x5x5Blindfolded => err,
        Event::Cube3x3x3MultiBlind => err,
        Event::FTOSpeedsolving => err,
        Event::MasterTetraminxSpeedsolving => err,
        Event::KilominxSpeedsolving => err,
        Event::RediCubeSpeedsolving => err,
    }
}

fn move_list_from_vec(move_str_list: Vec<&str>) -> Vec<Move> {
    move_str_list
        .iter()
        .map(|move_str| move_str.parse::<Move>().unwrap())
        .collect()
}

fn generators_from_vec_str(move_str_list: Vec<&str>) -> Generators {
    crate::_internal::Generators::Custom(CustomGenerators {
        moves: move_list_from_vec(move_str_list),
        algs: vec![],
    })
}

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

// TODO: switch to `LazyLock` once that's stable: https://doc.rust-lang.org/nightly/std/cell/struct.LazyCell.html
lazy_static! {
    static ref SCRAMBLE3X3X3_TWO_PHASE: Mutex<Scramble3x3x3TwoPhase> =
        Mutex::new(Scramble3x3x3TwoPhase::default());
}

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

impl Scramble3x3x3TwoPhase {
    pub fn scramble_3x3x3(&mut self) -> Alg {
        loop {
            let scramble_pattern = {
                let mut scramble_pattern = self.packed_kpuzzle.default_pattern();
                let orbit_info = &self.packed_kpuzzle.data.orbit_iteration_info[0];
                assert_eq!(orbit_info.name.0, "EDGES");
                let edge_order = randomize_orbit_naive(
                    &mut scramble_pattern,
                    orbit_info,
                    OrbitPermutationConstraint::AnyPermutation,
                    OrbitOrientationConstraint::OrientationsMustSumToZero,
                );
                let each_orbit_parity = basic_parity(&edge_order);
                let orbit_info = &self.packed_kpuzzle.data.orbit_iteration_info[1];
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
            };

            {
                if self
                    .filtering_idfs
                    .search(
                        &scramble_pattern,
                        IndividualSearchOptions {
                            min_num_solutions: Some(1),
                            min_depth: Some(0),
                            max_depth: Some(2),
                        },
                    )
                    .next()
                    .is_some()
                {
                    continue;
                }
            }

            let phase1_alg = {
                let phase1_search_pattern = self.phase1_target_pattern.clone();
                for orbit_info in &self.packed_kpuzzle.data.orbit_iteration_info {
                    for i in 0..orbit_info.num_pieces {
                        let old_piece = scramble_pattern
                            .packed_orbit_data
                            .get_packed_piece_or_permutation(orbit_info, i);
                        let old_piece_mapped = self
                            .phase1_target_pattern
                            .packed_orbit_data
                            .get_packed_piece_or_permutation(orbit_info, old_piece as usize);
                        phase1_search_pattern
                            .packed_orbit_data
                            .set_packed_piece_or_permutation(orbit_info, i, old_piece_mapped);
                        let ori = scramble_pattern
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
                let phase2_search_pattern = scramble_pattern.apply_transformation(
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
            return Alg { nodes };
        }
    }
}

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
