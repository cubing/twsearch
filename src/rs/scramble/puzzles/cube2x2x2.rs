use cubing::{alg::Alg, puzzles::cube2x2x2_kpuzzle};

use crate::{
    _internal::AlwaysValid,
    scramble::{
        puzzles::static_move_list::{add_random_suffixes_from, static_parsed_opt_list},
        randomize::PieceZeroConstraint,
        scramble_search::FilteredSearch,
    },
};

use super::{
    super::randomize::{
        randomize_orbit_naïve, OrbitOrientationConstraint, OrbitPermutationConstraint,
    },
    super::scramble_search::generators_from_vec_str,
};

pub fn scramble_2x2x2() -> Alg {
    let kpuzzle = cube2x2x2_kpuzzle();

    #[allow(non_snake_case)] // Move meanings are case sensitive.
    let mut filtered_search_L_B_D = FilteredSearch::<AlwaysValid>::new(
        kpuzzle,
        generators_from_vec_str(vec!["L", "B", "D"]),
        None,
        kpuzzle.default_pattern(),
    );

    #[allow(non_snake_case)] // Move meanings are case sensitive.
    let mut filtered_search_U_L_F_R = FilteredSearch::<AlwaysValid>::new(
        kpuzzle,
        generators_from_vec_str(vec!["U", "L", "F", "R"]),
        None,
        kpuzzle.default_pattern(),
    );

    loop {
        /* TODO: Since we don't yet have an API to solve to any orientation,
         * we perform the filtering search with a fixed orientation and then randomize the orientation for the returned scramble.
         */

        let mut scramble_pattern_fixed_corner = kpuzzle.default_pattern();
        randomize_orbit_naïve(
            &mut scramble_pattern_fixed_corner,
            0,
            "CORNERS",
            OrbitPermutationConstraint::AnyPermutation,
            OrbitOrientationConstraint::OrientationsMustSumToZero,
            PieceZeroConstraint::KeepSolved,
        );
        if let Some(filtered) = filtered_search_L_B_D.filter(&scramble_pattern_fixed_corner, 4) {
            dbg!(filtered.to_string());
            continue;
        }

        let s1 = static_parsed_opt_list(&["", "x", "x2", "x'", "z", "z'"]);
        let s2 = static_parsed_opt_list(&["", "y", "y2", "y'"]);
        let scramble_pattern_random_orientation = scramble_pattern_fixed_corner
            .apply_alg(&add_random_suffixes_from(Alg::default(), [s1, s2]))
            .unwrap();

        return filtered_search_U_L_F_R
            .generate_scramble(&scramble_pattern_random_orientation, Some(11));
    }
}
