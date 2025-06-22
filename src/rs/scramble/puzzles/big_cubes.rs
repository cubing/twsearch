use std::{marker::PhantomData, sync::Arc};

use cubing::{
    alg::{Alg, AlgNode, Move, MoveLayer, MovePrefix, QuantumMove},
    kpuzzle::{KPattern, KPuzzle, OrientationWithMod},
};
use num_integer::Integer;
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::{
    _internal::{
        canonical_fsm::{
            canonical_fsm::{CanonicalFSM, CANONICAL_FSM_START_STATE},
            move_class_mask::MoveClassIndex,
            search_generators::SearchGenerators,
        },
        cli::args::MetricEnum,
        search::{filter::filtering_decision::FilteringDecision, move_count::MoveCount},
    },
    scramble::{
        scramble_finder::{
            random_move_scramble_finder::RandomMoveScrambleFinder, scramble_finder::ScrambleFinder,
        },
        scramble_search::move_list_from_vec,
    },
};

use super::{
    canonicalizing_solved_kpattern_depth_filter::{
        CanonicalizingSolvedKPatternDepthFilter,
        CanonicalizingSolvedKPatternDepthFilterConstructionParameters,
    },
    definitions::{cube5x5x5_kpuzzle, cube6x6x6_kpuzzle, cube7x7x7_kpuzzle},
    static_move_list::add_random_suffixes_from,
};

#[allow(non_upper_case_globals)]
const BIG_CUBES_MINIMUM_OPTIMAL_SOLUTION_MOVE_COUNT: MoveCount = MoveCount(2);

const SPEFFZ_DLB_INDEX: u8 = 6;

pub struct BigCubeScrambleInfo {
    kpuzzle: KPuzzle,
    size: u32,
}

fn face_or_wide_move(inner_slice: u32, family_face: &str, amount: i32) -> Move {
    let family = if inner_slice == 1 {
        family_face.to_owned()
    } else {
        format!("{}w", family_face)
    };
    let prefix = if inner_slice > 2 {
        Some(MovePrefix::Layer(MoveLayer::new(inner_slice)))
    } else {
        None
    };
    let quantum = Arc::new(QuantumMove { family, prefix });
    Move { quantum, amount }
}

impl BigCubeScrambleInfo {
    fn depth_filtering_generator_moves(&self) -> Vec<Move> {
        let mut output = vec![];
        for family_face in ["U", "F", "R"] {
            for inner_slice in 1..=(self.size - 1) {
                output.push(face_or_wide_move(inner_slice, family_face, 1));
            }
        }
        output
    }

    fn scrambling_generator_moves(&self) -> Vec<Move> {
        let mut output = vec![];
        for family_face in ["U", "L", "F", "R", "B", "D"] {
            for inner_slice in 1..=(self.size) / 2 {
                output.push(face_or_wide_move(inner_slice, family_face, 1));
            }
        }
        output
    }

    fn orientation_canonicalization_mask(&self) -> KPattern {
        let mut mask = self.kpuzzle.default_pattern();
        for orbit_info in self.kpuzzle.orbit_info_iter() {
            #[allow(non_snake_case)]
            let orbit_is_CORNERS = orbit_info.num_pieces == 8;

            for i in 0..orbit_info.num_pieces {
                if orbit_is_CORNERS && (i == SPEFFZ_DLB_INDEX) {
                    continue;
                }
                mask.set_orientation_with_mod(
                    orbit_info,
                    i,
                    &OrientationWithMod {
                        orientation: 0,
                        orientation_mod: 1,
                    },
                );
                mask.set_piece(orbit_info, i, 0);
            }
        }
        mask
    }

    fn num_random_moves(&self) -> MoveCount {
        assert!(self.size >= 5);
        MoveCount((self.size as usize) * 20 - 40)
    }

    fn no_inspection_suffixes_from(&self) -> [Vec<Option<Move>>; 2] {
        assert!(self.size.is_odd());
        let layer = self.size.div_ceil(2);
        let s1 = vec![
            None,
            Some(face_or_wide_move(layer, "R", 1)),
            Some(face_or_wide_move(layer, "R", 2)),
            Some(face_or_wide_move(layer, "R", -1)),
            Some(face_or_wide_move(layer, "F", 1)),
            Some(face_or_wide_move(layer, "F", -1)),
        ];
        let s2 = vec![
            None,
            Some(face_or_wide_move(layer, "U", 1)),
            Some(face_or_wide_move(layer, "U", 2)),
            Some(face_or_wide_move(layer, "U", -1)),
        ];
        [s1, s2]
    }
}

