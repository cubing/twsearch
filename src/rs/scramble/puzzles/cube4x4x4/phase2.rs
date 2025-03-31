use std::{hash::Hasher, sync::Arc};

use cubing::{
    alg::{parse_alg, Alg},
    kpuzzle::{KPattern, KPuzzle, OrientationWithMod},
};

use crate::{
    _internal::{
        errors::SearchError,
        puzzle_traits::puzzle_traits::HashablePatternPuzzle,
        search::{
            coordinates::{
                masked_kpuzzle_deriver::MaskedPuzzleDeriver, pattern_deriver::PatternDeriver,
                unenumerated_derived_pattern_puzzle::UnenumeratedDerivedPatternPuzzle,
            },
            filter::filtering_decision::FilteringDecision,
            hash_prune_table::HashPruneTable,
            iterative_deepening::{
                iterative_deepening_search::{
                    IterativeDeepeningSearch, IterativeDeepeningSearchConstructionOptions,
                    SolutionMoves,
                },
                search_adaptations::IndividualSearchAdaptations,
            },
            search_logger::SearchLogger,
        },
    },
    experimental_lib_api::{
        derived_puzzle_search_phase::DerivedPuzzleSearchPhase, CompoundDerivedPuzzle,
        CompoundPuzzle, SearchPhase,
    },
    scramble::{
        puzzles::definitions::{
            cube4x4x4_kpuzzle, cube4x4x4_phase2_centers_target_kpattern,
            cube4x4x4_phase2_wing_parity_kpuzzle,
        },
        randomize::{basic_parity, BasicParity},
        scramble_search::move_list_from_vec,
    },
};

// #[derive(Debug, Clone, Hash, PartialEq, Eq)]
// pub(crate) struct WingParityPattern {
//     pub(crate) parity: BasicParity,
// }

fn wing_permutation_slice(pattern: &KPattern) -> &[u8] {
    let orbit = &pattern.kpuzzle().data.ordered_orbit_info[1];
    assert_eq!(orbit.name.0, "WINGS");

    let from = orbit.pieces_or_permutations_offset;
    let to = from + (orbit.num_pieces as usize);

    let full_byte_slice = unsafe { pattern.byte_slice() };
    &full_byte_slice[from..to]
}

#[derive(Clone, Debug)]
pub(crate) struct WingParityPatternDeriver {}

impl PatternDeriver<KPuzzle> for WingParityPatternDeriver {
    type DerivedPattern = KPattern;

    fn derive_pattern(&self, source_puzzle_pattern: &KPattern) -> Option<Self::DerivedPattern> {
        // TODO: optimize this
        let kpuzzle = cube4x4x4_phase2_wing_parity_kpuzzle(); // TODO: cache on self?
        let mut pattern = kpuzzle.default_pattern();
        let orbit = &kpuzzle.data.ordered_orbit_info[0];
        let parity = basic_parity(wing_permutation_slice(source_puzzle_pattern));
        pattern.set_orientation_with_mod(
            orbit,
            0,
            &OrientationWithMod {
                orientation: match parity {
                    BasicParity::Even => 0,
                    BasicParity::Odd => 1,
                },
                orientation_mod: 0,
            },
        );
        Some(pattern)
    }
}

pub(crate) type WingParityPuzzle =
    UnenumeratedDerivedPatternPuzzle<KPuzzle, KPuzzle, WingParityPatternDeriver>;

pub(crate) type Cube4x4x4Phase2Puzzle = CompoundDerivedPuzzle<
    KPuzzle,
    UnenumeratedDerivedPatternPuzzle<KPuzzle, KPuzzle, MaskedPuzzleDeriver>,
    WingParityPuzzle,
>;

