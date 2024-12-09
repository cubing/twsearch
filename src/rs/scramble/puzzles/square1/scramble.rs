use cubing::{alg::Alg, kpuzzle::KPattern};
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    _internal::search::check_pattern::PatternValidityChecker,
    scramble::{
        puzzles::{mask_pattern::apply_mask, square1::phase1::Phase1Checker},
        randomize::{
            randomize_orbit_naïve, OrbitOrientationConstraint, OrbitPermutationConstraint,
            PieceZeroConstraint,
        },
    },
};

use super::{
    super::definitions::{square1_square_square_shape_kpattern, square1_unbandaged_kpuzzle},
    solve::solve_square1,
};

pub(crate) fn scramble_square1() -> Alg {
    solve_square1(&random_pattern())
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
            OrbitPermutationConstraint::AnyPermutation,
            OrbitOrientationConstraint::AnySum,
            PieceZeroConstraint::KeepSolved,
        );

        // TODO: do this check without masking.
        let phase1_start_pattern =
            apply_mask(&scramble_pattern, square1_square_square_shape_kpattern()).unwrap();

        if Phase1Checker::is_valid(&phase1_start_pattern) {
            dbg!(&scramble_pattern);
            return scramble_pattern;
        }

        eprintln!("discarding invalid scramble"); //<<<}
    }
}
