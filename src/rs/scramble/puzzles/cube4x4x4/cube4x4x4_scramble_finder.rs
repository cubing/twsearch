use cubing::alg::{parse_alg, Alg};
use cubing::kpuzzle::{KPattern, KPuzzle};

use crate::_internal::cli::args::VerbosityLevel;
use crate::_internal::search::filter::filtering_decision::FilteringDecision;
use crate::_internal::search::move_count::MoveCount;
use crate::_internal::search::search_logger::SearchLogger;
use crate::experimental_lib_api::{
    KPuzzleSimpleMaskPhase, MultiPhaseSearch, MultiPhaseSearchOptions,
};
use crate::scramble::get_kpuzzle::GetKPuzzle;
use crate::scramble::puzzles::canonicalizing_solved_kpattern_depth_filter::{
    CanonicalizingSolvedKPatternDepthFilter,
    CanonicalizingSolvedKPatternDepthFilterConstructionParameters,
};
use crate::scramble::puzzles::definitions::{
    cube4x4x4_kpuzzle, cube4x4x4_orientation_canonicalization_kpattern,
    cube4x4x4_phase1_target_kpattern,
};
use crate::scramble::scramble_finder::scramble_finder::ScrambleFinder;
use crate::{_internal::errors::SearchError, scramble::scramble_search::move_list_from_vec};

use crate::scramble::{
    collapse::collapse_adjacent_moves,
    randomize::{randomize_orbit, OrbitOrientationConstraint, OrbitRandomizationConstraints},
    scramble_finder::solving_based_scramble_finder::{
        NoScrambleOptions, SolvingBasedScrambleFinder,
    },
};

use super::phase2::phase2_search;
use super::phase3::Cube4x4x4Phase3Search;
use super::phase4::Cube4x4x4Phase4Search;

#[allow(non_upper_case_globals)]
const CUBE4x4x4_MINIMUM_OPTIMAL_SOLUTION_MOVE_COUNT: MoveCount = MoveCount(2);

/*

Wings and centers are indexed by Speffz ordering: https://www.speedsolving.com/wiki/index.php?title=Speffz

│                  (16)                            │
╭──────────────────────────────────────────────────╮
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

impl ScrambleFinder for Cube4x4x4ScrambleFinder {
    type TPuzzle = KPuzzle;
    type ScrambleOptions = NoScrambleOptions;

    fn filter_pattern(
        &mut self,
        pattern: &KPattern,
        _scramble_options: &Self::ScrambleOptions,
    ) -> FilteringDecision {
        self.canonicalizing_solved_kpattern_depth_filter
            .depth_filter(pattern)
            .unwrap() // TODO
    }
}

impl SolvingBasedScrambleFinder for Cube4x4x4ScrambleFinder {
    fn generate_fair_unfiltered_random_pattern(
        &mut self,
        _scramble_options: &Self::ScrambleOptions,
    ) -> KPattern {
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

        scramble_pattern
    }

    fn solve_pattern(
        &mut self,
        pattern: &KPattern,
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

        let multi_phase_search = MultiPhaseSearch::try_new(
            kpuzzle.clone(),
            vec![
                Box::new(
                    KPuzzleSimpleMaskPhase::try_new(
                        "Place F/B centers on F/B".to_owned(),
                        cube4x4x4_phase1_target_kpattern().clone(),
                        phase1_generator_moves,
                        Default::default(),
                    )
                    .unwrap(),
                ),
                Box::new(phase2_search(Default::default())),
                Box::new(Cube4x4x4Phase3Search::default()),
                Box::new(Cube4x4x4Phase4Search::default()),
            ],
            MultiPhaseSearchOptions {
                // TODO: change the verbosity once 4×4×4 scrambles are much faster.
                search_logger: SearchLogger {
                    verbosity: VerbosityLevel::Info,
                },
                include_pause_between_phases: false,
            },
        )
        .unwrap();

        let canonicalizing_solved_kpattern_depth_filter =
            CanonicalizingSolvedKPatternDepthFilter::try_new(
                CanonicalizingSolvedKPatternDepthFilterConstructionParameters {
                    canonicalization_mask: cube4x4x4_orientation_canonicalization_kpattern()
                        .clone(),
                    canonicalization_generator_moves: move_list_from_vec(vec!["x", "y"]),
                    max_canonicalizing_move_count_below: MoveCount(4),
                    solved_pattern: kpuzzle.default_pattern(),
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

#[cfg(test)]
mod tests {
    use crate::scramble::{
        puzzles::{
            cube4x4x4::cube4x4x4_scramble_finder::Cube4x4x4ScrambleFinder,
            definitions::cube4x4x4_kpuzzle,
        },
        scramble_finder::scramble_finder::ScrambleFinder,
    };
    use cubing::alg::{parse_alg, Alg};

    #[test]
    // TODO: generalize and automate this across all events.
    fn simple_scramble_filtering_test_4x4x4() -> Result<(), String> {
        let mut scramble_finder = Cube4x4x4ScrambleFinder::default();
        let pattern = |alg: &Alg| {
            cube4x4x4_kpuzzle()
                .default_pattern()
                .apply_alg(alg)
                .unwrap()
        };
        assert!(scramble_finder
            .filter_pattern(&pattern(parse_alg!("z")), &Default::default())
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(&pattern(parse_alg!("x y x")), &Default::default())
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(&pattern(parse_alg!("Lw z Uw")), &Default::default())
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(&pattern(parse_alg!("Lw z Uw' R")), &Default::default())
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(&pattern(parse_alg!("Rw z' Uw")), &Default::default())
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(&pattern(parse_alg!("R U")), &Default::default())
            .is_accept());
        assert!(scramble_finder
            .filter_pattern(&pattern(parse_alg!("Rw L")), &Default::default())
            .is_accept());
        assert!(scramble_finder
            .filter_pattern(&pattern(parse_alg!("U L F R B D")), &Default::default())
            .is_accept());
        assert!(scramble_finder
            .filter_pattern(&pattern(parse_alg!("U F 3Rw 3Uw2")), &Default::default())
            .is_accept());
        assert!(scramble_finder
            .filter_pattern(&pattern(parse_alg!("Rw Lw")), &Default::default())
            .is_reject());
        Ok(())
    }
}
