use std::collections::HashSet;

use cubing::{
    alg::{Alg, QuantumMove},
    puzzles::cube2x2x2_kpuzzle,
};

use crate::{
    _internal::{
        canonical_fsm::canonical_fsm::CanonicalFSMConstructionOptions,
        search::{
            idf_search::idf_search::{IDFSearch, IDFSearchConstructionOptions},
            move_count::MoveCount,
        },
    },
    scramble::{
        collapse::collapse_adjacent_moves,
        puzzles::static_move_list::{add_random_suffixes_from, static_parsed_opt_list},
        randomize::{ConstraintForFirstPiece, OrbitRandomizationConstraints},
        scramble_search::{move_list_from_vec, FilteredSearch},
    },
};

use super::super::randomize::{randomize_orbit_naïve, OrbitOrientationConstraint};

pub fn scramble_2x2x2() -> Alg {
    let kpuzzle = cube2x2x2_kpuzzle();

    #[allow(non_snake_case)] // Move meanings are case sensitive.
    let mut filtered_search_L_B_D = <FilteredSearch>::new(
        IDFSearch::try_new(
            kpuzzle.clone(),
            move_list_from_vec(vec!["L", "B", "D"]),
            kpuzzle.default_pattern(),
            Default::default(),
        )
        .unwrap(),
    );

    #[allow(non_snake_case)] // Move meanings are case sensitive.
    let mut filtered_search_U_L_F_R = <FilteredSearch>::new(
        IDFSearch::try_new(
            kpuzzle.clone(),
            move_list_from_vec(vec!["U", "L", "F", "R"]),
            kpuzzle.default_pattern(),
            IDFSearchConstructionOptions {
                canonical_fsm_construction_options: CanonicalFSMConstructionOptions {
                    forbid_transitions_by_quantums_either_direction: HashSet::from([(
                        QuantumMove::new("L", None),
                        QuantumMove::new("R", None),
                    )]),
                },
                ..Default::default()
            },
        )
        .unwrap(),
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
            OrbitRandomizationConstraints {
                orientation: Some(OrbitOrientationConstraint::SumToZero),
                first_piece: Some(ConstraintForFirstPiece::KeepSolved),
                ..Default::default()
            },
        );
        if filtered_search_L_B_D
            .filter(&scramble_pattern_fixed_corner, MoveCount(4))
            .is_some()
        {
            continue;
        }

        let s1 = static_parsed_opt_list(&["", "x", "x2", "x'", "z", "z'"]);
        let s2 = static_parsed_opt_list(&["", "y", "y2", "y'"]);
        let scramble_pattern_random_orientation = scramble_pattern_fixed_corner
            .apply_alg(&add_random_suffixes_from(Alg::default(), [s1, s2]))
            .unwrap();

        return collapse_adjacent_moves(
            filtered_search_U_L_F_R
                .generate_scramble(&scramble_pattern_random_orientation, Some(MoveCount(11))),
            4,
            -1,
        );
    }
}
