use std::sync::Arc;

use cubing::{
    alg::{parse_alg, Alg},
    kpuzzle::{KPattern, KPuzzle, KPuzzleOrbitInfo, OrientationWithMod},
};

use crate::{
    _internal::{
        errors::SearchError,
        puzzle_traits::puzzle_traits::HashablePatternPuzzle,
        search::{
            coordinates::{
                pattern_deriver::PatternDeriver,
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
            mask_pattern::apply_mask,
            search_logger::SearchLogger,
        },
    },
    experimental_lib_api::{derived_puzzle_search_phase::DerivedPuzzleSearchPhase, SearchPhase},
    scramble::{
        parity::{basic_parity, BasicParity},
        puzzles::{
            cube4x4x4::wings::{NUM_WINGS, POSITION_IS_PRIMARY, WING_TO_PRIMARY_WING_IN_DEDGE},
            definitions::{cube4x4x4_kpuzzle, cube4x4x4_phase2_search_kpuzzle},
        },
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

pub fn transfer_orbit(
    source_pattern: &KPattern,
    source_orbit_info: &KPuzzleOrbitInfo,
    destination_pattern: &mut KPattern,
    destination_orbit_info: &KPuzzleOrbitInfo,
) {
    assert_eq!(
        source_orbit_info.num_pieces,
        destination_orbit_info.num_pieces
    );
    assert_eq!(
        source_orbit_info.num_orientations,
        destination_orbit_info.num_orientations
    );
    for i in 0..source_orbit_info.num_pieces {
        destination_pattern.set_piece(
            destination_orbit_info,
            i,
            source_pattern.get_piece(source_orbit_info, i),
        );
        destination_pattern.set_orientation_with_mod(
            destination_orbit_info,
            i,
            source_pattern.get_orientation_with_mod(source_orbit_info, i),
        );
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Cube4x4x4Phase2PatternDeriver {}

impl PatternDeriver<KPuzzle> for Cube4x4x4Phase2PatternDeriver {
    type DerivedPattern = KPattern;

    fn derive_pattern(&self, source_puzzle_pattern: &KPattern) -> Option<Self::DerivedPattern> {
        // TODO: optimize this
        let kpuzzle = cube4x4x4_phase2_search_kpuzzle(); // TODO: cache on self?
        let mut pattern = kpuzzle.default_pattern();
        {
            let source_centers_orbit_info = &cube4x4x4_kpuzzle().data.ordered_orbit_info[2];
            assert_eq!(source_centers_orbit_info.name.0, "CENTERS");
            let destination_centers_orbit_info = &kpuzzle.data.ordered_orbit_info[0];
            assert_eq!(destination_centers_orbit_info.name.0, "CENTERS");
            transfer_orbit(
                source_puzzle_pattern,
                source_centers_orbit_info,
                &mut pattern,
                destination_centers_orbit_info,
            );
        }
        {
            let orbit_info = &kpuzzle.data.ordered_orbit_info[1];
            assert_eq!(orbit_info.name.0, "WING_PARITY");
            let parity = basic_parity(wing_permutation_slice(source_puzzle_pattern));

            pattern.set_orientation_with_mod(
                orbit_info,
                0,
                &OrientationWithMod::new_using_default_orientation_mod(parity.to_0_or_1()),
            );
        }

        Some(apply_mask(&pattern, &kpuzzle.default_pattern()).unwrap())
    }
}

pub(crate) type Cube4x4x4Phase2Puzzle =
    UnenumeratedDerivedPatternPuzzle<KPuzzle, KPuzzle, Cube4x4x4Phase2PatternDeriver>;

impl Default for Cube4x4x4Phase2Puzzle {
    fn default() -> Self {
        let pattern_deriver = Cube4x4x4Phase2PatternDeriver {};

        UnenumeratedDerivedPatternPuzzle {
            source_puzzle: cube4x4x4_kpuzzle().clone(),
            derived_puzzle: cube4x4x4_phase2_search_kpuzzle().clone(),
            pattern_deriver,
        }
    }
}

impl HashablePatternPuzzle for Cube4x4x4Phase2Puzzle {
    fn pattern_hash_u64(&self, pattern: &Self::Pattern) -> u64 {
        self.derived_puzzle.pattern_hash_u64(pattern)
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
        let filter_search_solution_fn =
            move |_pattern: &KPattern, solution_moves: &SolutionMoves| -> FilteringDecision {
                let alg: Alg = Alg::from(solution_moves);
                let pattern = phase_search_pattern_owned.apply_alg(&alg).unwrap();
                // dbg!(&phase_search_pattern_owned);
                // dbg!(alg.to_string());
                // dbg!(&pattern);
                if is_each_wing_pair_separated_across_primary_and_secondary(&pattern) {
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

fn is_each_wing_pair_separated_across_primary_and_secondary(pattern: &KPattern) -> bool {
    let orbit_info = &pattern.kpuzzle().data.ordered_orbit_info[1];
    debug_assert_eq!(orbit_info.name.0, "WINGS");

    // TODO: There are some clever ways to optimize this. Are any of them worth it?
    let mut seen_in_primary_position = [false; NUM_WINGS as usize];
    let mut seen_in_secondary_position = [false; NUM_WINGS as usize];

    // println!("is_each_wing_pair_separated_across_low_high");
    let mut primary_pieces_in_primary_positions_parity = BasicParity::Even;
    for position in 0..NUM_WINGS {
        let piece = pattern.get_piece(orbit_info, position);
        let arr = if POSITION_IS_PRIMARY[position as usize] {
            if POSITION_IS_PRIMARY[piece as usize] {
                primary_pieces_in_primary_positions_parity.flip();
            }
            &mut seen_in_primary_position
        } else {
            &mut seen_in_secondary_position
        };
        let primary_piece = WING_TO_PRIMARY_WING_IN_DEDGE[piece as usize];
        if arr[primary_piece as usize] {
            return false;
        }
        arr[primary_piece as usize] = true;
    }
    primary_pieces_in_primary_positions_parity == BasicParity::Even
}
