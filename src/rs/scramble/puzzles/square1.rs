use std::{
    env,
    fmt::Debug,
    io::{stdout, Write},
    time::{Duration, Instant},
};

use cubing::{
    alg::{parse_alg, parse_move, Alg},
    kpuzzle::{KPattern, KPuzzle},
};
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    _internal::{
        Depth, HashPruneTable, IDFSearch, IndividualSearchOptions, PatternValidityChecker,
        SearchLogger, SearchOptimizations,
    },
    scramble::{
        puzzles::mask_pattern::mask,
        randomize::{basic_parity, BasicParity},
        scramble_search::{move_list_from_vec, FilteredSearch},
    },
};

use super::{
    coordinate_lookup_table::{PhaseCoordinatePuzzle, SemanticCoordinate},
    definitions::{square1_square_square_shape_kpattern, square1_unbandaged_kpuzzle},
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

pub struct Phase1Checker;

const SLOTS_THAT_ARE_AFTER_SLICES: [u8; 4] = [0, 6, 12, 18];

impl PatternValidityChecker<KPuzzle> for Phase1Checker {
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

impl PatternValidityChecker<KPuzzle> for Phase2Checker {
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

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Square1Phase1CompoundSemanticCoordinate {
    masked_pattern: KPattern,
    parity: BasicParity,
}

impl Debug for Square1Phase1CompoundSemanticCoordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Phase1Coordinates")
            .field("masked_pattern", &self.masked_pattern.to_data())
            .field("parity", &self.parity)
            .finish()
    }
}

impl SemanticCoordinate<KPuzzle> for Square1Phase1CompoundSemanticCoordinate {
    fn try_new(_kpuzzle: &KPuzzle, full_pattern: &KPattern) -> Option<Self> {
        let phase_mask = &square1_square_square_shape_kpattern(); // TODO: Store this with the coordinate lookup?
        let Ok(masked_pattern) = mask(full_pattern, phase_mask) else {
            panic!("Mask application failed");
        };

        // TODO: this isn't a full validity check for scramble positions.
        if !Phase1Checker::is_valid(&masked_pattern) {
            return None;
        }

        let parity = wedge_parity(full_pattern);
        Some(Self {
            masked_pattern,
            parity,
        })
    }
}

type Square1Phase1Puzzle = PhaseCoordinatePuzzle<Square1Phase1CompoundSemanticCoordinate>;

struct Square1Phase2Optimizations {}

impl SearchOptimizations<KPuzzle> for Square1Phase2Optimizations {
    type PatternValidityChecker = Phase2Checker;

    type PruneTable = HashPruneTable<KPuzzle, Phase2Checker>;
}

pub fn scramble_square1() -> Alg {
    match env::var("EXPERIMENTAL_SQUARE1").as_deref() {
        Ok("true") => {
            eprintln!("⚠️👷 Square-1 scrambling is still under construction! The code may hang or produce invalid results at this time. 👷⚠️");
        }
        _ => {
            eprintln!("⚠️👷 Square-1 scrambling is still under construction! Returning a bogus alg. Set `EXPERIMENTAL_SQUARE1=true` in the environment to run the scramble code. 👷⚠️");
            return parse_alg!("_SLASH_");
        }
    }

    let kpuzzle = square1_unbandaged_kpuzzle();
    let generator_moves = vec![parse_move!("U_SQ_"), parse_move!("D_SQ_"), parse_move!("/")];

    let (square1_phase1_lookup_table, _search_generators) = Square1Phase1Puzzle::new(
        kpuzzle.clone(),
        kpuzzle.default_pattern(),
        generator_moves.clone(),
    );

    let scramble_pattern = random_pattern();

    let phase1_start_pattern =
        square1_phase1_lookup_table.full_pattern_to_coordinates(&random_pattern());
    let phase1_target_pattern =
        square1_phase1_lookup_table.full_pattern_to_coordinates(&kpuzzle.default_pattern());
    let mut generic_idfs = IDFSearch::<Square1Phase1Puzzle>::try_new(
        square1_phase1_lookup_table,
        phase1_target_pattern,
        generator_moves,
        SearchLogger {
            verbosity: crate::_internal::options::VerbosityLevel::Info,
        }
        .into(),
        &crate::_internal::options::MetricEnum::Hand,
        false,
        None,
    )
    .unwrap();

    // let start_time = Instant::now();
    // let mut last_solution: Alg = parse_alg!("/");
    let num_solutions = 1_000_000;
    let phase1_search = generic_idfs.search(
        &phase1_start_pattern,
        IndividualSearchOptions {
            min_num_solutions: Some(num_solutions),
            ..Default::default()
        },
    );
    // for (i, solution) in phase1_search.enumerate() {
    //     if (i + 1) % (num_solutions / 10) == 0 {
    //         println!(
    //             "// Phase 1 solution #{}
    // {}
    // ",
    //             i + 1,
    //             solution
    //         )
    //     }
    //     last_solution = solution;
    // }
    // println!(
    //     "Elapsed time to find {} solutions for phase 1 test: {:?}
    // ",
    //     num_solutions,
    //     Instant::now() - start_time
    // );

    // todo!();

    // let generators2 = generators_from_vec_str(vec!["US", "DS", "UUU", "DDD"]); // TODO: cache
    let generator_moves = move_list_from_vec(vec!["U_SQ_", "D_SQ_", "_SLASH_"]); // TODO: cache
    let mut phase2_filtered_search = FilteredSearch::<KPuzzle, Square1Phase2Optimizations>::new(
        kpuzzle,
        generator_moves,
        None, // TODO
        kpuzzle.default_pattern(),
    );

    println!("PHASE1ING");

    let start_time = Instant::now();
    let mut num_phase2_starts = 0;
    let mut phase1_start_time = Instant::now();
    let mut phase1_cumulative_time = Duration::default();
    let mut phase2_cumulative_time = Duration::default();
    #[allow(non_snake_case)]
    let _SLASH_ = parse_move!("/");
    'phase1_loop: for mut phase1_solution in phase1_search {
        phase1_cumulative_time += Instant::now() - phase1_start_time;

        // TODO: Push the candidate check into a trait for `IDFSearch`.
        while let Some(cubing::alg::AlgNode::MoveNode(r#move)) = phase1_solution.nodes.last() {
            if r#move == &_SLASH_
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
        let phase2_start_time = Instant::now();
        let phase2_solution = phase2_filtered_search
            .search(
                &phase2_start_pattern,
                Some(1),
                None,
                Some(Depth(17)), // <<< needs explanation
            )
            .next();

        if let Some(mut phase2_solution) = phase2_solution {
            let mut nodes = phase1_solution.nodes;
            nodes.append(&mut phase2_solution.nodes);
            dbg!(&phase1_start_pattern);

            // <<< return Alg { nodes }.invert()
            return Alg { nodes }; // because slash' is not a valid move we can print
        }
        phase2_cumulative_time += Instant::now() - phase2_start_time;

        let cumulative_time = Instant::now() - start_time;
        if num_phase2_starts % 100 == 0 {
            println!(
                    "\n{} phase 2 starts so far, {:?} in phase 1, {:?} in phase 2, {:?} in phase transition\n",
                    num_phase2_starts,
                    phase1_cumulative_time,
                    phase2_cumulative_time,
                    cumulative_time - phase1_cumulative_time - phase2_cumulative_time,
                )
        }

        phase1_start_time = Instant::now();
    }

    panic!("at the (lack of) disco(very)")
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

fn random_pattern() -> KPattern {
    let mut rng = thread_rng();

    square1_unbandaged_kpuzzle()
        .default_pattern()
        .apply_alg(&parse_alg!(
            "(0, -1) / (4, -2) / (5, -1) / (4, -5) / (0, -3) / (-1, -3) / (3, 0) / (-3, 0) / (4, 0) / (4, 0) /"
        ))
        .unwrap()

    // loop {
    //     let mut scramble_pattern = square1_unbandaged_kpuzzle().default_pattern();

    //     let mut deep_wedges = vec![
    //         vec![0, 1],
    //         vec![2],
    //         vec![3, 4],
    //         vec![5],
    //         vec![6, 7],
    //         vec![8],
    //         vec![9, 10],
    //         vec![11],
    //         vec![12],
    //         vec![13, 14],
    //         vec![15],
    //         vec![16, 17],
    //         vec![18],
    //         vec![19, 20],
    //         vec![21],
    //         vec![22, 23],
    //     ];
    //     deep_wedges.shuffle(&mut rng);
    //     let wedge_orbit_info = &scramble_pattern.kpuzzle().clone().data.ordered_orbit_info[0];
    //     assert_eq!(wedge_orbit_info.name.0, "WEDGES");
    //     for (i, value) in deep_wedges.into_iter().flatten().enumerate() {
    //         unsafe {
    //             scramble_pattern
    //                 .packed_orbit_data_mut()
    //                 .set_raw_piece_or_permutation_value(wedge_orbit_info, i as u8, value);
    //         }
    //     }

    //     randomize_orbit_naïve(
    //         &mut scramble_pattern,
    //         1,
    //         "EQUATOR",
    //         OrbitPermutationConstraint::AnyPermutation,
    //         OrbitOrientationConstraint::AnySum,
    //         PieceZeroConstraint::KeepSolved,
    //     );

    //     // TODO: do this check without masking.
    //     let phase1_start_pattern =
    //         mask(&scramble_pattern, square1_square_square_shape_kpattern()).unwrap();

    //     if Phase1Checker::is_valid(&phase1_start_pattern) {
    //         return scramble_pattern;
    //     }

    //     println!("discarding invalid scramble"); //<<<}
    // }
}
