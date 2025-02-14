use std::sync::Mutex;

use cubing::{
    alg::{parse_move, Alg, AlgNode, Move, QuantumMove},
    kpuzzle::{KPattern, KPuzzle},
};
use lazy_static::lazy_static;

use crate::{
    _internal::search::{
        idf_search::idf_search::{
            IDFSearch, IDFSearchConstructionOptions, IndividualSearchOptions,
        },
        mask_pattern::apply_mask,
        move_count::MoveCount,
    },
    scramble::{
        collapse::collapse_adjacent_moves,
        randomize::{basic_parity, BasicParity, OrbitRandomizationConstraints},
        scramble_search::{move_list_from_vec, FilteredSearch},
        solving_based_scramble_finder::{
            generate_fair_scramble, FilteringDecision, SolvingBasedScrambleFinder,
        },
    },
};

use super::{
    super::randomize::{
        randomize_orbit_naïve, OrbitOrientationConstraint, OrbitPermutationConstraint,
    },
    definitions::{cube3x3x3_centerless_g1_target_kpattern, cube3x3x3_centerless_kpuzzle},
    static_move_list::{add_random_suffixes_from, static_parsed_list, static_parsed_opt_list},
};

pub(crate) struct TwoPhase3x3x3Scramble {
    kpuzzle: KPuzzle,

    filtered_search: FilteredSearch<KPuzzle>,

    phase1_target_pattern: KPattern,
    phase1_idfs: IDFSearch<KPuzzle>,

    phase2_idfs: IDFSearch<KPuzzle>,
}

pub(crate) struct TwoPhase3x3x3ScrambleOptions {
    prefix_or_suffix_constraints: PrefixOrSuffixConstraints,
}

enum TwoPhase3x3x3ScrambleAssociatedAffixes {
    None,
    ForFMC,
    ForBLD(Alg),
}

pub(crate) struct TwoPhase3x3x3ScrambleAssociatedData {
    affixes: TwoPhase3x3x3ScrambleAssociatedAffixes,
}

impl SolvingBasedScrambleFinder for TwoPhase3x3x3Scramble {
    type TPuzzle = KPuzzle;
    type ScrambleAssociatedData = TwoPhase3x3x3ScrambleAssociatedData;
    type ScrambleOptions = TwoPhase3x3x3ScrambleOptions;

    fn generate_fair_unfiltered_random_pattern(
        &mut self,
        scramble_options: &TwoPhase3x3x3ScrambleOptions,
    ) -> (KPattern, TwoPhase3x3x3ScrambleAssociatedData) {
        let kpuzzle = cube3x3x3_centerless_kpuzzle();
        let mut scramble_pattern = kpuzzle.default_pattern();
        let edge_order = randomize_orbit_naïve(
            &mut scramble_pattern,
            0,
            "EDGES",
            OrbitRandomizationConstraints {
                orientation: Some(OrbitOrientationConstraint::SumToZero),
                ..Default::default()
            },
        );
        let each_orbit_parity = basic_parity(&edge_order);
        randomize_orbit_naïve(
            &mut scramble_pattern,
            1,
            "CORNERS",
            OrbitRandomizationConstraints {
                permutation: Some(match each_orbit_parity {
                    BasicParity::Even => OrbitPermutationConstraint::EvenParity,
                    BasicParity::Odd => OrbitPermutationConstraint::OddParity,
                }),
                orientation: Some(OrbitOrientationConstraint::SumToZero),
                ..Default::default()
            },
        );

        let (scramble_pattern, affixes) = match scramble_options.prefix_or_suffix_constraints {
            PrefixOrSuffixConstraints::None => (
                scramble_pattern,
                TwoPhase3x3x3ScrambleAssociatedAffixes::None,
            ),
            PrefixOrSuffixConstraints::ForFMC => (
                scramble_pattern,
                TwoPhase3x3x3ScrambleAssociatedAffixes::ForFMC,
            ),
            PrefixOrSuffixConstraints::ForBLD => {
                // TODO: randomize centers directly?
                let suffix = random_suffix_for_bld();
                (
                    scramble_pattern.apply_alg(&suffix).unwrap(),
                    TwoPhase3x3x3ScrambleAssociatedAffixes::ForBLD(suffix),
                )
            }
        };
        (
            scramble_pattern,
            TwoPhase3x3x3ScrambleAssociatedData { affixes },
        )
    }

