use std::sync::LazyLock;

use cubing::{
    alg::{parse_alg, parse_move, Alg, AlgNode, Move, QuantumMove},
    kpuzzle::{KPattern, KPuzzle, KTransformation},
    puzzles::cube3x3x3_kpuzzle,
};

use crate::{
    _internal::{
        errors::SearchError,
        search::{
            filter::filtering_decision::FilteringDecision,
            iterative_deepening::iterative_deepening_search::{
                IndividualSearchOptions, IterativeDeepeningSearch,
                IterativeDeepeningSearchConstructionOptions,
            },
            mask_pattern::apply_mask,
            move_count::MoveCount,
        },
    },
    scramble::{
        collapse::collapse_adjacent_moves,
        randomize::{
            basic_parity, randomize_orbit_naïve, BasicParity, OrbitOrientationConstraint,
            OrbitPermutationConstraint, OrbitRandomizationConstraints,
        },
        scramble_search::{move_list_from_vec, FilteredSearch},
        solving_based_scramble_finder::SolvingBasedScrambleFinder,
    },
};

use super::{
    definitions::cube3x3x3_g1_target_kpattern,
    static_move_list::{add_random_suffixes_from, static_parsed_opt_list},
};

// static FMC_AFFIX: [&str; 3] = ["R'", "U'", "F"];
static FMC_AFFIX_ALG: LazyLock<Alg> = LazyLock::new(|| parse_alg!("R' U' F").clone());

pub(crate) struct TwoPhase3x3x3ScrambleFinder {
    kpuzzle: KPuzzle,

    filtered_search: FilteredSearch<KPuzzle>,

    phase1_target_pattern: KPattern,
    phase1_iterative_deepening_search: IterativeDeepeningSearch<KPuzzle>,

    phase2_iterative_deepening_search: IterativeDeepeningSearch<KPuzzle>,
}

pub(crate) struct TwoPhase3x3x3ScrambleOptions {
    pub(crate) prefix_or_suffix_constraints: TwoPhase3x3x3PrefixOrSuffixConstraints,
}

pub(crate) enum TwoPhase3x3x3ScrambleAssociatedAffixes {
    None,
    ForFMC,
    ForBLD(Alg),
}

// TODO: validation
fn kpattern_to_transformation(kpattern: &KPattern) -> Option<KTransformation> {
    let mut transformation = kpattern.kpuzzle().identity_transformation();
    for orbit_info in kpattern.kpuzzle().orbit_info_iter() {
        for i in 0..orbit_info.num_pieces {
            transformation.set_permutation_idx(orbit_info, i, kpattern.get_piece(orbit_info, i));
            let orientation_with_mod = kpattern.get_orientation_with_mod(orbit_info, i);
            // TODO
            if orientation_with_mod.orientation_mod != 0
                && orientation_with_mod.orientation_mod != 1
            {
                return None;
            }
            transformation.set_orientation_delta(orbit_info, i, orientation_with_mod.orientation);
        }
    }
    Some(transformation)
}

fn apply_pre_alg(kpattern: &KPattern, alg: &Alg) -> Option<KPattern> {
    let pattern_transformation = kpattern_to_transformation(kpattern)?;
    let Ok(alg_pattern) = kpattern.kpuzzle().default_pattern().apply_alg(alg) else {
        return None;
    };
    Some(alg_pattern.apply_transformation(&pattern_transformation))
}

pub(crate) struct TwoPhase3x3x3ScrambleAssociatedData {
    pub(crate) affixes: TwoPhase3x3x3ScrambleAssociatedAffixes,
}

impl SolvingBasedScrambleFinder for TwoPhase3x3x3ScrambleFinder {
    type TPuzzle = KPuzzle;
    type ScrambleAssociatedData = TwoPhase3x3x3ScrambleAssociatedData;
    type ScrambleOptions = TwoPhase3x3x3ScrambleOptions;

