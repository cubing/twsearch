// use std::time::{Duration, Instant};

// use cubing::kpuzzle::KPuzzle;
use cubing::{alg::Alg, kpuzzle::KPattern};

use crate::_internal::cli::args::VerbosityLevel;
use crate::_internal::search::search_logger::SearchLogger;
use crate::experimental_lib_api::{SimpleMaskMultiphaseSearch, SimpleMaskPhaseInfo};
use crate::scramble::puzzles::definitions::{cube4x4x4_kpuzzle, cube4x4x4_phase1_target_kpattern};
use crate::{_internal::errors::SearchError, scramble::scramble_search::move_list_from_vec};

pub(crate) struct Cube4x4x4Solver {
    // kpuzzle: KPuzzle,
    // phase1_ifds: IDFSearch<KPuzzle>,
    phase1_search: SimpleMaskMultiphaseSearch,
    // square1_phase2_puzzle: Square1Phase2Puzzle,
    // phase2_idfs: IDFSearch<Square1Phase2Puzzle, Square1Phase2SearchAdaptations>,
    // // TODO: lazy-initialize `depth_filtering_search`?
    // pub(crate) depth_filtering_search: FilteredSearch<KPuzzle, FilteringSearchAdaptations>,
}

impl Cube4x4x4Solver {
    pub(crate) fn new() -> Self {
        let kpuzzle = cube4x4x4_kpuzzle();
        let generator_moves = move_list_from_vec(vec![
            "Uw", "U", "Lw", "L", "Fw", "F", "Rw", "R", "Bw", "B", "Dw", "D",
        ]);

        // let phase1_ifds = <IDFSearch>::try_new(
        //     kpuzzle.clone(),
        //     generator_moves,
        //     cube4x4x4_phase1_target_kpattern().clone(),
        //     IDFSearchConstructionOptions {
        //         search_logger: SearchLogger {
        //             verbosity: VerbosityLevel::Info,
        //         }
        //         .into(),
        //         ..Default::default()
        //     },
        // )
        // .unwrap();

        let phase1_search = SimpleMaskMultiphaseSearch::try_new(
            kpuzzle,
            vec![SimpleMaskPhaseInfo {
                name: "Place L/R centers on L/R".to_owned(),
                mask: cube4x4x4_phase1_target_kpattern().clone(),
                generator_moves,
                individual_search_options: None,
            }],
            Some(SearchLogger {
                verbosity: VerbosityLevel::Info,
            }), // Default::default(),
        )
        .unwrap();

        // let phase2_target_pattern = cube4x4x4_phase2_target_kpattern();
        // let phase2_idfs =
        //     IDFSearch::<Square1Phase2Puzzle, Square1Phase2SearchAdaptations>::try_new(
        //         kpuzzle.clone(),
        //         generator_moves.clone(),
        //         phase2_target_pattern,
        //         IDFSearchConstructionOptions {
        //             ..Default::default()
        //         },
        //     )
        //     .unwrap();

        // let depth_filtering_search = {
        //     let kpuzzle = square1_unbandaged_kpuzzle();
        //     let generator_moves = move_list_from_vec(vec!["U_SQ_", "D_SQ_", "/"]);

        //     let idfs = IDFSearch::<KPuzzle, FilteringSearchAdaptations>::try_new(
        //         kpuzzle.clone(),
        //         generator_moves,
        //         kpuzzle.default_pattern(),
        //         Default::default(),
        //     )
        //     .unwrap();
        //     FilteredSearch::<KPuzzle, FilteringSearchAdaptations>::new(idfs)
        // };

        Self {
            // kpuzzle: kpuzzle.clone(),
            // phase1_ifds,
            phase1_search, // square1_phase2_puzzle,
                           // phase2_idfs,
                           // depth_filtering_search,
        }
    }

    pub(crate) fn solve_4x4x4(&mut self, pattern: &KPattern) -> Result<Alg, SearchError> {
        // let masked = apply_mask(pattern, &cube4x4x4_phase1_target_kpattern()).unwrap();
        // let mut a = self.phase1_ifds.search(&masked, Default::default());
        // Ok(a.next().unwrap())
        self.phase1_search
            .chain_first_solution_for_each_phase(pattern)
        // dbg!(pattern.apply_alg(&alg).unwrap());
        // Ok(alg)
    }
}
