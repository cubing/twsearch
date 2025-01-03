use cubing::alg::{parse_alg, Alg};
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    _internal::search::{move_count::MoveCount, prune_table_trait::Depth},
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
        kpuzzle,
        filter_generator_moves,
        None,
        kpuzzle.default_pattern(),
    );

    let generator_moves = move_list_from_vec(vec!["U", "L", "F", "R", "D"]);
    let mut search =
        <FilteredSearch>::new(kpuzzle, generator_moves, None, kpuzzle.default_pattern());

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

        dbg!(&scramble_pattern);

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
            .search(&scramble_pattern, Some(1), Some(Depth(10)), None)
            .next()
        {
            return solution.invert();
        }
    }
}
