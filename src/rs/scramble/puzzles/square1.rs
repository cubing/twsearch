use std::{
    io::{stdout, Write},
    time::{Duration, Instant},
};

use cubing::{
    alg::{parse_alg, parse_move, Alg},
    kpuzzle::KPattern,
};
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::{
    _internal::{PatternValidityChecker, FlatMoveIndex},
    scramble::{
        randomize::{basic_parity, BasicParity, PieceZeroConstraint},
        scramble_search::FilteredSearch,
    },
};

use super::{
    super::{
        randomize::{
            randomize_orbit_naïve, OrbitOrientationConstraint, OrbitPermutationConstraint,
        },
        scramble_search::generators_from_vec_str,
    },
    definitions::{square1_square_square_shape_kpattern, square1_unbandaged_kpuzzle},
    mask_pattern::mask,
    square1_phase_lookup_table::{build_phase_lookup_table, PhasePatternIndex},
};

#[derive(PartialEq, Eq)]
enum WedgeType {
    CornerLower,
    CornerUpper,
    Edge,
}

const NUM_WEDGES: u8 = 24;

const WEDGE_TYPE_LOOKUP: [WedgeType; NUM_WEDGES as usize] = [
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
];

struct Phase1Checker;

const SLOTS_THAT_ARE_AFTER_SLICES: [u8; 4] = [0, 6, 12, 18];

impl PatternValidityChecker for Phase1Checker {
    fn is_valid(pattern: &cubing::kpuzzle::KPattern) -> bool {
        let orbit_info = &pattern.kpuzzle().data.ordered_orbit_info[0];
        assert_eq!(orbit_info.name.0, "WEDGES");

        for slot in SLOTS_THAT_ARE_AFTER_SLICES {
            let value = unsafe {
                pattern
                    .packed_orbit_data()
                    .get_raw_piece_or_permutation_value(orbit_info, slot)
            };

            // TODO: consider removing this lookup. We know that the wedge values are only 0, 1, or
            // 2 during this phase.
            if WEDGE_TYPE_LOOKUP[value as usize] == WedgeType::CornerUpper {
                return false;
            }
        }

        true
    }
}

struct Phase2Checker;

impl PatternValidityChecker for Phase2Checker {
    fn is_valid(pattern: &cubing::kpuzzle::KPattern) -> bool {
        let orbit_info = &pattern.kpuzzle().data.ordered_orbit_info[0];
        assert_eq!(orbit_info.name.0, "WEDGES");

        for slot in [0, 1, 2, 12, 13, 14] {
            let value = unsafe {
                pattern
                    .packed_orbit_data()
                    .get_raw_piece_or_permutation_value(orbit_info, slot)
            };
            let wedge_type = &WEDGE_TYPE_LOOKUP[value as usize];

            if *wedge_type == WedgeType::CornerUpper && (slot == 0 || slot == 12) {
                // We can't slice.
                return false;
            }

            for slot_offset in [3, 6, 9] {
                let offset_value = unsafe {
                    pattern
                        .packed_orbit_data()
                        .get_raw_piece_or_permutation_value(orbit_info, slot + slot_offset)
                };
                let offset_wedge_type = &WEDGE_TYPE_LOOKUP[offset_value as usize];

                if wedge_type != offset_wedge_type {
                    return false;
                }
            }
        }

        true
    }
}

