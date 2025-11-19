use cubing::{
    alg::{parse_move, Alg, AlgNode, Move},
    kpuzzle::KPuzzle,
};
use rand::{rng, Rng};

use crate::{
    _internal::search::move_count::MoveCount,
    scramble::{
        scramble_finder::{
            random_move_scramble_finder::RandomMoveScrambleFinder, scramble_finder::ScrambleFinder,
            solving_based_scramble_finder::NoScrambleOptions,
        },
        scramble_search::move_list_from_vec,
    },
};

use super::{
    canonicalizing_solved_kpattern_depth_filter::{
        CanonicalizingSolvedKPatternDepthFilter,
        CanonicalizingSolvedKPatternDepthFilterConstructionParameters,
    },
    definitions::{clock_kpuzzle, clock_orientation_canonicalization_kpattern},
};

#[allow(non_upper_case_globals)]
const CUBE4x4x4_MINIMUM_OPTIMAL_SOLUTION_MOVE_COUNT: MoveCount = MoveCount(2);

pub struct ClockScrambleFinder {
    depth_filtering_search: CanonicalizingSolvedKPatternDepthFilter,
}

impl Default for ClockScrambleFinder {
    fn default() -> Self {
        let kpuzzle = clock_kpuzzle();
        let depth_filtering_search = CanonicalizingSolvedKPatternDepthFilter::try_new(
            CanonicalizingSolvedKPatternDepthFilterConstructionParameters {
                canonicalization_mask: clock_orientation_canonicalization_kpattern().clone(),
                canonicalization_generator_moves: move_list_from_vec(vec!["z", "y2"]),
                max_canonicalizing_move_count_below: MoveCount(3),
                solved_pattern: kpuzzle.default_pattern(),
                depth_filtering_generator_moves: move_list_from_vec(vec![
                    "U_PLUS_",
                    "R_PLUS_",
                    "D_PLUS_",
                    "L_PLUS_",
                    "B_PLUS_",
                    "F_PLUS_",
                    "UR_PLUS_",
                    "DR_PLUS_",
                    "DL_PLUS_",
                    "UL_PLUS_",
                    "ULw_PLUS_",
                    "URw_PLUS_",
                    "DLw_PLUS_",
                    "DRw_PLUS_",
                    "BULw_PLUS_",
                    "BURw_PLUS_",
                    "BDLw_PLUS_",
                    "BDRw_PLUS_",
                    "BU_PLUS_",
                    "BR_PLUS_",
                    "BD_PLUS_",
                    "BL_PLUS_",
                    "BUR_PLUS_",
                    "BUL_PLUS_",
                    "BDL_PLUS_",
                    "BDR_PLUS_",
                    "MUL_PLUS_",
                    "MUR_PLUS_",
                    "BMUL_PLUS_",
                    "BMUR_PLUS_",
                    "BMDR_PLUS_",
                    "BMDL_PLUS_",
                ]),
                min_optimal_solution_move_count: CUBE4x4x4_MINIMUM_OPTIMAL_SOLUTION_MOVE_COUNT,
            },
        )
        .unwrap();
        Self {
            depth_filtering_search,
        }
    }
}

impl ScrambleFinder for ClockScrambleFinder {
    type TPuzzle = KPuzzle;
    type ScrambleOptions = NoScrambleOptions;

    fn filter_pattern(
        &mut self,
        pattern: &<<Self as ScrambleFinder>::TPuzzle as crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle>::Pattern,
        _scramble_options: &NoScrambleOptions,
    ) -> crate::_internal::search::filter::filtering_decision::FilteringDecision {
        self.depth_filtering_search.depth_filter(pattern).unwrap() // TODO: avoid the need to `.unwrap()`
    }
}

// TODO: This should probably be a `SolvingBasedScrambleFinder`?
impl RandomMoveScrambleFinder for ClockScrambleFinder {
    fn generate_unfiltered_random_move_scramble(
        &mut self,
        _scramble_options: &NoScrambleOptions,
    ) -> Alg {
        let mut rng = rng();
        let mut alg_nodes = Vec::<AlgNode>::new();

        // TODO: implement `parse_quantum_move!(…)`?
        let back_moves = vec![
            parse_move!("U_PLUS_").quantum.to_owned(),
            parse_move!("R_PLUS_").quantum.to_owned(),
            parse_move!("D_PLUS_").quantum.to_owned(),
            parse_move!("L_PLUS_").quantum.to_owned(),
            parse_move!("ALL_PLUS_").quantum.to_owned(),
        ];

        let front_moves = [
            back_moves.clone(),
            vec![
                parse_move!("UR_PLUS_").quantum.to_owned(),
                parse_move!("DR_PLUS_").quantum.to_owned(),
                parse_move!("DL_PLUS_").quantum.to_owned(),
                parse_move!("UL_PLUS_").quantum.to_owned(),
            ],
        ]
        .concat();

        for front_move in front_moves {
            alg_nodes.push(
                Move {
                    quantum: front_move,
                    amount: rng.random_range(-5..7),
                }
                .into(),
            );
        }
        alg_nodes.push(parse_move!("y2").clone().into());
        for back_move in back_moves {
            alg_nodes.push(
                Move {
                    quantum: back_move,
                    amount: rng.random_range(-5..7),
                }
                .into(),
            );
        }

        Alg { nodes: alg_nodes }
    }

    fn puzzle(&self) -> &KPuzzle {
        clock_kpuzzle()
    }
}

#[cfg(test)]
mod tests {
    use cubing::alg::{parse_alg, Alg};

    use crate::scramble::{
        puzzles::{clock_scramble_finder::ClockScrambleFinder, definitions::clock_kpuzzle},
        scramble_finder::{
            scramble_finder::ScrambleFinder, solving_based_scramble_finder::NoScrambleOptions,
        },
    };

    #[test]
    // TODO: generalize and automate this across all events.
    fn simple_scramble_filtering_test() -> Result<(), String> {
        let mut scramble_finder = ClockScrambleFinder::default();
        let pattern = |alg: &Alg| clock_kpuzzle().default_pattern().apply_alg(alg).unwrap();
        assert!(scramble_finder
            .filter_pattern(&pattern(parse_alg!("R1+ y2 BL2+")), &NoScrambleOptions {},)
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!(
                    "UR1- DR0+ DL0+ UL0- U0- R0+ D0+ L0- ALL0- y2 U0+ R0- D0- L0+ ALL0+"
                )),
                &NoScrambleOptions {},
            )
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(&pattern(parse_alg!("R2+ ALL4-")), &NoScrambleOptions {},)
            .is_accept());
        Ok(())
    }
}
