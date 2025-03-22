use std::env::var;

use cubing::alg::{parse_alg, Alg};
use cubing::kpuzzle::{KPattern, KPuzzle};

use crate::_internal::cli::args::VerbosityLevel;
use crate::_internal::search::coordinates::graph_enumerated_derived_pattern_puzzle::DerivedPattern;
use crate::_internal::search::filter::filtering_decision::FilteringDecision;
use crate::_internal::search::search_logger::SearchLogger;
use crate::experimental_lib_api::{
    KPuzzleSimpleMaskPhase, KPuzzleSimpleMaskPhaseConstructionOptions, MultiPhaseSearch,
};
use crate::scramble::puzzles::definitions::{
    cube4x4x4_kpuzzle, cube4x4x4_phase1_target_kpattern, cube4x4x4_phase2_centers_target_kpattern,
};
use crate::scramble::randomize::{basic_parity, BasicParity};
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct WingParityPattern {
    parity: BasicParity,
}

impl DerivedPattern<KPuzzle> for WingParityPattern {
    fn derived_pattern_name() -> &'static str {
        "phase 2 wing parity"
    }

    fn try_new(_puzzle: &KPuzzle, pattern: &KPattern) -> Option<Self> {
        Some(Self {
            parity: basic_parity(WingParityPattern::wing_permutation_slice(pattern)),
        })
    }
}

impl WingParityPattern {
    // TODO: is there a good way to implement this without `unsafe`? Or should we expose this directly on `KPattern`?
    pub(crate) fn wing_permutation_slice(pattern: &KPattern) -> &[u8] {
        let orbit = &pattern.kpuzzle().data.ordered_orbit_info[0];
        assert_eq!(orbit.name.0, "WINGS");

        let from = orbit.orientations_offset;
        let to = from + (orbit.num_pieces as usize);

        let full_byte_slice = unsafe { pattern.byte_slice() };
        &full_byte_slice[from..to]
    }
}
