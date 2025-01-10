use std::{env::var, str::FromStr, sync::Mutex};

use cubing::{alg::Alg, kpuzzle::KPattern};
use lazy_static::lazy_static;
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    _internal::search::{check_pattern::PatternValidityChecker, mask_pattern::apply_mask},
    scramble::{
        puzzles::square1::phase1::Phase1Checker,
        randomize::{
            randomize_orbit_naïve, ConstraintForFirstPiece, OrbitRandomizationConstraints,
        },
    },
};

use super::{
    super::definitions::{square1_square_square_shape_kpattern, square1_unbandaged_kpuzzle},
    solve::Square1Solver,
};

const DEBUG_STATIC_SQUARE_1_SCRAMBLE_SETUP_ALG: &str = "(-2, 3) / (-1, 2) / (-5, -2) / (3, -3) / (-4, 5) / (0, -2) / (0, -3) / (-2, -3) / (0, -4) / (2, 0) / (-3, 2) / (0, 2)";

impl Square1Solver {
    pub(crate) fn scramble_square1(&mut self) -> Alg {
        let use_static = match var("USE_STATIC_SQUARE1_SCRAMBLE_SETUP") {
            Ok(value) => value == "true",
            _ => false,
        };

        let pattern = if use_static {
            eprintln!("Observed USE_STATIC_SQUARE1_SCRAMBLE_SETUP");
            eprintln!(
                "Using static scramble setup: {}",
                DEBUG_STATIC_SQUARE_1_SCRAMBLE_SETUP_ALG
            );
            square1_unbandaged_kpuzzle()
                .default_pattern()
                .apply_alg(&Alg::from_str(DEBUG_STATIC_SQUARE_1_SCRAMBLE_SETUP_ALG).unwrap())
                .unwrap()
        } else {
            random_pattern()
        };

        self.solve_square1(&pattern).unwrap()
    }
}

pub(crate) fn scramble_square1() -> Alg {
    // TODO: figure out a better pattern for this?
    lazy_static! {
        static ref SQUARE1_SOLVER: Mutex<Square1Solver> = Mutex::new(Square1Solver::new());
    }
    SQUARE1_SOLVER.lock().unwrap().scramble_square1()
}

fn random_pattern() -> KPattern {
    let mut rng = thread_rng();

    loop {
        let mut scramble_pattern = square1_unbandaged_kpuzzle().default_pattern();

        let mut deep_wedges = vec![
            vec![0, 1],
            vec![2],
            vec![3, 4],
            vec![5],
            vec![6, 7],
            vec![8],
            vec![9, 10],
            vec![11],
            vec![12],
            vec![13, 14],
            vec![15],
            vec![16, 17],
            vec![18],
            vec![19, 20],
            vec![21],
            vec![22, 23],
        ];
        deep_wedges.shuffle(&mut rng);
        let wedge_orbit_info = &scramble_pattern.kpuzzle().clone().data.ordered_orbit_info[0];
        assert_eq!(wedge_orbit_info.name.0, "WEDGES");
        for (i, value) in deep_wedges.into_iter().flatten().enumerate() {
            unsafe {
                scramble_pattern
                    .packed_orbit_data_mut()
                    .set_raw_piece_or_permutation_value(wedge_orbit_info, i as u8, value);
            }
        }

        randomize_orbit_naïve(
            &mut scramble_pattern,
            1,
            "EQUATOR",
            OrbitRandomizationConstraints {
                first_piece: Some(ConstraintForFirstPiece::KeepSolved),
                ..Default::default()
            },
        );

        // TODO: do this check without masking.
        let phase1_start_pattern =
            apply_mask(&scramble_pattern, square1_square_square_shape_kpattern()).unwrap();

        if Phase1Checker::is_valid(&phase1_start_pattern) {
            return scramble_pattern;
        }
    }
}
