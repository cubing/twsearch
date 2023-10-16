use std::sync::OnceLock;

use cubing::{
    alg::{Alg, Move},
    puzzles::cube2x2x2_kpuzzle,
};

use crate::_internal::{CustomGenerators, Generators, PackedKPuzzle, SearchError};

use super::{
    randomize::{
        randomize_orbit_naive, OrbitOrientationParityConstraints, OrbitPermutationParityConstraints,
    },
    scramble_search::scramble_search,
    Event,
};

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

fn generators_from_vec_str(move_str_list: Vec<&str>) -> Generators {
    crate::_internal::Generators::Custom(CustomGenerators {
        moves: move_str_list
            .iter()
            .map(|move_str| move_str.parse::<Move>().unwrap())
            .collect(),
        algs: vec![],
    })
}

static KPUZZLE_222_CELL: OnceLock<PackedKPuzzle> = OnceLock::new();
pub fn scramble_222() -> Alg {
    let packed_kpuzzle =
        KPUZZLE_222_CELL.get_or_init(|| PackedKPuzzle::try_from(cube2x2x2_kpuzzle()).unwrap());

    loop {
        let mut scramble_pattern = packed_kpuzzle.default_pattern();
        let orbit_info = &packed_kpuzzle.data.orbit_iteration_info[0];
        randomize_orbit_naive(
            &mut scramble_pattern,
            orbit_info,
            OrbitPermutationParityConstraints::IgnoreParity,
            OrbitOrientationParityConstraints::OrientationsMustSumToZero,
        );
        let generators = generators_from_vec_str(vec!["U", "L", "F", "R"]);
        if let Some(scramble) = scramble_search(&scramble_pattern, generators, 4, 11) {
            return scramble;
        }
    }
}