pub trait BigCube: Default + Send + Sync {
    fn info() -> BigCubeScrambleInfo;
}

pub struct BigCubeScrambleFinder<TBigCube: BigCube> {
    info: BigCubeScrambleInfo,
    canonicalizing_solved_kpattern_depth_filter: CanonicalizingSolvedKPatternDepthFilter,
    generators: SearchGenerators<KPuzzle>,
    canonical_fsm: CanonicalFSM<KPuzzle>,
    phantom_data: PhantomData<TBigCube>,
}

impl<TBigCube: BigCube> Default for BigCubeScrambleFinder<TBigCube> {
    fn default() -> Self {
        let info = TBigCube::info();
        let canonicalizing_solved_kpattern_depth_filter =
            CanonicalizingSolvedKPatternDepthFilter::try_new(
                CanonicalizingSolvedKPatternDepthFilterConstructionParameters {
                    canonicalization_mask: info.orientation_canonicalization_mask(),
                    canonicalization_generator_moves: move_list_from_vec(vec!["x", "y"]),
                    max_canonicalizing_move_count_below: MoveCount(4),
                    solved_pattern: info.kpuzzle.default_pattern(),
                    depth_filtering_generator_moves: info.depth_filtering_generator_moves(),
                    min_optimal_solution_move_count: BIG_CUBES_MINIMUM_OPTIMAL_SOLUTION_MOVE_COUNT,
                },
            )
            .unwrap();
        let generators = SearchGenerators::try_new(
            &info.kpuzzle,
            info.scrambling_generator_moves(),
            &MetricEnum::Hand,
            false,
        )
        .unwrap();
        let canonical_fsm =
            CanonicalFSM::try_new(info.kpuzzle.clone(), generators.clone(), Default::default())
                .unwrap();
        Self {
            info,
            canonicalizing_solved_kpattern_depth_filter,
            generators,
            canonical_fsm,
            phantom_data: PhantomData,
        }
    }
}

pub enum BigCubeScrambleFinderSuffixConstraints {
    None,
    ForNoInspection,
}

pub struct BigCubeScrambleFinderScrambleOptions {
    pub suffix_constraints: BigCubeScrambleFinderSuffixConstraints,
}

impl<TBigCube: BigCube> ScrambleFinder for BigCubeScrambleFinder<TBigCube> {
    type TPuzzle = KPuzzle;
    type ScrambleOptions = BigCubeScrambleFinderScrambleOptions;

    fn filter_pattern(
        &mut self,
        pattern: &KPattern,
        _scramble_options: &Self::ScrambleOptions,
    ) -> FilteringDecision {
        self.canonicalizing_solved_kpattern_depth_filter
            .depth_filter(pattern)
            .unwrap() // TODO: avoid `.unwrap()`
    }
}

impl<TBigCube: BigCube> RandomMoveScrambleFinder for BigCubeScrambleFinder<TBigCube> {
    fn generate_unfiltered_random_move_scramble(
        &mut self,
        scramble_options: &Self::ScrambleOptions,
    ) -> Alg {
        // TODO: globally cache generators and `canonical_fsm` for each puzzle.
        let mut current_fsm_state = CANONICAL_FSM_START_STATE;
        let mut rng = thread_rng();
        let mut nodes = Vec::<AlgNode>::default();
        for _ in 0..self.info.num_random_moves().0 {
            // TODO: we can forward-cache the valid move classes for each state instead of rejection sampling.
            loop {
                let move_class_index =
                    MoveClassIndex(rng.gen_range(0..self.generators.by_move_class.len()));
                let next = self
                    .canonical_fsm
                    .next_state(current_fsm_state, move_class_index);
                if let Some(next) = next {
                    nodes.push(AlgNode::MoveNode(
                        self.generators.by_move_class[move_class_index]
                            .choose(&mut rng)
                            .unwrap()
                            .r#move
                            .clone(),
                    ));
                    current_fsm_state = next;
                    break;
                };
            }
        }

        let mut alg = Alg { nodes };

        match scramble_options.suffix_constraints {
            BigCubeScrambleFinderSuffixConstraints::None => {
                // no-op
            }
            BigCubeScrambleFinderSuffixConstraints::ForNoInspection => {
                if self.info.size.is_odd() {
                    alg = add_random_suffixes_from(alg, &self.info.no_inspection_suffixes_from())
                }
            }
        }

        alg
    }

    fn puzzle(&self) -> &Self::TPuzzle {
        &self.info.kpuzzle
    }
}

