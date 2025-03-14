use std::collections::HashSet;

use cubing::alg::{parse_alg, Alg, QuantumMove};
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    _internal::{
        canonical_fsm::canonical_fsm::CanonicalFSMConstructionOptions,
        search::{
            iterative_deepening::iterative_deepening_search::{
                IndividualSearchOptions, IterativeDeepeningSearch,
                IterativeDeepeningSearchConstructionOptions,
            },
            move_count::MoveCount,
            prune_table_trait::Depth,
        },
    },
    scramble::{
        randomize::OrbitRandomizationConstraints,
        scramble_search::{move_list_from_vec, FilteredSearch},
    },
};

use super::{
    super::randomize::{
        randomize_orbit_naïve, OrbitOrientationConstraint, OrbitPermutationConstraint,
    },
    definitions::baby_fto_kpuzzle,
};

pub fn scramble_baby_fto() -> Alg {
    let kpuzzle = baby_fto_kpuzzle();
    let filter_generator_moves = move_list_from_vec(vec!["U", "L", "F", "R"]);
    let mut filtered_search = <FilteredSearch>::new(
        IterativeDeepeningSearch::try_new(
            kpuzzle.clone(),
            filter_generator_moves,
            vec![kpuzzle.default_pattern()],
            Default::default(),
        )
        .unwrap(),
    );

    let generator_moves = move_list_from_vec(vec!["U", "L", "F", "R", "BR"]);
    let mut search = <FilteredSearch>::new(
        IterativeDeepeningSearch::try_new(
            kpuzzle.clone(),
            generator_moves,
            vec![kpuzzle.default_pattern()],
            IterativeDeepeningSearchConstructionOptions {
                canonical_fsm_construction_options: CanonicalFSMConstructionOptions {
                    forbid_transitions_by_quantums_either_direction: HashSet::from([(
                        QuantumMove::new("L", None),
                        QuantumMove::new("BR", None),
                    )]),
                },
                ..Default::default()
            },
        )
        .unwrap(),
    );

    loop {
        let mut scramble_pattern = kpuzzle.default_pattern();

        randomize_orbit_naïve(
            &mut scramble_pattern,
            1,
            "C4RNER",
            OrbitRandomizationConstraints {
                permutation: Some(OrbitPermutationConstraint::EvenParity),
                orientation: Some(OrbitOrientationConstraint::EvenOddHackSumToZero(vec![
                    1, 3, 4,
                ])),
                subset: Some((0..5).collect()), // TODO: reorder to allow the first piece to be fixed
                ..Default::default()
            },
        );

        for subset in [vec![0, 1, 2, 7], vec![3, 4, 5, 6]] {
            randomize_orbit_naïve(
                &mut scramble_pattern,
                0,
                "CENTERS",
                OrbitRandomizationConstraints {
                    permutation: Some(OrbitPermutationConstraint::EvenParity),
                    orientation: Some(OrbitOrientationConstraint::IgnoreAllOrientations),
                    subset: Some(subset),
                    ..Default::default()
                },
            );
        }

        if let Some(alg) = filtered_search.filter(&scramble_pattern, MoveCount(5)) {
            eprintln!("Skipping due to short solution: {}", alg);
            continue;
        }

        let mut rng = thread_rng();
        // TODO: Have a consistent way to handle orientation (de)normalization.
        let scramble_pattern = scramble_pattern
            .apply_alg(
                [parse_alg!(""), parse_alg!("Rv Uv")]
                    .choose(&mut rng)
                    .unwrap(),
            )
            .unwrap();
        let scramble_pattern = scramble_pattern
            .apply_alg(
                [parse_alg!(""), parse_alg!("Rv'")]
                    .choose(&mut rng)
                    .unwrap(),
            )
            .unwrap();
        let scramble_pattern = scramble_pattern
            .apply_alg(
                [parse_alg!(""), parse_alg!("Uv"), parse_alg!("Uv'")]
                    .choose(&mut rng)
                    .unwrap(),
            )
            .unwrap();
        if let Some(solution) = search
            .iterative_deepening_search
            .search(
                &scramble_pattern,
                IndividualSearchOptions {
                    min_num_solutions: Some(1),
                    min_depth: Some(Depth(10)),
                    ..Default::default()
                },
            )
            .next()
        {
            return solution.invert();
        }
    }
}
