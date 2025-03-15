use std::env::var;

use cubing::alg::{parse_alg, Alg};
use cubing::kpuzzle::KPuzzle;

use crate::_internal::cli::args::VerbosityLevel;
use crate::_internal::search::search_logger::SearchLogger;
use crate::experimental_lib_api::{
    KPuzzleSimpleMaskPhase, KPuzzleSimpleMaskPhaseConstructionOptions, MultiPhaseSearch,
};
use crate::scramble::puzzles::definitions::{
    cube4x4x4_kpuzzle, cube4x4x4_phase1_target_kpattern, cube4x4x4_phase2_target_kpattern,
};
use crate::scramble::solving_based_scramble_finder::FilteringDecision;
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
        let phase1_generator_moves = move_list_from_vec(vec![
            "Uw", "U", "Lw", "L", "Fw", "F", "Rw", "R", "Bw", "B", "Dw", "D",
        ]);
        let phase2_generator_moves = move_list_from_vec(vec![
            "Uw2", "U", "Lw", "L", "Fw2", "F", "Rw", "R", "Bw2", "B", "Dw2", "D",
        ]);

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
        .map(|alg| cube4x4x4_phase2_target_kpattern().apply_alg(alg).unwrap());

        let multi_phase_search = MultiPhaseSearch::try_new(
            kpuzzle.clone(),
            vec![
                Box::new(
                    KPuzzleSimpleMaskPhase::try_new(
                        "Place L/R centers on L/R".to_owned(),
                        cube4x4x4_phase1_target_kpattern().clone(),
                        phase1_generator_moves,
                        KPuzzleSimpleMaskPhaseConstructionOptions {
search_logger:
                            Some(search_logger.clone()),
                            ..Default::default()
                        },
                    )
                    .unwrap(),
                ),
                Box::new(
                    KPuzzleSimpleMaskPhase::try_new(
                        "Place F/B and U/D centers on correct axes and make L/R solvable with half turns".to_owned(),
                        cube4x4x4_phase2_target_kpattern().clone(),
                        phase2_generator_moves,
                        KPuzzleSimpleMaskPhaseConstructionOptions {
search_logger:
                            Some(search_logger.clone()),
                            masked_target_patterns:Some(phase2_target_patterns.to_vec()),
                            ..Default::default()
                        }
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