pub fn scramble_square1() -> Alg {
    let kpuzzle = square1_unbandaged_kpuzzle();
    let generators = generators_from_vec_str(vec!["U_SQ_", "D_SQ_", "_SLASH_"]); // TODO: cache

    let (phase_lookup_table, search_generators) = build_phase_lookup_table::<Phase1Checker>(
        kpuzzle.clone(),
        &generators,
        &square1_square_square_shape_kpattern().to_owned(),
    );
    // let idx = phase_lookup_table
    //     .index_to_lookup_pattern
    //     .at(PhasePatternIndex(0));
    #[allow(non_snake_case)]
    let U_SQ_ = phase_lookup_table.apply_move(PhasePatternIndex(0), FlatMoveIndex(22));
    dbg!(U_SQ_);
    dbg!(phase_lookup_table
        .index_to_lookup_pattern
        .at(U_SQ_.unwrap()));
    dbg!(phase_lookup_table.apply_move(U_SQ_.unwrap(), FlatMoveIndex(10)));
    #[allow(non_snake_case)]
    let U_SQ_SLICE = phase_lookup_table
        .index_to_lookup_pattern
        .at(phase_lookup_table
            .apply_move(U_SQ_.unwrap(), FlatMoveIndex(22))
            .unwrap());
    dbg!(U_SQ_SLICE);
    dbg!(
        U_SQ_,
        phase_lookup_table
            .move_application_table
            .at(U_SQ_.unwrap())
            .at(FlatMoveIndex(22))
    );
    dbg!(
        U_SQ_,
        phase_lookup_table
            .index_to_lookup_pattern
            .at(PhasePatternIndex(1))
    );

    dbg!(&search_generators.flat[10]);

    //     dbg!(wedge_parity(
    //         &kpuzzle
    //             .default_pattern()
    //             .apply_alg(&parse_alg!(
    //                 "(0, 5) / (3, 0) / (-5, -2) / (3, -3) / (5, -4) / (0, -3) / (-3, 0) / (-3, -3)
    // / U_SQ_2' D_SQ_ / U_SQ_'"
    //             ))
    //             .unwrap()
    //     ));
    //     exit(1);
    //
    loop {
        let mut scramble_pattern = kpuzzle.default_pattern();

        let mut rng = thread_rng();

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

        // <<< let scramble_pattern = scramble_pattern.apply_alg(&parse_alg!("U_SQ_3 D_SQ_ _SLASH_")).unwrap();
        // <<< let scramble_pattern = scramble_pattern.apply_alg(&parse_alg!("_SLASH_ U_SQ_3 D_SQ_ _SLASH_")).unwrap();

        // <<< let scramble_pattern = scramble_pattern.apply_alg(&parse_alg!("U_SQ_2' _SLASH_ U_SQ_5 D_SQ_2 _SLASH_ U_SQ_4 D_SQ_2' _SLASH_")).unwrap();
        // <<< let scramble_pattern = scramble_pattern.apply_alg(&parse_alg!("U_SQ_3 D_SQ_2 _SLASH_ D_SQ_")).unwrap();
        // <<< let scramble_pattern = scramble_pattern.apply_alg(&parse_alg!("(U_SQ_5' D_SQ_0) / (U_SQ_0 D_SQ_3) / (U_SQ_3 D_SQ_0) / (U_SQ_' D_SQ_4') / (U_SQ_4 D_SQ_2') / (U_SQ_5 D_SQ_4') / (U_SQ_2' D_SQ_0) / (U_SQ_0 D_SQ_3') / (U_SQ_' D_SQ_0) / (U_SQ_3 D_SQ_4') / (U_SQ_4 D_SQ_2') /")).unwrap();
        // <<< let scramble_pattern = scramble_pattern.apply_alg(&parse_alg!("(U_SQ_4 D_SQ_3) / (U_SQ_' D_SQ_') / (U_SQ_0 D_SQ_3') / (U_SQ_3' D_SQ_3') / (U_SQ_ D_SQ_2') / (U_SQ_3' D_SQ_4') / (U_SQ_3 D_SQ_0) / (U_SQ_4' D_SQ_5') / (U_SQ_3' D_SQ_0) / (U_SQ_4' D_SQ_0) / (U_SQ_0 D_SQ_2')")).unwrap();
        // <<< let scramble_pattern = scramble_pattern.apply_alg(&parse_alg!("(U_SQ_0 D_SQ_5) / (U_SQ_ D_SQ_5') / (U_SQ_0 D_SQ_3') / (U_SQ_3 D_SQ_0) / (U_SQ_4' D_SQ_') / (U_SQ_3' D_SQ_3') / (U_SQ_0 D_SQ_5') / (U_SQ_3' D_SQ_3') / (U_SQ_4' D_SQ_0) / (U_SQ_0 D_SQ_5') / (U_SQ_4 D_SQ_3') / (U_SQ_0 D_SQ_2') /")).unwrap();

        let scramble_pattern = kpuzzle
            .default_pattern()
            .apply_alg(&parse_alg!(
                // this is square-square
                // "(0, 5) / (3, 0) / (-5, -2) / (3, -3) / (5, -4) / (0, -3) / (-3, 0) / (-3, -3)"

                // this is not square-square
                "(0, -1) / (0, -3) / (0, -3) / (-2, -2) / (-3, -4) / (-3, 0) / (0, -3) / (-5, 0) / (5, 0) / (-2, -3) / (0, -4) / (-5, 0) / (0, -2) /"
            ))
            .unwrap();

        let phase1_start_pattern =
            mask(&scramble_pattern, square1_square_square_shape_kpattern()).unwrap();

        if !Phase1Checker::is_valid(&phase1_start_pattern) {
            println!("discarding invalid scramble"); //<<<
            continue;
        }

        dbg!(&phase1_start_pattern.to_data());

        // dbg!(
        //     &phase1_start_pattern
        //         .apply_alg(&parse_alg!("(3, 3) / (-1, 1)"))
        //         .unwrap()
        //         == square1_square_square_shape_kpattern()
        // );
        // <<<
        dbg!(&square1_square_square_shape_kpattern().to_data());

        // <<< if let Some(solution) = simple_filtered_search(&phase1_start_pattern, generators, 11, None) {

        // let direct = simple_filtered_search::<AlwaysValid>(
        //     &phase1_start_pattern,
        //     generators.clone(),
        //     0,
        //     None,
        // )
        // .unwrap();
        // println!("{}", direct);

        let mut phase1_filtered_search = FilteredSearch::<Phase1Checker>::new(
            kpuzzle,
            generators.clone(),
            None,
            square1_square_square_shape_kpattern().clone(),
        );

        // let generators2 = generators_from_vec_str(vec!["US", "DS", "UUU", "DDD"]); // TODO: cache
        let mut phase2_filtered_search = FilteredSearch::<Phase2Checker>::new(
            kpuzzle,
            generators,
            None, // TODO
            kpuzzle.default_pattern(),
        );
        // phase2_filtered_search
        //     .idfs
        //     .prune_table
        //     .extend_for_search_depth(11, 6140878);

        println!("PHASE1ING");

        let start_time = Instant::now();
        let mut odd_parity_counter = 0;
        let mut num_phase2_starts = 0;
        let mut phase1_start_time = Instant::now();
        let mut phase1_cumulative_time = Duration::default();
        let mut phase2_cumulative_time = Duration::default();
        let mut parity_check_cumulative_time = Duration::default();
        'phase1_loop: for mut phase1_solution in phase1_filtered_search.search(
            &phase1_start_pattern,
            Some(10_000_000), // see "le tired' below
            None,
            Some(18), // Max phase 1 length
        ) {
            phase1_cumulative_time += Instant::now() - phase1_start_time;

            let phase2_start_pattern_for_parity =
                scramble_pattern.apply_alg(&phase1_solution).unwrap();

            // println!("--------\n{}", phase1_solution);
            let parity_check_start_time = Instant::now();
            let par = wedge_parity(&phase2_start_pattern_for_parity);
            // println!("{:?}", par);
            // println!("{:?}", wedge_parity(&kpuzzle.default_pattern()
            //     .apply_alg(&parse_alg!("(0, 5) / (3, 0) / (-5, -2) / (3, -3) / (5, -4) / (0, -3) / (-3, 0) / (-3, -3)")).unwrap()
            //     .apply_alg(&phase1_solution).unwrap())
            // );
            // println!("{:?}", par == BasicParity::Odd);
            parity_check_cumulative_time += Instant::now() - parity_check_start_time;
            if par == BasicParity::Odd {
                odd_parity_counter += 1;
                phase1_start_time = Instant::now();
                continue;
            }

            while let Some(cubing::alg::AlgNode::MoveNode(r#move)) = phase1_solution.nodes.last() {
                if r#move == &parse_move!("_SLASH_'")
                // TODO: redundant parsing
                {
                    break;
                }
                // Discard equivalent phase 1 solutions (reduces redundant phase 2 searches by a factor of 16).
                if r#move.amount > 2 || r#move.amount < 0 {
                    phase1_start_time = Instant::now();
                    continue 'phase1_loop;
                }
                phase1_solution.nodes.pop();
            }

            let phase2_start_pattern = scramble_pattern.apply_alg(&phase1_solution).unwrap();

            num_phase2_starts += 1;
            // println!("\n{}", phase1_solution);
            // println!("\nSearching for a phase2 solution");
            print!(".");
            let _ = stdout().flush();
            let phase2_solution = phase2_filtered_search
                .search(
                    &phase2_start_pattern,
                    Some(1),
                    None,
                    Some(17), // <<< needs explanation
                )
                .next();

            let phase2_start_time = Instant::now();
            if let Some(mut phase2_solution) = phase2_solution {
                let mut nodes = phase1_solution.nodes;
                nodes.append(&mut phase2_solution.nodes);
                dbg!(&scramble_pattern.to_data());

                // <<< return Alg { nodes }.invert()
                return Alg { nodes }; // because slash' is not a valid move we can print
            }
            phase2_cumulative_time += Instant::now() - phase2_start_time;

            let cumulative_time = Instant::now() - start_time;
            if num_phase2_starts % 100 == 0 {
                println!(
                    "\n{} phase 2 starts so far, {:?} in phase 1, {:?} in phase 2, {:?} in phase transition, {:?} in parity check, {:?} odd parities\n",
                    num_phase2_starts,
                    phase1_cumulative_time,
                    phase2_cumulative_time,
                    cumulative_time - phase1_cumulative_time - phase2_cumulative_time,
                    parity_check_cumulative_time,
                    odd_parity_counter,
                )
            }

            phase1_start_time = Instant::now();
        }

        panic!("I am le tired, I give up");

        // <<< return Alg {
        // <<<     nodes: phase1_solution.nodes,
        // <<< }; //<<<

        // <<< while let Some(cubing::alg::AlgNode::MoveNode(r#move)) = phase1_solution.nodes.last() {
        // <<<     if r#move == &parse_move!("_SLASH_'")
        // <<<     // TODO: redundant parsing
        // <<<     {
        // <<<         break;
        // <<<     }
        // <<<     phase1_solution.nodes.pop();
        // <<< }
        // <<< dbg!(&parse_move!("/")); //<<<

        // dbg!(basic_parity(phase2_start_pattern.));

        // if let Some(solution) = filtered_search.generate_scramble(&phase1_start_pattern, 0) {
        //     //<<<
        //     return solution.invert();
        // }
    }
}

pub fn wedge_parity(pattern: &KPattern) -> BasicParity {
    let wedge_orbit_info = &pattern.kpuzzle().data.ordered_orbit_info[0];
    assert_eq!(wedge_orbit_info.name.0, "WEDGES");

    let mut bandaged_wedges = Vec::<u8>::default();
    for slot in 0..NUM_WEDGES {
        let value = unsafe {
            pattern
                .packed_orbit_data()
                .get_raw_piece_or_permutation_value(wedge_orbit_info, slot)
        };
        if WEDGE_TYPE_LOOKUP[value as usize] != WedgeType::CornerUpper {
            bandaged_wedges.push(value);
        }
    }
    basic_parity(&bandaged_wedges)
}