impl Default for Cube4x4x4Phase2Puzzle {
    fn default() -> Self {
        let kpuzzle = cube4x4x4_kpuzzle();

        let masked_centers_puzzle_deriver =
            MaskedPuzzleDeriver::new(cube4x4x4_phase2_centers_target_kpattern().clone());
        let masked_centers_derived_puzzle = UnenumeratedDerivedPatternPuzzle::new(
            kpuzzle.clone(),
            kpuzzle.clone(),
            masked_centers_puzzle_deriver,
        );

        let wing_parity_pattern_deriver = WingParityPatternDeriver {};
        let wing_parity_derived_puzzle = UnenumeratedDerivedPatternPuzzle::new(
            kpuzzle.clone(),
            cube4x4x4_phase2_wing_parity_kpuzzle().clone(),
            wing_parity_pattern_deriver,
        );

        let compound_puzzle: CompoundPuzzle<
            UnenumeratedDerivedPatternPuzzle<KPuzzle, KPuzzle, MaskedPuzzleDeriver>,
            WingParityPuzzle,
        > = CompoundPuzzle {
            tpuzzle0: masked_centers_derived_puzzle,
            tpuzzle1: wing_parity_derived_puzzle.clone(),
        };
        compound_puzzle.into()
    }
}

impl HashablePatternPuzzle for Cube4x4x4Phase2Puzzle {
    fn pattern_hash_u64(&self, pattern: &Self::Pattern) -> u64 {
        // TODO: derive this in a performant way.
        let mut city_hasher = cityhasher::CityHasher::new();
        let pattern0_bytes = self
            .compound_puzzle
            .tpuzzle0
            .source_puzzle
            .pattern_hash_u64(&pattern.0)
            .to_le_bytes();
        city_hasher.write(&pattern0_bytes);
        let pattern1_bytes = self
            .compound_puzzle
            .tpuzzle1
            .source_puzzle
            .pattern_hash_u64(&pattern.1)
            .to_le_bytes();
        city_hasher.write(&pattern1_bytes);
        city_hasher.finish()
    }
}

// TODO: We have to wrap `DerivedPuzzleSearchPhase<…>` in order to implement `pre_phase(…)`. There's probably a neat way around this.
pub(crate) struct Cube4x4x4Phase2Search {
    derived_puzzle_search_phase: DerivedPuzzleSearchPhase<KPuzzle, Cube4x4x4Phase2Puzzle>,
}

pub(crate) fn phase2_search(search_logger: Arc<SearchLogger>) -> Cube4x4x4Phase2Search {
    let phase2_generator_moves =
        move_list_from_vec(vec!["Uw2", "U", "L", "Fw", "F", "Rw2", "R", "B", "D"]);

    // This would be inline, but we need to work around https://github.com/cubing/twsearch/issues/128
    let phase2_name =
        "Place L/R and U/D centers on correct axes and make F/B solvable with half turns"
            .to_owned();

    let cube4x4x4_phase2_puzzle = Cube4x4x4Phase2Puzzle::default();
    let phase2_target_patterns = [
        parse_alg!(""),
        parse_alg!("y2"),
        parse_alg!("Lw2"),
        parse_alg!("Rw2"),
        parse_alg!("Uw2"),
        parse_alg!("Dw2"),
        parse_alg!("Lw2 Fw2"),
        parse_alg!("Rw2 Fw2"),
        parse_alg!("Uw2 Fw2"),
        parse_alg!("Dw2 Fw2"),
        parse_alg!("Dw2 Fw2 Lw2"),
        parse_alg!("Lw2 Fw2 Uw2"),
    ]
    .map(|alg| {
        // TODO: figure out a way to derive before alg application?
        cube4x4x4_phase2_puzzle
            .derive_pattern(
                &cube4x4x4_kpuzzle()
                    .default_pattern()
                    .apply_alg(alg)
                    .unwrap(),
            )
            .unwrap()
    });

    dbg!(&phase2_target_patterns);

    let phase2_iterative_deepening_search =
        IterativeDeepeningSearch::<Cube4x4x4Phase2Puzzle>::try_new_prune_table_construction_shim::<
            HashPruneTable<Cube4x4x4Phase2Puzzle>,
        >(
            cube4x4x4_phase2_puzzle.clone(),
            phase2_generator_moves,
            phase2_target_patterns.to_vec(),
            IterativeDeepeningSearchConstructionOptions {
                search_logger,
                ..Default::default()
            },
            None,
        )
        .unwrap();
    Cube4x4x4Phase2Search {
        derived_puzzle_search_phase: DerivedPuzzleSearchPhase::new(
            phase2_name,
            cube4x4x4_phase2_puzzle,
            phase2_iterative_deepening_search,
            Default::default(),
        ),
    }
}

