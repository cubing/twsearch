use std::{hash::Hasher, sync::Arc};

use cubing::{
    alg::parse_alg,
    kpuzzle::{KPattern, KPuzzle},
};

use crate::{
    _internal::{
        puzzle_traits::puzzle_traits::HashablePatternPuzzle,
        search::{
            coordinates::{
                masked_kpuzzle_deriver::MaskedPuzzleDeriver, pattern_deriver::PatternDeriver,
                unenumerated_derived_pattern_puzzle::UnenumeratedDerivedPatternPuzzle,
            },
            hash_prune_table::HashPruneTable,
            iterative_deepening::iterative_deepening_search::{
                IterativeDeepeningSearch, IterativeDeepeningSearchConstructionOptions,
            },
            search_logger::SearchLogger,
        },
    },
    experimental_lib_api::{
        derived_puzzle_search_phase::DerivedPuzzleSearchPhase, CompoundDerivedPuzzle,
        CompoundPuzzle,
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

    let from = orbit.orientations_offset;
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
        pattern.set_piece(
            orbit,
            0,
            match parity {
                BasicParity::Even => 0,
                BasicParity::Odd => 1,
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

pub(crate) fn phase2_search(
    search_logger: Arc<SearchLogger>,
) -> DerivedPuzzleSearchPhase<KPuzzle, Cube4x4x4Phase2Puzzle> {
    let phase2_generator_moves =
        move_list_from_vec(vec!["Uw2", "U", "L", "Fw2", "F", "Rw", "R", "B", "D"]);

    // This would be inline, but we need to work around https://github.com/cubing/twsearch/issues/128
    let phase2_name =
        "Place F/B and U/D centers on correct axes and make L/R solvable with half turns"
            .to_owned();

    let cube4x4x4_phase2_puzzle = Cube4x4x4Phase2Puzzle::default();
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
        // TODO: figure out a way to derive before alg application?
        cube4x4x4_phase2_puzzle
            .derive_pattern(
                &cube4x4x4_phase2_centers_target_kpattern()
                    .apply_alg(alg)
                    .unwrap(),
            )
            .unwrap()
    });

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
    DerivedPuzzleSearchPhase::new(
        phase2_name,
        cube4x4x4_phase2_puzzle,
        phase2_iterative_deepening_search,
        Default::default(),
    )
}
