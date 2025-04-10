use cubing::{
    alg::Alg,
    kpuzzle::{KPattern, KPuzzle, KTransformation, OrientationWithMod},
};

use crate::{
    _internal::{
        errors::SearchError,
        puzzle_traits::puzzle_traits::{HashablePatternPuzzle, SemiGroupActionPuzzle},
        search::{
            coordinates::{
                masked_kpuzzle_deriver::MaskedKPuzzleDeriver,
                pattern_deriver::{DerivedPuzzle, PatternDeriver},
                unenumerated_derived_pattern_puzzle::UnenumeratedDerivedPatternPuzzle,
            },
            hash_prune_table::HashPruneTable,
            iterative_deepening::iterative_deepening_search::IterativeDeepeningSearch,
        },
    },
    experimental_lib_api::{derived_puzzle_search_phase::DerivedPuzzleSearchPhase, SearchPhase},
    scramble::{
        orbit_pieces_byte_slice::orbit_pieces_byte_slice,
        parity::basic_parity,
        puzzles::{
            cube4x4x4::{
                phase2::transfer_orbit,
                wings::{
                    NUM_WINGS, POSITION_IS_PRIMARY, WING_TO_PARTNER_WING,
                    WING_TO_PRIMARY_WING_IN_DEDGE,
                },
            },
            definitions::{cube4x4x4_kpuzzle, cube4x4x4_phase3_search_kpuzzle},
        },
        scramble_search::move_list_from_vec,
    },
};

#[derive(Clone, Debug)]
pub(crate) struct Cube4x4x4Phase3PatternDeriver {
    masked_kpuzzle_deriver: MaskedKPuzzleDeriver,
    phase3_kpuzzle: KPuzzle,
}

impl PatternDeriver<KPuzzle> for Cube4x4x4Phase3PatternDeriver {
    type DerivedPattern = KPattern;

    fn derive_pattern(&self, source_puzzle_pattern: &KPattern) -> Option<Self::DerivedPattern> {
        let mut unmasked_pattern = self.phase3_kpuzzle.default_pattern();

        // WINGS
        {
            let source_orbit_info = &cube4x4x4_kpuzzle().data.ordered_orbit_info[1];
            debug_assert_eq!(source_orbit_info.name.0, "WINGS");
            let destination_orbit_info =
                &cube4x4x4_phase3_search_kpuzzle().data.ordered_orbit_info[0];
            debug_assert_eq!(destination_orbit_info.name.0, "WINGS");

            transfer_orbit(
                source_puzzle_pattern,
                source_orbit_info,
                &mut unmasked_pattern,
                destination_orbit_info,
            );
        }
        canonicalize_wings(&mut unmasked_pattern);

        // CENTERS
        {
            let source_orbit_info = &cube4x4x4_kpuzzle().data.ordered_orbit_info[2];
            debug_assert_eq!(source_orbit_info.name.0, "CENTERS");
            let destination_orbit_info =
                &cube4x4x4_phase3_search_kpuzzle().data.ordered_orbit_info[1];
            debug_assert_eq!(destination_orbit_info.name.0, "CENTERS");

            transfer_orbit(
                source_puzzle_pattern,
                source_orbit_info,
                &mut unmasked_pattern,
                destination_orbit_info,
            );
        }

        // PLL_PARITY
        {
            let corners_byte_slice = orbit_pieces_byte_slice(source_puzzle_pattern, 0, "CORNERS");
            let wings_byte_slice = orbit_pieces_byte_slice(source_puzzle_pattern, 1, "WINGS");
            let mut wings_in_primary_positions = Vec::<u8>::default();
            for i in 0..NUM_WINGS {
                if POSITION_IS_PRIMARY[i as usize] {
                    wings_in_primary_positions
                        .push(WING_TO_PRIMARY_WING_IN_DEDGE[wings_byte_slice[i as usize] as usize]);
                }
            }

            let pll_parity_orbit_info =
                &cube4x4x4_phase3_search_kpuzzle().data.ordered_orbit_info[2];
            debug_assert_eq!(pll_parity_orbit_info.name.0, "PLL_PARITY");

            let orientation = (basic_parity(corners_byte_slice)
                + basic_parity(&wings_in_primary_positions))
            .to_0_or_1();
            unmasked_pattern.set_orientation_with_mod(
                pll_parity_orbit_info,
                0,
                &OrientationWithMod {
                    orientation,
                    orientation_mod: 0,
                },
            )
        }

        // TODO: Masking is only needed for centers (and only until we update the default pattern in `4x4x4.kpuzzle.json` to have indistinguishable centers).
        let masked_pattern = self
            .masked_kpuzzle_deriver
            .derive_pattern(&unmasked_pattern)?;

        Some(masked_pattern)
    }
}

