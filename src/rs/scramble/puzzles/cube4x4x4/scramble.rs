use std::{env::var, str::FromStr, sync::Mutex};

use cubing::{alg::Alg, kpuzzle::KPattern};
use lazy_static::lazy_static;

use crate::scramble::{
    collapse::collapse_adjacent_moves,
    puzzles::definitions::cube4x4x4_kpuzzle,
    randomize::{
        randomize_orbit_naïve, OrbitOrientationConstraint, OrbitRandomizationConstraints
    },
};

use super::solve::Cube4x4x4Solver;

#[allow(non_upper_case_globals)]
const DEBUG_USE_STATIC_4x4x4_SCRAMBLE_SETUP_ALG: &str = "B' D' R' D' R2 D' F' D2 L' F2 R' U2 F2 R L' D2 B2 L U F Fw2 Rw2 B' D2 F' Uw2 Rw2 U' Fw2 B' D' U2 R' D B Rw L' U Uw B2 F' Rw Uw' D' U2";

impl Cube4x4x4Solver {
    pub(crate) fn scramble_4x4x4(&mut self) -> Alg {
        let use_static = match var("USE_STATIC_4x4x4_SCRAMBLE_SETUP") {
            Ok(value) => value == "true",
            _ => false,
        };

        let pattern = if use_static {
            eprintln!("Observed USE_STATIC_4x4x4_SCRAMBLE_SETUP");
            eprintln!(
                "Using static scramble setup: {}",
                DEBUG_USE_STATIC_4x4x4_SCRAMBLE_SETUP_ALG
            );
            cube4x4x4_kpuzzle()
                .default_pattern()
                .apply_alg(&Alg::from_str(DEBUG_USE_STATIC_4x4x4_SCRAMBLE_SETUP_ALG).unwrap())
                .unwrap()
        } else {
            random_pattern_without_depth_filtering() // TODO
                                                     // self.random_depth_filtered_pattern()
        };

        dbg!(&pattern);

        collapse_adjacent_moves(self.solve_4x4x4(&pattern).unwrap().invert(), 4, -1)
    }

    // fn random_depth_filtered_pattern(&mut self) -> KPattern {
    //     loop {
    //         let pattern = random_pattern_without_depth_filtering();
    //         if self
    //             .depth_filtering_search
    //             .filter(&pattern, SQUARE_1_SCRAMBLE_MIN_OPTIMAL_MOVE_COUNT)
    //             .is_none()
    //         {
    //             return pattern;
    //         }
    //     }
    // }
}

pub(crate) fn scramble_4x4x4() -> Alg {
    // TODO: figure out a better pattern for this?
    lazy_static! {
        static ref SQUARE1_SOLVER: Mutex<Cube4x4x4Solver> = Mutex::new(Cube4x4x4Solver::new());
    }
    SQUARE1_SOLVER.lock().unwrap().scramble_4x4x4()
}

fn random_pattern_without_depth_filtering() -> KPattern {
    let mut scramble_pattern = cube4x4x4_kpuzzle().default_pattern();

    randomize_orbit_naïve(
        &mut scramble_pattern,
        0,
        "CORNERS",
        OrbitRandomizationConstraints {
            orientation: Some(OrbitOrientationConstraint::SumToZero),
            ..Default::default()
        },
    );

    randomize_orbit_naïve(&mut scramble_pattern, 1, "WINGS", Default::default());
    randomize_orbit_naïve(&mut scramble_pattern, 2, "CENTERS", Default::default());

    scramble_pattern
}
