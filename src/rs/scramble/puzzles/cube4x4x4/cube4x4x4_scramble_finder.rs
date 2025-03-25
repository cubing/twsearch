use std::env::var;

use cubing::alg::{parse_alg, parse_move, Alg};
use cubing::kpuzzle::KPuzzle;

use crate::_internal::cli::args::VerbosityLevel;
use crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle;
use crate::_internal::search::coordinates::pattern_deriver::PatternDeriver;
use crate::_internal::search::filter::filtering_decision::FilteringDecision;
use crate::_internal::search::search_logger::SearchLogger;
use crate::experimental_lib_api::{
    KPuzzleSimpleMaskPhase, KPuzzleSimpleMaskPhaseConstructionOptions, MultiPhaseSearch,
};
use crate::scramble::puzzles::cube4x4x4::phase2::Cube4x4x4Phase2Puzzle;
use crate::scramble::puzzles::definitions::{
    cube4x4x4_kpuzzle, cube4x4x4_phase1_target_kpattern, cube4x4x4_phase2_centers_target_kpattern,
};
use crate::{_internal::errors::SearchError, scramble::scramble_search::move_list_from_vec};

use crate::scramble::{
    collapse::collapse_adjacent_moves,
    randomize::{
        randomize_orbit_naïve, OrbitOrientationConstraint, OrbitRandomizationConstraints
    },
    solving_based_scramble_finder::{
        NoScrambleAssociatedData, NoScrambleOptions, SolvingBasedScrambleFinder,
    },
};

/*

Wings and centers are indexed by Speffz ordering: https://www.speedsolving.com/wiki/index.php?title=Speffz

╭──────────────────────────────────────────────────╮
│                  (16)                            │
│               ╭─────┬───╮                        │
│               │    0│   │                        │
│               ├───┬─┤  1│                        │
│               │3  ├─┴───┤(12)                    │
│         (3)   │   │2    │   (1)       (0)        │
│     ╭─────┬───┼───┴─┬───┼─────┬───┬─────┬───╮    │
│     │    4│   │    8│   │   12│   │16   │   │    │
│ (17)├───┬─┤  5├───┬─┤  9│───┬─┤ 13├───┬─┤  7│    │
│     │7  ├─┴───┤11 ├─┴───┤15 ├─┴───┤19 ├─┴───┤(7) │
│     │   │6    │   │10   │   │14   │   │18   │    │
│     ╰───┴─────┼───┴─────┼───┴─────┴───┴─────╯    │
│         (23)  │   20│   │   (21)      (22)       │
│            (6)├───┬─┤ 21│                        │
│               │23 ├─┴───┤(14)                    │
│               │   │22   │                        │
│               ╰───┴─────╯                        │
│                   (18)                           │
╰──────────────────────────────────────────────────╯

*/

pub(crate) struct Cube4x4x4ScrambleFinder {
    multi_phase_search: MultiPhaseSearch<KPuzzle>,
}

fn run_incomplete_scramble_finder_check() {
    let run_incomplete_scramble_finders = match var("RUN_INCOMPLETE_SCRAMBLE_FINDERS") {
        Ok(value) => value == "true",
        _ => false,
    };
    if !run_incomplete_scramble_finders {
        panic!("To run this finder, use: `env RUN_INCOMPLETE_SCRAMBLE_FINDERS=true`")
    }
}

impl SolvingBasedScrambleFinder for Cube4x4x4ScrambleFinder {
    type TPuzzle = KPuzzle;
    type ScrambleAssociatedData = NoScrambleAssociatedData;
    type ScrambleOptions = NoScrambleOptions;

    fn generate_fair_unfiltered_random_pattern(
        &mut self,
        _scramble_options: &Self::ScrambleOptions,
    ) -> (
        <<Self as SolvingBasedScrambleFinder>::TPuzzle as crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle>::Pattern,
        Self::ScrambleAssociatedData,
    ){
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

        (scramble_pattern, NoScrambleAssociatedData {})
    }

    fn filter_pattern(
        &mut self,
        _pattern: &<<Self as SolvingBasedScrambleFinder>::TPuzzle as crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle>::Pattern,
        _scramble_associated_data: &Self::ScrambleAssociatedData,
        _scramble_options: &Self::ScrambleOptions,
    ) -> FilteringDecision {
        dbg!("WARNING: 4×4×4 filtering is not implemented yet.");
        FilteringDecision::Accept
    }

    fn solve_pattern(
        &mut self,
        pattern: &<<Self as SolvingBasedScrambleFinder>::TPuzzle as crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle>::Pattern,
        _scramble_associated_data: &Self::ScrambleAssociatedData,
        _scramble_options: &Self::ScrambleOptions,
    ) -> Result<Alg, SearchError> {
        run_incomplete_scramble_finder_check();

        self.multi_phase_search
            .chain_first_solution_for_each_phase(pattern)
    }

    fn collapse_inverted_alg(&mut self, alg: Alg) -> Alg {
        collapse_adjacent_moves(alg, 4, -1)
    }
}