    fn generate_fair_unfiltered_random_pattern(
        &mut self,
        scramble_options: &TwoPhase3x3x3ScrambleOptions,
    ) -> (KPattern, TwoPhase3x3x3ScrambleAssociatedData) {
        let kpuzzle = cube3x3x3_kpuzzle();
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
            TwoPhase3x3x3PrefixOrSuffixConstraints::None => (
                scramble_pattern,
                TwoPhase3x3x3ScrambleAssociatedAffixes::None,
            ),
            TwoPhase3x3x3PrefixOrSuffixConstraints::ForFMC => (
                scramble_pattern,
                TwoPhase3x3x3ScrambleAssociatedAffixes::ForFMC,
            ),
            TwoPhase3x3x3PrefixOrSuffixConstraints::ForBLD => {
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
                    pattern = pattern.apply_move(&inverse_move).unwrap(); // TODO
                }
                pattern
            }
        };
        // dbg!(&pattern_to_filter);
        self.filtered_search
            .filtering_decision(&pattern_to_filter, MoveCount(2))
    }

    // TODO: handle all `unwrap()`s.
    fn solve_pattern(
        &mut self,
        pattern: &KPattern,
        scramble_associated_data: &TwoPhase3x3x3ScrambleAssociatedData,
        _scramble_options: &TwoPhase3x3x3ScrambleOptions,
    ) -> Result<Alg, SearchError> {
        // TODO: we pass premoves and postmoves to both phases in case the other
        // turns out to have an empty alg solution. We can handle this better by making
        // a way to bridge the FSM between phases.
        let (
            pre_alg,
            search_pattern,
            post_alg,
            canonical_fsm_pre_moves_phase1,
            canonical_fsm_post_moves_phase1,
            canonical_fsm_pre_moves_phase2,
            canonical_fsm_post_moves_phase2,
        ) = match &scramble_associated_data.affixes {
            TwoPhase3x3x3ScrambleAssociatedAffixes::None => (
                Alg::default(),
                pattern.clone(),
                Alg::default(),
                None,
                None,
                None,
                None,
            ),
            TwoPhase3x3x3ScrambleAssociatedAffixes::ForFMC => {
                let fmc_affix_alg = FMC_AFFIX_ALG.clone();
                let a = fmc_affix_alg.invert();
                let search_pattern = apply_pre_alg(pattern, &(a))
                    .unwrap()
                    .apply_alg(&fmc_affix_alg.invert())
                    .unwrap();
                // For the pre-moves, we don't have to specify R' and U' because we know the FSM only depends on the final `F` move.
                // For similar reasons, we only have to specify R' for the post-moves.
                (
                    (fmc_affix_alg.clone()),
                    search_pattern,
                    (fmc_affix_alg.clone()),
                    Some(vec![parse_move!("F").to_owned()]),
                    Some(vec![parse_move!("R'").to_owned()]),
                    // TODO: support a way to specify a quantum factor
                    Some(vec![parse_move!("F2'").to_owned()]),
                    Some(vec![parse_move!("R2'").to_owned()]),
                )
            }
            TwoPhase3x3x3ScrambleAssociatedAffixes::ForBLD(alg) => (
                alg.clone().invert(),
                // apply_pre_alg(pattern, &alg.invert()).unwrap(),
                pattern.apply_alg(&alg.invert()).unwrap(),
                Alg::default(),
                None,
                None,
                None,
                None,
            ),
        };

        // dbg!(&search_pattern);

        let phase1_search_pattern =
            apply_mask(&search_pattern, &self.phase1_target_pattern).unwrap();
        let Some(mut phase1_alg) = self
            .phase1_iterative_deepening_search
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
        else {
            return Err(SearchError {
                description: "Could not find a solution".to_owned(),
            });
        };

        let mut phase2_alg = {
            let phase2_search_pattern = search_pattern
                .apply_transformation(&self.kpuzzle.transformation_from_alg(&phase1_alg).unwrap());
            self.phase2_iterative_deepening_search
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

        let mut nodes = pre_alg.invert().nodes;
        nodes.append(&mut phase1_alg.nodes);
        nodes.append(&mut phase2_alg.nodes);
        nodes.append(&mut post_alg.invert().nodes);
        Ok(Alg { nodes })
    }

    fn collapse_inverted_alg(&mut self, alg: Alg) -> Alg {
        collapse_adjacent_moves(alg, 4, -1)
    }
}

impl Default for TwoPhase3x3x3ScrambleFinder {
    fn default() -> Self {
        // TODO: add centerless optimizations where possible?
        let kpuzzle = cube3x3x3_kpuzzle().clone();
        let generators = move_list_from_vec(vec!["U", "L", "F", "R", "B", "D"]);
        let filtered_search = FilteredSearch::new(
            IterativeDeepeningSearch::try_new_kpuzzle_with_hash_prune_table_shim(
                kpuzzle.clone(),
                generators.clone(),
                vec![kpuzzle.default_pattern()],
                IterativeDeepeningSearchConstructionOptions {
                    min_prune_table_size: Some(32),
                    ..Default::default()
                },
                None,
            )
            .unwrap(),
        );

        let phase1_target_pattern = cube3x3x3_g1_target_kpattern().clone();
        let phase1_iterative_deepening_search =
            IterativeDeepeningSearch::try_new_kpuzzle_with_hash_prune_table_shim(
                kpuzzle.clone(),
                generators.clone(),
                vec![phase1_target_pattern.clone()],
                IterativeDeepeningSearchConstructionOptions {
                    min_prune_table_size: Some(32),
                    ..Default::default()
                },
                None,
            )
            .unwrap();

        let phase2_generators = move_list_from_vec(vec!["U", "L2", "F2", "R2", "B2", "D"]);
        let phase2_iterative_deepening_search =
            IterativeDeepeningSearch::try_new_kpuzzle_with_hash_prune_table_shim(
                kpuzzle.clone(),
                phase2_generators.clone(),
                vec![kpuzzle.default_pattern()],
                IterativeDeepeningSearchConstructionOptions {
                    min_prune_table_size: Some(1 << 24),
                    ..Default::default()
                },
                None,
            )
            .unwrap();

        Self {
            kpuzzle,
            filtered_search,

            phase1_target_pattern,
            phase1_iterative_deepening_search,

            phase2_iterative_deepening_search,
        }
    }
}

fn random_suffix_for_bld() -> Alg {
    let s1 = static_parsed_opt_list(&["", "Rw", "Rw2", "Rw'", "Fw", "Fw'"]);
    let s2 = static_parsed_opt_list(&["", "Uw", "Uw2", "Uw'"]);
    add_random_suffixes_from(Alg::default(), [s1, s2])
}

pub(crate) enum TwoPhase3x3x3PrefixOrSuffixConstraints {
    None,
    ForFMC,
    ForBLD,
}