    fn filter_pattern(
        &mut self,
        pattern: &KPattern,
        scramble_associated_data: &TwoPhase3x3x3ScrambleAssociatedData,
        _scramble_options: &TwoPhase3x3x3ScrambleOptions, // TODO: check that this matches the associated data?
    ) -> FilteringDecision {
        // TODO: Figure out how to avoid cloning `pattern`.
        let pattern_to_filter = match &scramble_associated_data.affixes {
            TwoPhase3x3x3ScrambleAssociatedAffixes::None => pattern.clone(),
            TwoPhase3x3x3ScrambleAssociatedAffixes::ForFMC => pattern.clone(),
            TwoPhase3x3x3ScrambleAssociatedAffixes::ForBLD(alg) => {
                // TODO: make the code below less custom to 3x3x3.
                let mut pattern = pattern.clone();
                for node in alg.nodes.iter().rev() {
                    let AlgNode::MoveNode(r#move) = node else {
                        panic!("Invalid BLD orientation alg");
                    };

                    let family = match r#move.quantum.family.as_ref() {
                        "Rw" => "x",
                        "Uw" => "y",
                        "Fw" => "z",
                        _ => panic!("Invalid BLD orientation alg move"),
                    }
                    .to_owned();
                    let inverse_move = Move {
                        quantum: QuantumMove {
                            family,
                            prefix: r#move.quantum.prefix.clone(),
                        }
                        .into(),
                        amount: r#move.amount,
                    };
                    pattern = pattern.apply_move(&inverse_move).unwrap();
                }
                pattern
            }
        };
        dbg!(&pattern_to_filter);
        self.filtered_search
            .filtering_decision(&pattern_to_filter, MoveCount(2))
    }

    fn solve_pattern(
        &mut self,
        pattern: &KPattern,
        scramble_associated_data: &TwoPhase3x3x3ScrambleAssociatedData,
        _scramble_options: &TwoPhase3x3x3ScrambleOptions,
    ) -> Alg {
        // TODO: we pass premoves and postmoves to both phases in case the other
        // turns out to have an empty alg solution. We can handle this better by making
        // a way to bridge the FSM between phases.
        let (
            search_pattern,
            canonical_fsm_pre_moves_phase1,
            canonical_fsm_post_moves_phase1,
            canonical_fsm_pre_moves_phase2,
            canonical_fsm_post_moves_phase2,
        ) = match &scramble_associated_data.affixes {
            TwoPhase3x3x3ScrambleAssociatedAffixes::None => (pattern, None, None, None, None),
            TwoPhase3x3x3ScrambleAssociatedAffixes::ForFMC => {
                // For the pre-moves, we don't have to specify R' and U' because we know the FSM only depends on the final `F` move.
                // For similar reasons, we only have to specify R' for the post-moves.
                (
                    pattern,
                    Some(vec![parse_move!("F").to_owned()]),
                    Some(vec![parse_move!("R'").to_owned()]),
                    // TODO: support a way to specify a quantum factor
                    Some(vec![parse_move!("F2'").to_owned()]),
                    Some(vec![parse_move!("R2'").to_owned()]),
                )
            }
            TwoPhase3x3x3ScrambleAssociatedAffixes::ForBLD(alg) => (
                &pattern.apply_alg(&alg.invert()).unwrap(),
                None,
                None,
                None,
                None,
            ),
        };

        let phase1_alg = {
            let phase1_search_pattern =
                apply_mask(search_pattern, &self.phase1_target_pattern).unwrap();
            self.phase1_idfs
                .search(
                    &phase1_search_pattern,
                    IndividualSearchOptions {
                        min_num_solutions: Some(1),
                        canonical_fsm_pre_moves: canonical_fsm_pre_moves_phase1,
                        canonical_fsm_post_moves: canonical_fsm_post_moves_phase1,
                        ..Default::default()
                    },
                )
                .next()
                .unwrap()
        };

        let mut phase2_alg = {
            let phase2_search_pattern = search_pattern
                .apply_transformation(&self.kpuzzle.transformation_from_alg(&phase1_alg).unwrap());
            self.phase2_idfs
                .search(
                    &phase2_search_pattern,
                    IndividualSearchOptions {
                        min_num_solutions: Some(1),
                        canonical_fsm_pre_moves: canonical_fsm_pre_moves_phase2,
                        canonical_fsm_post_moves: canonical_fsm_post_moves_phase2,
                        ..Default::default()
                    },
                )
                .next()
                .unwrap()
        };

        let mut nodes = phase1_alg.nodes;
        nodes.append(&mut phase2_alg.nodes);
        Alg { nodes }
    }

    fn collapse_inverted_alg(&mut self, alg: Alg) -> Alg {
        collapse_adjacent_moves(alg, 4, -1)
    }
}