impl Default for Cube4x4x4Phase3PatternDeriver {
    fn default() -> Self {
        let masked_kpuzzle_deriver =
            MaskedKPuzzleDeriver::new(cube4x4x4_phase3_search_kpuzzle().default_pattern());
        Self {
            masked_kpuzzle_deriver,
            phase3_kpuzzle: cube4x4x4_phase3_search_kpuzzle().clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Cube4x4x4Phase3Puzzle {
    // TODO: remove one level of indirection here?
    internal_derived_puzzle:
        UnenumeratedDerivedPatternPuzzle<KPuzzle, KPuzzle, Cube4x4x4Phase3PatternDeriver>,
}

impl SemiGroupActionPuzzle for Cube4x4x4Phase3Puzzle {
    type Pattern = KPattern;
    type Transformation = KTransformation;

    fn move_order(
        &self,
        r#move: &cubing::alg::Move,
    ) -> Result<crate::_internal::search::move_count::MoveCount, cubing::kpuzzle::InvalidAlgError>
    {
        self.internal_derived_puzzle.move_order(r#move)
    }

    fn puzzle_transformation_from_move(
        &self,
        r#move: &cubing::alg::Move,
    ) -> Result<Self::Transformation, cubing::kpuzzle::InvalidAlgError> {
        self.internal_derived_puzzle
            .puzzle_transformation_from_move(r#move)
    }

    fn do_moves_commute(
        &self,
        move1: &cubing::alg::Move,
        move2: &cubing::alg::Move,
    ) -> Result<bool, cubing::kpuzzle::InvalidAlgError> {
        self.internal_derived_puzzle.do_moves_commute(move1, move2)
    }

    fn pattern_apply_transformation(
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Option<Self::Pattern> {
        let mut pattern = self
            .internal_derived_puzzle
            .pattern_apply_transformation(pattern, transformation_to_apply)?;
        canonicalize_wings(&mut pattern);
        Some(pattern)
    }

    fn pattern_apply_transformation_into(
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    ) -> bool {
        if !self
            .internal_derived_puzzle
            .pattern_apply_transformation_into(pattern, transformation_to_apply, into_pattern)
        {
            return false;
        }
        canonicalize_wings(into_pattern);
        true
    }
}

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

/// This performs canonicalization with respect to outer moves. That is, we
/// want 3×3×3 pre-moves on a pattern to have no effect on the canonicalized
/// wing state in phase 3.
///
/// This is similar to "pre-moves" (e.g. in FMC) where you can conjugate by
/// any combination of 3 rotation moves (24 possibilities total), except
/// that here you can conjugate by any of 6 face moves (12! * 2^12 states,
/// ignoring parity considerations).
fn canonicalize_wings(pattern: &mut KPattern) {
    let orbit_info = &cube4x4x4_phase3_search_kpuzzle().data.ordered_orbit_info[0];
    debug_assert_eq!(orbit_info.name.0, "WINGS");

    // Input
    // [14, 16, 4, 18, 2, 17, 8, 12, 21, 7, 20, 6, 3, 5, 22, 23, 10, 15, 0, 9, 1, 13, 19, 11]
    //   ↑   ↑  ↑   ↑                    ↑      ↑                     ↑     ↑  ↑   ↑   ↑   ↑

    // POSITION_IS_LOW = [1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1]
    // PIECE_TO_LOW_PIECE = [0, 1, 2, 3, 3, 11, 23, 17, 2, 9, 20, 11, 1, 19, 21, 9, 0, 17, 22, 19, 20, 21, 22, 23];
    // PIECE_TO_HIGH_PIECE = [16, 12, 8, 4, 4, 5, 6, 7, 8, 15, 10, 5, 12, 13, 14, 15, 16, 7, 18, 13, 10, 14, 18, 6];

    // low_position_to_canonicalized_low_piece
    // [N, N, N, N,   N, N, N, N,   N, N, N, N,   N, N, 0, N,   N, N, N, N,   N, 16, N, N]

    // Output:
    // [ 0,  1,  2,  3,  _,  _,  _,  _,  _,  _,  _, _,  _,  _,  _,  _, 17,  _, 19, 20, 21, 22, 23]
    //   ↑   ↑   ↑   ↑                       ↑      ↑                            ↑   ↑  ↑   ↑   ↑
    // [ 0,  1,  2,  3,  _,  _,  _, 16,  _,  _,  _, _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _,  _]

    // dbg!(&pattern);

    //    [0, 1, 5, 18, 16, 20, 21, 7, 11, 10, 9, 14, 3, 15, 8, 2, 12, 17, 4, 19, 13, 6, 22, 23]
    // i:  0  1  2   3   4   5   6  7   8   9 10

    // 9 ↔ 15
    // 10 ↔ 20
    // {
    //     let mut seen_in_low_position: [bool; 24] = [false; 24];
    //     let mut seen_in_high_position: [bool; 24] = [false; 24];
    //     for i in 0..orbit_info.num_pieces {
    //         if POSITION_IS_LOW[i as usize] {
    //             seen_in_low_position
    //                 [PIECE_TO_LOW_PIECE[pattern.get_piece(orbit_info, i) as usize] as usize] =
    //                 true;
    //         } else {
    //             seen_in_high_position
    //                 [PIECE_TO_LOW_PIECE[pattern.get_piece(orbit_info, i) as usize] as usize] =
    //                 true;
    //         }
    //     }
    //     dbg!(seen_in_low_position);
    //     dbg!(seen_in_high_position);
    //     dbg!(seen_in_low_position == seen_in_high_position);
    // }

    // let mut piece_mapping: [Option<u8>; 24] = [None; 24];
    // for i in 0..orbit_info.num_pieces {
    //     // i == 0
    //     if POSITION_IS_LOW[i as usize] {
    //         let piece = pattern.get_piece(orbit_info, i); // 14 (high piece)
    //         let partner_piece = PIECE_TO_PARTNER_PIECE[piece as usize]; // 21 (low piece)
    //         piece_mapping[partner_piece as usize] = Some(PIECE_TO_PARTNER_PIECE[i as usize]);
    //         // piece_mapping[21] = 16;
    //     }
    // }
    // dbg!(&piece_mapping);
    // for i in 0..orbit_info.num_pieces {
    //     if POSITION_IS_LOW[i as usize] {
    //         pattern.set_piece(orbit_info, i, i);
    //     } else {
    //         dbg!(i);
    //         // i == 7
    //         let original_piece_in_position_i = pattern.get_piece(orbit_info, i); // 21
    //         dbg!(original_piece_in_position_i);
    //         let new_piece_in_position_i =
    //             piece_mapping[original_piece_in_position_i as usize].unwrap(); // 16
    //         assert!(!POSITION_IS_LOW[new_piece_in_position_i as usize]);
    //         pattern.set_piece(orbit_info, i, new_piece_in_position_i);
    //     }
    // }

    // Tracks piece assignments for the canonicalization remapping.
    // The piece with value `i` (regardless of its position) will be replaced with the value `piece_mapping[i]`.
    let mut piece_mapping: [Option<u8>; 24] = [None; 24];
    // When a piece has not yet been assigned a mapping (due to its partner), we use this to select the next available value.
    let mut next_assignment: u8 = 0;
    // Tracks piece assignments that cannot be used for future pieces because they were used by (the partner of) a previous assigned piece.
    let mut blocked_assignments: [bool; 24] = [false; 24];
    for i in 0..orbit_info.num_pieces {
        let piece = pattern.get_piece(orbit_info, i) as usize;
        if piece_mapping[piece].is_none() {
            let assigned_piece = next_assignment;
            piece_mapping[piece] = Some(assigned_piece);

            let partner_piece = WING_TO_PARTNER_WING[piece];
            let assigned_piece_partner = WING_TO_PARTNER_WING[assigned_piece as usize];
            piece_mapping[partner_piece as usize] = Some(assigned_piece_partner);
            blocked_assignments[assigned_piece_partner as usize] = true;

            next_assignment += 1;
            while next_assignment < orbit_info.num_pieces
                && blocked_assignments[next_assignment as usize]
            {
                next_assignment += 1;
            }
        }
    }
    for i in 0..orbit_info.num_pieces {
        pattern.set_piece(
            orbit_info,
            i,
            // By the time we reach this loop, every entry of `piece_mapping` has a `Some(…)` value.
            // (If not, the input piece array was invalid. We don't perform validation in this function, since it's very hot code.)
            piece_mapping[pattern.get_piece(orbit_info, i) as usize].unwrap(),
        );
    }

    // dbg!(&pattern);
    // {
    //     let mut seen_in_low_position: [bool; 24] = [false; 24];
    //     let mut seen_in_high_position: [bool; 24] = [false; 24];
    //     for i in 0..orbit_info.num_pieces {
    //         if POSITION_IS_LOW[i as usize] {
    //             seen_in_low_position
    //                 [PIECE_TO_LOW_PIECE[pattern.get_piece(orbit_info, i) as usize] as usize] =
    //                 true;
    //         } else {
    //             seen_in_high_position
    //                 [PIECE_TO_LOW_PIECE[pattern.get_piece(orbit_info, i) as usize] as usize] =
    //                 true;
    //         }
    //     }
    //     dbg!(seen_in_low_position);
    //     dbg!(seen_in_high_position);
    //     dbg!(seen_in_low_position == seen_in_high_position);
    // }
}

impl HashablePatternPuzzle for Cube4x4x4Phase3Puzzle {
    fn pattern_hash_u64(&self, pattern: &Self::Pattern) -> u64 {
        self.internal_derived_puzzle
            .derived_puzzle
            .pattern_hash_u64(pattern)
    }
}

impl Default for Cube4x4x4Phase3Puzzle {
    fn default() -> Self {
        let derived_puzzle = UnenumeratedDerivedPatternPuzzle::<
            KPuzzle,
            KPuzzle,
            Cube4x4x4Phase3PatternDeriver,
        >::new(
            cube4x4x4_kpuzzle().clone(),
            cube4x4x4_phase3_search_kpuzzle().clone(),
            Cube4x4x4Phase3PatternDeriver::default(),
        );
        Self {
            internal_derived_puzzle: derived_puzzle,
        }
    }
}

impl PatternDeriver for Cube4x4x4Phase3Puzzle {
    type DerivedPattern = KPattern;

    fn derive_pattern(
        &self,
        source_puzzle_pattern: &<KPuzzle as SemiGroupActionPuzzle>::Pattern,
    ) -> Option<Self::DerivedPattern> {
        self.internal_derived_puzzle
            .derive_pattern(source_puzzle_pattern)
    }
}

impl DerivedPuzzle<KPuzzle> for Cube4x4x4Phase3Puzzle {}

pub(crate) struct Cube4x4x4Phase3Search {
    derived_puzzle_search_phase: DerivedPuzzleSearchPhase<KPuzzle, Cube4x4x4Phase3Puzzle>,
}

impl Default for Cube4x4x4Phase3Search {
    fn default() -> Self {
        let phase3_generator_moves = move_list_from_vec(vec![
            "Uw2", "U", // U
            "L", // L
            "Fw2", "F2", // F
            "Rw2", "R",  // R
            "B2", // B
            "D",  // D
        ]);

        let cube4x4x4_phase3_puzzle = Cube4x4x4Phase3Puzzle::default();

        let phase3_iterative_deepening_search =
                IterativeDeepeningSearch::<Cube4x4x4Phase3Puzzle>::try_new_prune_table_construction_shim::<
                    HashPruneTable<Cube4x4x4Phase3Puzzle>,
                >(
                    cube4x4x4_phase3_puzzle.clone(),
                    phase3_generator_moves,
                    vec![cube4x4x4_phase3_search_kpuzzle().default_pattern()],
                    Default::default(),
                    None,
                )
                .unwrap();
        let derived_puzzle_search_phase =
            DerivedPuzzleSearchPhase::<KPuzzle, Cube4x4x4Phase3Puzzle>::new(
                "4×4×4 reduction with parity avoidance".to_owned(),
                cube4x4x4_phase3_puzzle,
                phase3_iterative_deepening_search,
                Default::default(),
            );
        Self {
            derived_puzzle_search_phase,
        }
    }
}

impl SearchPhase<KPuzzle> for Cube4x4x4Phase3Search {
    fn phase_name(&self) -> &str {
        self.derived_puzzle_search_phase.phase_name()
    }

    fn first_solution(
        &mut self,
        phase_search_pattern: &KPattern,
    ) -> Result<Option<Alg>, SearchError> {
        self.derived_puzzle_search_phase
            .first_solution(phase_search_pattern)
    }
}