#[derive(Default)]
pub struct Cube5x5x5 {}

impl BigCube for Cube5x5x5 {
    fn info() -> BigCubeScrambleInfo {
        BigCubeScrambleInfo {
            kpuzzle: cube5x5x5_kpuzzle().clone(),
            size: 5,
        }
    }
}

pub type Cube5x5x5ScrambleFinder = BigCubeScrambleFinder<Cube5x5x5>;

#[derive(Default)]
pub struct Cube6x6x6 {}

impl BigCube for Cube6x6x6 {
    fn info() -> BigCubeScrambleInfo {
        BigCubeScrambleInfo {
            kpuzzle: cube6x6x6_kpuzzle().clone(),
            size: 6,
        }
    }
}

pub type Cube6x6x6ScrambleFinder = BigCubeScrambleFinder<Cube6x6x6>;

#[derive(Default)]
pub struct Cube7x7x7 {}

impl BigCube for Cube7x7x7 {
    fn info() -> BigCubeScrambleInfo {
        BigCubeScrambleInfo {
            kpuzzle: cube7x7x7_kpuzzle().clone(),
            size: 7,
        }
    }
}

pub type Cube7x7x7ScrambleFinder = BigCubeScrambleFinder<Cube7x7x7>;

#[cfg(test)]
mod tests {
    use crate::scramble::{
        puzzles::{
            big_cubes::{
                BigCubeScrambleFinderScrambleOptions, BigCubeScrambleFinderSuffixConstraints,
                Cube5x5x5ScrambleFinder, Cube6x6x6ScrambleFinder, Cube7x7x7ScrambleFinder,
            },
            definitions::{cube5x5x5_kpuzzle, cube6x6x6_kpuzzle, cube7x7x7_kpuzzle},
        },
        scramble_finder::scramble_finder::ScrambleFinder,
    };
    use cubing::alg::{parse_alg, Alg};

    use crate::_internal::search::move_count::MoveCount;

    #[test]
    fn num_moves() -> Result<(), String> {
        assert_eq!(
            Cube5x5x5ScrambleFinder::default().info.num_random_moves(),
            MoveCount(60)
        );
        Ok(())
    }

    #[test]
    // TODO: generalize and automate this across all events.
    fn simple_scramble_filtering_test_5x5x5() -> Result<(), String> {
        let mut scramble_finder = Cube5x5x5ScrambleFinder::default();
        let pattern = |alg: &Alg| {
            cube5x5x5_kpuzzle()
                .default_pattern()
                .apply_alg(alg)
                .unwrap()
        };
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("z")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("x y x")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("Lw z Uw")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("Lw z Uw' R")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("Rw z' Uw")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("R U")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_accept());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("Rw L")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_accept());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("U L F R B D")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_accept());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("U F 3Rw 3Uw2")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_accept());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("3Rw Lw")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::ForNoInspection
                },
            )
            .is_reject());
        Ok(())
    }

    #[test]
    // TODO: generalize and automate this across all events.
    fn simple_scramble_filtering_test_6x6x6() -> Result<(), String> {
        let mut scramble_finder = Cube6x6x6ScrambleFinder::default();
        let pattern = |alg: &Alg| {
            cube6x6x6_kpuzzle()
                .default_pattern()
                .apply_alg(alg)
                .unwrap()
        };
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("z")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("x y x")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("Lw z Uw")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("Lw z Uw' R")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("Rw z' Uw")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("R U")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_accept());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("Rw L")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_accept());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("U L F R B D")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_accept());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("3Rw 3Lw")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::ForNoInspection
                },
            )
            .is_reject());
        Ok(())
    }

    #[test]
    // TODO: generalize and automate this across all events.
    fn simple_scramble_filtering_test_7x7x7() -> Result<(), String> {
        let mut scramble_finder = Cube7x7x7ScrambleFinder::default();
        let pattern = |alg: &Alg| {
            cube7x7x7_kpuzzle()
                .default_pattern()
                .apply_alg(alg)
                .unwrap()
        };
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("z")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("x y x")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("Lw z Uw")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("Lw z Uw' R")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("Rw z' Uw")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_reject());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("R U")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_accept());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("Rw L")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_accept());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("U L F R B D")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None
                },
            )
            .is_accept());
        assert!(scramble_finder
            .filter_pattern(
                &pattern(parse_alg!("3Rw 4Lw")),
                &BigCubeScrambleFinderScrambleOptions {
                    suffix_constraints: BigCubeScrambleFinderSuffixConstraints::ForNoInspection
                },
            )
            .is_reject());
        Ok(())
    }
}
