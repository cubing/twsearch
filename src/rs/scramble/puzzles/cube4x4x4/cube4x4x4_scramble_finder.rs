use std::sync::Arc;

use cubing::alg::{parse_alg, Alg};
use cubing::kpuzzle::KPuzzle;

use crate::_internal::cli::args::VerbosityLevel;
use crate::_internal::search::filter::filtering_decision::FilteringDecision;
use crate::_internal::search::move_count::MoveCount;
use crate::_internal::search::search_logger::SearchLogger;
use crate::experimental_lib_api::{
    KPuzzleSimpleMaskPhase, KPuzzleSimpleMaskPhaseConstructionOptions, MultiPhaseSearch,
};
use crate::scramble::get_kpuzzle::GetKPuzzle;
use crate::scramble::puzzles::canonicalizing_solved_kpattern_depth_filter::{
    CanonicalizingSolvedKPatternDepthFilter,
    CanonicalizingSolvedKPatternDepthFilterConstructionParameters,
};
use crate::scramble::puzzles::definitions::{
    cube4x4x4_kpuzzle, cube4x4x4_orientation_canonicalization_kpattern,
    cube4x4x4_phase1_target_kpattern, cube4x4x4_solved_kpattern,
};
use crate::{_internal::errors::SearchError, scramble::scramble_search::move_list_from_vec};

use crate::scramble::{
    collapse::collapse_adjacent_moves,
    randomize::{randomize_orbit, OrbitOrientationConstraint, OrbitRandomizationConstraints},
    solving_based_scramble_finder::{
        NoScrambleAssociatedData, NoScrambleOptions, SolvingBasedScrambleFinder,
    },
};

use super::phase2::phase2_search;
use super::phase3::Cube4x4x4Phase3Search;
use super::phase4::Cube4x4x4Phase4Search;

#[allow(non_upper_case_globals)]
const CUBE4x4x4_MINIMUM_OPTIMAL_SOLUTION_MOVE_COUNT: MoveCount = MoveCount(2);

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
│ (17)├───┬─┤  5├───┬─┤  9│───┬─┤ 13├───┬─┤ 17│    │
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
    canonicalizing_solved_kpattern_depth_filter: CanonicalizingSolvedKPatternDepthFilter,
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

        randomize_orbit(
            &mut scramble_pattern,
            0,
            "CORNERS",
            OrbitRandomizationConstraints {
                orientation: Some(OrbitOrientationConstraint::SumToZero),
                ..Default::default()
            },
        );
        randomize_orbit(&mut scramble_pattern, 1, "WINGS", Default::default());
        randomize_orbit(&mut scramble_pattern, 2, "CENTERS", Default::default());
        scramble_pattern = scramble_pattern.apply_alg(parse_alg!("R")).unwrap();

        (scramble_pattern, NoScrambleAssociatedData {})
    }

    fn filter_pattern(
        &mut self,
        pattern: &<<Self as SolvingBasedScrambleFinder>::TPuzzle as crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle>::Pattern,
        _scramble_associated_data: &Self::ScrambleAssociatedData,
        _scramble_options: &Self::ScrambleOptions,
    ) -> FilteringDecision {
        self.canonicalizing_solved_kpattern_depth_filter
            .depth_filter(pattern)
            .unwrap() // TODO
    }

    fn solve_pattern(
        &mut self,
        pattern: &<<Self as SolvingBasedScrambleFinder>::TPuzzle as crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle>::Pattern,
        _scramble_associated_data: &Self::ScrambleAssociatedData,
        _scramble_options: &Self::ScrambleOptions,
    ) -> Result<Alg, SearchError> {
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
        let search_logger = SearchLogger {
            verbosity: VerbosityLevel::Info,
        };

        let multi_phase_search = MultiPhaseSearch::try_new(
            kpuzzle.clone(),
            vec![
                Box::new(
                    KPuzzleSimpleMaskPhase::try_new(
                        "Place F/B centers on F/B".to_owned(),
                        cube4x4x4_phase1_target_kpattern().clone(),
                        phase1_generator_moves,
                        KPuzzleSimpleMaskPhaseConstructionOptions {
                            search_logger: Some(search_logger.clone()),
                            ..Default::default()
                        },
                    )
                    .unwrap(),
                ),
                Box::new(phase2_search(Arc::new(search_logger.clone()))),
                Box::new(Cube4x4x4Phase3Search::default()),
                Box::new(Cube4x4x4Phase4Search::default()),
            ],
            Some(SearchLogger {
                verbosity: VerbosityLevel::Info,
            }),
        )
        .unwrap();

        let canonicalizing_solved_kpattern_depth_filter =
            CanonicalizingSolvedKPatternDepthFilter::try_new(
                CanonicalizingSolvedKPatternDepthFilterConstructionParameters {
                    canonicalization_mask: cube4x4x4_orientation_canonicalization_kpattern()
                        .clone(),
                    canonicalization_generator_moves: move_list_from_vec(vec!["x", "y"]),
                    solved_pattern: cube4x4x4_solved_kpattern().clone(),
                    depth_filtering_generator_moves: move_list_from_vec(vec![
                        "3Uw", "Uw", "U", // U
                        "3Fw", "Fw", "F", // F
                        "3Rw", "Rw", "R", // R
                    ]),
                    min_optimal_solution_move_count: CUBE4x4x4_MINIMUM_OPTIMAL_SOLUTION_MOVE_COUNT,
                },
            )
            .unwrap();

        Self {
            canonicalizing_solved_kpattern_depth_filter,
            multi_phase_search,
        }
    }
}

impl GetKPuzzle for Cube4x4x4ScrambleFinder {
    fn get_kpuzzle(&self) -> &'static KPuzzle {
        cube4x4x4_kpuzzle()
    }
}
