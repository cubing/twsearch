use std::sync::{Arc, OnceLock};

use cubing::{alg::Alg, parse_move, puzzles::cube2x2x2_kpuzzle};
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::_internal::{
    CustomGenerators, IDFSearch, IndividualSearchOptions, PackedKPattern, PackedKPuzzle,
    PackedKPuzzleOrbitInfo, SearchError, SearchLogger,
};

use super::Event;

pub fn random_scramble_for_event(event: Event) -> Result<Alg, SearchError> {
    match event {
        Event::Cube2x2x2 => return Ok(scramble_222()),
        Event::Pyraminx => {}
        _ => {}
    };

    Err(SearchError {
        description: "Could not generate scramble".to_owned(),
    })
}

static KPUZZLE_222_CELL: OnceLock<PackedKPuzzle> = OnceLock::new();
pub fn scramble_222() -> Alg {
    let packed_kpuzzle =
        KPUZZLE_222_CELL.get_or_init(|| PackedKPuzzle::try_from(cube2x2x2_kpuzzle()).unwrap());

    loop {
        let mut inverse_scramble_pattern = packed_kpuzzle.default_pattern();
        let orbit_info = &packed_kpuzzle.data.orbit_iteration_info[0];
        randomize_orbit_naive(
            &mut inverse_scramble_pattern,
            orbit_info,
            OrbitPermutationParityConstraints::IgnoreParity,
            OrbitOrientationParityConstraints::OrientationsMustSumToZero,
        );
        let generators = crate::_internal::Generators::Custom(CustomGenerators {
            moves: vec![
                parse_move!("U").unwrap(),
                parse_move!("L").unwrap(),
                parse_move!("F").unwrap(),
                parse_move!("R").unwrap(),
            ],
            algs: vec![],
        });
        let mut idfs = IDFSearch::try_new(
            packed_kpuzzle.clone(),
            packed_kpuzzle.default_pattern(),
            generators,
            Arc::new(SearchLogger {
                verbosity: crate::_internal::VerbosityLevel::Error,
            }),
            &crate::_internal::MetricEnum::Hand,
            true,
        )
        .unwrap();
        // Scramble filtering by rejection sampling : Too close to solved?
        // https://www.worldcubeassociation.org/regulations/#4b3b
        // https://github.com/thewca/tnoodle/blob/master/webscrambles/src/main/resources/wca/readme-scramble.md#scramble-length
        if idfs
            .search(
                &inverse_scramble_pattern,
                IndividualSearchOptions {
                    min_num_solutions: Some(1),
                    min_depth: Some(0),
                    max_depth: Some(3),
                },
            )
            .next()
            .is_some()
        {
            println!("Rejected!");
            continue;
        }
        return idfs
            .search(
                &inverse_scramble_pattern,
                IndividualSearchOptions {
                    min_num_solutions: Some(1),
                    min_depth: Some(11),
                    max_depth: None,
                },
            )
            .next()
            .unwrap();
    }
}

enum OrbitPermutationParityConstraints {
    IgnoreParity,
    #[allow(dead_code)] // TODO
    SingleOrbitEvenParity,
}
enum OrbitOrientationParityConstraints {
    #[allow(dead_code)] // TODO
    IgnoreParity,
    OrientationsMustSumToZero,
}

// Selects a random permutation (ignoring parity).
// Applies a random orientation to each piece (ensuring the total is 0).
fn randomize_orbit_naive(
    pattern: &mut PackedKPattern,
    orbit_info: &PackedKPuzzleOrbitInfo,
    permutation_constraints: OrbitPermutationParityConstraints,
    orientation_constraints: OrbitOrientationParityConstraints,
) {
    match permutation_constraints {
        OrbitPermutationParityConstraints::IgnoreParity => {}
        OrbitPermutationParityConstraints::SingleOrbitEvenParity => panic!("unimplemented"),
    }

    let mut rng = thread_rng();
    let mut corner_order: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7];
    corner_order.shuffle(&mut rng);
    let mut total_orientation = 0;
    for (i, p) in corner_order.into_iter().enumerate() {
        pattern
            .packed_orbit_data
            .set_packed_piece_or_permutation(orbit_info, i, p);
        let orientation = match orientation_constraints {
            OrbitOrientationParityConstraints::IgnoreParity => {
                let random_orientation = rng.gen_range(0..orbit_info.num_orientations);
                total_orientation = add_u8_mod(
                    total_orientation,
                    random_orientation,
                    orbit_info.num_orientations,
                );
                random_orientation
            }
            OrbitOrientationParityConstraints::OrientationsMustSumToZero => {
                subtract_u8_mod(0, total_orientation, orbit_info.num_orientations)
            }
        };

        pattern
            .packed_orbit_data
            .set_packed_orientation(orbit_info, i, orientation);
    }
}

// Adds without overflow.
fn add_u8_mod(v1: u8, v2: u8, modulus: u8) -> u8 {
    ((v1 as u32) + (v2 as u32)).rem_euclid(modulus as u32) as u8
}

fn subtract_u8_mod(v1: u8, v2: u8, modulus: u8) -> u8 {
    ((v1 as i32) - (v2 as i32)).rem_euclid(modulus as i32) as u8
}