impl SearchPhase<KPuzzle> for Cube4x4x4Phase2Search {
    fn phase_name(&self) -> &str {
        self.derived_puzzle_search_phase.phase_name()
    }

    fn first_solution(
        &mut self,
        phase_search_pattern: &KPattern,
    ) -> Result<Option<Alg>, SearchError> {
        let phase_search_pattern_owned = phase_search_pattern.clone();
        let filter_search_solution_fn = move |_pattern: &(KPattern, KPattern),
                                              solution_moves: &SolutionMoves|
              -> FilteringDecision {
            let alg: Alg = Alg::from(solution_moves);
            let pattern = phase_search_pattern_owned.apply_alg(&alg).unwrap();
            // dbg!(&phase_search_pattern_owned);
            // dbg!(alg.to_string());
            // dbg!(&pattern);
            if is_each_wing_pair_separated_across_low_high(&pattern) {
                FilteringDecision::Accept
            } else {
                FilteringDecision::Reject
            }
        };
        self.derived_puzzle_search_phase
            .first_solution_with_individual_search_adaptations(
                phase_search_pattern,
                IndividualSearchAdaptations {
                    filter_search_solution_fn: Some(Arc::new(filter_search_solution_fn)),
                },
            )
    }
}

const NUM_WINGS: u8 = 24;

// A low position is any wing position that can be reached from piece 0 (UBl) using <U, L, R, D>.
// A low piece is any piece that starts in a low position, and a high position/piece is any that is not a low position/piece.
// This lookup does the following:
//
// - Given a low position, it returns that same position.
// - Given a high position, it returns the low position that is part of the same dedge (double edge).
//
// Note that the low position of a dedge may or may not have the lower index. (TODO: use a different terminology like "primary"/"secondary"?)
const POSITION_TO_LOW_PIECE: [u8; NUM_WINGS as usize] = [
    0, 1, 2, 3, 3, 11, 23, 17, 2, 9, 20, 11, 1, 19, 21, 9, 0, 17, 22, 19, 20, 21, 22, 23,
];

const POSITION_IS_LOW: [bool; NUM_WINGS as usize] = [
    true, true, true, true, false, false, false, false, false, true, false, true, false, false,
    false, false, false, true, false, true, true, true, true, true,
];

fn is_each_wing_pair_separated_across_low_high(pattern: &KPattern) -> bool {
    let orbit_info = &pattern.kpuzzle().data.ordered_orbit_info[1];
    assert_eq!(orbit_info.name.0, "WINGS");

    // TODO: There are some clever ways to optimize this. Are any of them worth it?
    let mut seen_low = [false; NUM_WINGS as usize];
    let mut seen_high = [false; NUM_WINGS as usize];

    // println!("is_each_wing_pair_separated_across_low_high");
    for position in 0..NUM_WINGS {
        let piece = pattern.get_piece(orbit_info, position);
        let arr = if POSITION_IS_LOW[position as usize] {
            &mut seen_low
        } else {
            &mut seen_high
        };
        let low_piece = POSITION_TO_LOW_PIECE[piece as usize];
        if arr[low_piece as usize] {
            return false;
        }
        arr[low_piece as usize] = true;
    }
    true
}