impl Default for Cube4x4x4ScrambleFinder {
    fn default() -> Self {
        let kpuzzle = cube4x4x4_kpuzzle();
        // TODO: should we reduce the move count here as we've done for phase2? For example, Fw and
        // Bw are redundant, right?
        let phase1_generator_moves = move_list_from_vec(vec![
            "Uw", "U", "Lw", "L", "Fw", "F", "Rw", "R", "Bw", "B", "Dw", "D",
        ]);
        let phase2_generator_moves =
            move_list_from_vec(vec!["Uw2", "U", "L", "Fw2", "F", "Rw", "R", "B", "D"]);

        // let phase1_ifds = <IterativeDeepeningSearch>::try_new(
        //     kpuzzle.clone(),
        //     generator_moves,
        //     cube4x4x4_phase1_target_kpattern().clone(),
        //     IterativeDeepeningSearchConstructionOptions {
        //         search_logger: SearchLogger {
        //             verbosity: VerbosityLevel::Info,
        //         }
        //         .into(),
        //         ..Default::default()
        //     },
        // )
        // .unwrap();

        let search_logger = SearchLogger {
            verbosity: VerbosityLevel::Info,
        };

        let phase2_target_patterns = [
            parse_alg!(""),
            parse_alg!("y2"),
            parse_alg!("Fw2"),
            parse_alg!("Bw2"),
            parse_alg!("Uw2"),
            parse_alg!("Dw2"),
            parse_alg!("Fw2 Rw2"),
            parse_alg!("Bw2 Rw2"),
            parse_alg!("Uw2 Rw2"),
            parse_alg!("Dw2 Rw2"),
            parse_alg!("Dw2 Rw2 Fw2"),
            parse_alg!("Fw2 Rw2 Uw2"),
        ]
        .map(|alg| {
            cube4x4x4_phase2_centers_target_kpattern()
                .apply_alg(alg)
                .unwrap()
        });

        // This would be inline, but we need to work around https://github.com/cubing/twsearch/issues/128
        let phase2_name =
            "Place F/B and U/D centers on correct axes and make L/R solvable with half turns"
                .to_owned();

        dbg!(parse_move!("2R"));

        let cube4x4x4_phase2_puzzle = Cube4x4x4Phase2Puzzle::default();

        let derived_pattern = cube4x4x4_phase2_puzzle
            .derive_pattern(&kpuzzle.default_pattern())
            .unwrap();
        dbg!(&derived_pattern);
        let derived_transformation = cube4x4x4_phase2_puzzle
            .puzzle_transformation_from_move(parse_move!("2R"))
            .unwrap();
        dbg!(&derived_transformation);
        dbg!(cube4x4x4_phase2_puzzle
            .pattern_apply_transformation(&derived_pattern, &derived_transformation));

        // let g: <CompoundDerivedPuzzle<
        //     WingParityPatternDeriver,
        //     WingParityPatternDeriver,
        // > as SemiGroupActionPuzzle>::Pattern = (WingParityPattern {
        //     parity: crate::scramble::randomize::BasicParity::Even,
        // }, WingParityPattern {
        //     parity: crate::scramble::randomize::BasicParity::Odd,
        // });

        // dbg!(f);
        // dbg!(g);

        let multi_phase_search = MultiPhaseSearch::try_new(
            kpuzzle.clone(),
            vec![
                Box::new(
                    KPuzzleSimpleMaskPhase::try_new(
                        "Place L/R centers on L/R".to_owned(),
                        cube4x4x4_phase1_target_kpattern().clone(),
                        phase1_generator_moves,
                        KPuzzleSimpleMaskPhaseConstructionOptions {
                            // TODO: figure out why the linter and formatter aren't catching this indentation: https://github.com/cubing/twsearch/issues/128
                            search_logger: Some(search_logger.clone()),
                            ..Default::default()
                        },
                    )
                    .unwrap(),
                ),
                // TODO: filter out phase2 solutions that don't obey the EP rules.
                //                   00 01 02 03 04 05 06 07 08 09 10 11 12 13 14 15 16 17 18 19 20 21 22 23
                //
                // Analyzing move U: 03 00 01 02 08 05 06 07 12 09 10 11 16 13 14 15 04 17 18 19 20 21 22 23
                //                    *  *  *  *  *           *           *           *
                //
                //
                // Analyzing move D: 00 01 02 03 04 05 18 07 08 09 06 11 12 13 10 15 16 17 14 19 23 20 21 22
                //                                      *           *           *           *     *  *  *  *
                //
                //
                //
                // Analyzing move B: 13 01 02 03 04 05 06 00 08 09 10 11 12 22 14 15 19 16 17 18 20 21 07 23
                //                    *                    *                 *        *  *  *  *        *
                //
                // Analyzing move R: 00 09 02 03 04 05 06 07 08 21 10 11 15 12 13 14 16 17 18 01 20 19 22 23
                //                       *                       *        *  *  *  *           *     *
                //
                // Analyzing move F: 00 01 05 03 04 20 06 07 11 08 09 10 12 13 14 02 16 17 18 19 15 21 22 23
                //                          *        *        *  *  *  *           *              *
                // Analyzing move L: 00 01 02 17 07 04 05 06 08 09 10 03 12 13 14 15 16 23 18 19 20 21 22 11
                //                             *  *  *  *  *           *                 *                 *
                //
                //                   00 01 02 03 04 05 06 07 08 09 10 11 12 13 14 15 16 17 18 19 20 21 22 23
                //
                // slot_to_highness = [ 0,  0, 0, 0, 1,  0,  1,  0, 1,  1,  1, 1, 1,  0,  1, 0, 1, 1,  1,  1,  0,  0,  0, 0]
                // edge_partner =     [16, 12, 8, 4, 3, 11, 23, 17, 2, 15, 20, 5, 1, 19, 21, 9, 0, 7, 22, 13, 10, 14, 18, 6]
                //
                // "low":   [0, 1, 2, 3, 5, 7, 13, 15, 20, 21, 22, 23]
                // "high":  [4, 6, 8, 9, 10, 11, 12, 14, 16, 17, 18, 19]
                //
                // BR = 13, 19
                // BL = 7, 17
                // FR = 9, 15
                // FL = 5, 11
                // DR = 14, 21
                // UF = 2, 8
                // DF = 10, 20
                // UB = 0, 16
                // DB = 18, 22
                // UL = 3, 4
                // UR = 1, 12
                // DL = 6, 23
                Box::new(
                    KPuzzleSimpleMaskPhase::try_new(
                        phase2_name,
                        cube4x4x4_phase2_centers_target_kpattern().clone(),
                        phase2_generator_moves,
                        KPuzzleSimpleMaskPhaseConstructionOptions {
                            search_logger: Some(search_logger.clone()),
                            masked_target_patterns: Some(phase2_target_patterns.to_vec()),
                            ..Default::default()
                        },
                    )
                    .unwrap(),
                ),
            ],
            // Default::default(),
            Some(SearchLogger {
                verbosity: VerbosityLevel::Info,
            }),
        )
        .unwrap();

        // let phase2_target_pattern = cube4x4x4_phase2_target_kpattern();
        // let phase2_iterative_deepening_search =
        //     IterativeDeepeningSearch::<Square1Phase2Puzzle, Square1Phase2SearchAdaptations>::try_new(
        //         kpuzzle.clone(),
        //         generator_moves.clone(),
        //         phase2_target_pattern,
        //         IterativeDeepeningSearchConstructionOptions {
        //             ..Default::default()
        //         },
        //     )
        //     .unwrap();

        // let depth_filtering_search = {
        //     let kpuzzle = square1_unbandaged_kpuzzle();
        //     let generator_moves = move_list_from_vec(vec!["U_SQ_", "D_SQ_", "/"]);

        //     let iterative_deepening_search = IterativeDeepeningSearch::<KPuzzle, FilteringSearchAdaptations>::try_new(
        //         kpuzzle.clone(),
        //         generator_moves,
        //         kpuzzle.default_pattern(),
        //         Default::default(),
        //     )
        //     .unwrap();
        //     FilteredSearch::<KPuzzle, FilteringSearchAdaptations>::new(iterative_deepening_search)
        // };

        Self {
            // kpuzzle: kpuzzle.clone(),
            // phase1_ifds,
            multi_phase_search, // square1_phase2_puzzle,
                                // phase2_iterative_deepening_search,
                                // depth_filtering_search,
        }
    }
}