impl Default for TwoPhase3x3x3Scramble {
    fn default() -> Self {
        let kpuzzle = cube3x3x3_centerless_kpuzzle().clone();
        let generators = move_list_from_vec(vec!["U", "L", "F", "R", "B", "D"]);
        let filtered_search = FilteredSearch::new(
            IDFSearch::try_new(
                kpuzzle.clone(),
                generators.clone(),
                kpuzzle.default_pattern(),
                IDFSearchConstructionOptions {
                    min_prune_table_size: Some(32),
                    ..Default::default()
                },
            )
            .unwrap(),
        );

        let phase1_target_pattern = cube3x3x3_centerless_g1_target_kpattern().clone();
        let phase1_idfs = IDFSearch::try_new(
            kpuzzle.clone(),
            generators.clone(),
            phase1_target_pattern.clone(),
            IDFSearchConstructionOptions {
                min_prune_table_size: Some(32),
                ..Default::default()
            },
        )
        .unwrap();

        let phase2_generators = move_list_from_vec(vec!["U", "L2", "F2", "R2", "B2", "D"]);
        let phase2_idfs = IDFSearch::try_new(
            kpuzzle.clone(),
            phase2_generators.clone(),
            kpuzzle.default_pattern(),
            IDFSearchConstructionOptions {
                min_prune_table_size: Some(1 << 24),
                ..Default::default()
            },
        )
        .unwrap();

        Self {
            kpuzzle,
            filtered_search,

            phase1_target_pattern,
            phase1_idfs,

            phase2_idfs,
        }
    }
}

fn random_suffix_for_bld() -> Alg {
    let s1 = static_parsed_opt_list(&["", "Rw", "Rw2", "Rw'", "Fw", "Fw'"]);
    let s2 = static_parsed_opt_list(&["", "Uw", "Uw2", "Uw'"]);
    add_random_suffixes_from(Alg::default(), [s1, s2])
}

pub(crate) enum PrefixOrSuffixConstraints {
    None,
    ForFMC,
    ForBLD,
}

pub(crate) fn scramble_3x3x3() -> Alg {
    generate_fair_scramble::<TwoPhase3x3x3Scramble>(&TwoPhase3x3x3ScrambleOptions {
        prefix_or_suffix_constraints: PrefixOrSuffixConstraints::None,
    })
}

pub(crate) fn scramble_3x3x3_bld() -> Alg {
    generate_fair_scramble::<TwoPhase3x3x3Scramble>(&TwoPhase3x3x3ScrambleOptions {
        prefix_or_suffix_constraints: PrefixOrSuffixConstraints::ForBLD,
    })
}

const FMC_AFFIX: [&str; 3] = ["R'", "U'", "F"];

pub(crate) fn scramble_3x3x3_fmc() -> Alg {
    generate_fair_scramble::<TwoPhase3x3x3Scramble>(&TwoPhase3x3x3ScrambleOptions {
        prefix_or_suffix_constraints: PrefixOrSuffixConstraints::ForFMC,
    })
}
