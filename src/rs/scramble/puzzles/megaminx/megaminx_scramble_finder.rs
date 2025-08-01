use cubing::alg::{parse_move, Alg, AlgNode, Move, Newline};
use cubing::kpuzzle::{KPattern, KPuzzle};
use rand::{thread_rng, Rng};

use crate::_internal::search::move_count::MoveCount;
use crate::scramble::puzzles::canonicalizing_solved_kpattern_depth_filter::{
    CanonicalizingSolvedKPatternDepthFilter,
    CanonicalizingSolvedKPatternDepthFilterConstructionParameters,
};
use crate::{
    _internal::search::filter::filtering_decision::FilteringDecision,
    scramble::{
        puzzles::definitions::{megaminx_kpuzzle, megaminx_phase1_target_kpattern},
        scramble_finder::{
            random_move_scramble_finder::RandomMoveScrambleFinder, scramble_finder::ScrambleFinder,
            solving_based_scramble_finder::NoScrambleOptions,
        },
        scramble_search::move_list_from_vec,
    },
};

const RANDOM_MOVE_SCRAMBLE_NUM_LINES: usize = 7;
const RANDOM_MOVE_SCRAMBLE_NUM_MOVE_PAIRS_PER_LINE: usize = 5;

// Note this would be called `MegaminxScrambleFinder`, but we've avoiding that
// name because we're not using it to generate scrambles.
pub struct MegaminxScrambleFinder {
    canonicalizing_solved_kpattern_depth_filter: CanonicalizingSolvedKPatternDepthFilter,
}

#[allow(non_upper_case_globals)]
const MEGAMINX_MINIMUM_OPTIMAL_SOLUTION_MOVE_COUNT: MoveCount = MoveCount(2);

impl Default for MegaminxScrambleFinder {
    fn default() -> Self {
        let canonicalizing_solved_kpattern_depth_filter =
            CanonicalizingSolvedKPatternDepthFilter::try_new(
                CanonicalizingSolvedKPatternDepthFilterConstructionParameters {
                    canonicalization_mask: megaminx_phase1_target_kpattern().clone(),
                    canonicalization_generator_moves: move_list_from_vec(vec!["Uv", "Lv"]),
                    max_canonicalizing_move_count_below: MoveCount(5),
                    solved_pattern: megaminx_kpuzzle().default_pattern().clone(),
                    depth_filtering_generator_moves: move_list_from_vec(vec![
                        "U", "L", "F", "R", "BR", "BL", "FL", "FR", "DL", "DR", "B", "D",
                    ]),
                    min_optimal_solution_move_count: MEGAMINX_MINIMUM_OPTIMAL_SOLUTION_MOVE_COUNT,
                },
            )
            .unwrap();

        Self {
            canonicalizing_solved_kpattern_depth_filter,
        }
    }
}

impl ScrambleFinder for MegaminxScrambleFinder {
    type TPuzzle = KPuzzle;
    type ScrambleOptions = NoScrambleOptions;

    fn filter_pattern(
        &mut self,
        pattern: &KPattern,
        _scramble_options: &Self::ScrambleOptions,
    ) -> FilteringDecision {
        self.canonicalizing_solved_kpattern_depth_filter
            .depth_filter(pattern)
            .unwrap() // TODO: avoid `.unwrap()`.
    }
}

impl RandomMoveScrambleFinder for MegaminxScrambleFinder {
    fn generate_unfiltered_random_move_scramble(
        &mut self,
        _scramble_options: &Self::ScrambleOptions,
    ) -> Alg {
        let mut rng = thread_rng();
        let mut alg_nodes = Vec::<AlgNode>::new();

        let r_array: [&Move; 2] = [parse_move!("R++"), parse_move!("R--")];
        let d_array: [&Move; 2] = [parse_move!("D++"), parse_move!("D--")];
        let u_array: [&Move; 2] = [parse_move!("U"), parse_move!("U'")];

        for _ in 0..RANDOM_MOVE_SCRAMBLE_NUM_LINES {
            let mut random_choice: usize = 0;
            for _ in 0..RANDOM_MOVE_SCRAMBLE_NUM_MOVE_PAIRS_PER_LINE {
                for arr in [&r_array, &d_array] {
                    random_choice = rng.gen_range(0..=1);
                    alg_nodes.push(arr[random_choice].clone().into());
                }
            }
            // Match TNoodle:
            //
            // - `D++` is followed by `U`
            // - `D--` is followed by `U'`
            alg_nodes.push(u_array[random_choice].clone().into());

            alg_nodes.push(Newline::default().into());
        }
        alg_nodes.pop(); // Remove the last newline.

        Alg { nodes: alg_nodes }
    }

    fn puzzle(&self) -> &Self::TPuzzle {
        megaminx_kpuzzle()
    }
}