impl Cube4x4x4ScrambleFinder {
    pub fn get_kpuzzle() -> &'static KPuzzle {
        cube4x4x4_kpuzzle()
    }
}

// #[derive(Debug, Clone, Hash, PartialEq, Eq)]
// struct Phase2WingSeparationPattern {
//     pattern: KPattern,
// }

// #[derive(Clone, Debug)]
// struct Phase2WingSeparationPuzzle {}

// impl DerivedPatternPuzzle<KPuzzle> for Phase2WingSeparationPuzzle {
//     type DerivedPattern = Phase2WingSeparationPattern;

//     fn derive_pattern(&self, source_puzzle_pattern: &KPattern) -> Phase2WingSeparationPattern {
//         let Ok(pattern) = apply_mask(
//             source_puzzle_pattern,
//             cube4x4x4_phase2_wing_separation_mask_kpattern(),
//         ) else {
//             return None;
//         };
//         Phase2WingSeparationPattern { pattern }
//     }

//     // fn derived_pattern_name() -> &'static str {
//     //     "phase 2 wing separation"
//     // }

//     // fn try_new(_puzzle: &KPuzzle, pattern: &KPattern) -> Option<Self> {
//     //     let Ok(pattern) = apply_mask(pattern, cube4x4x4_phase2_wing_separation_mask_kpattern())
//     //     else {
//     //         return None;
//     //     };
//     //     Some(Self { pattern })
//     // }
// }
