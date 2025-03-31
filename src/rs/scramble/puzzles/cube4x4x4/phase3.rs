use cubing::kpuzzle::{KPattern, KPuzzle, KTransformation};

use crate::{
    _internal::{
        cli::args::VerbosityLevel,
        puzzle_traits::puzzle_traits::{HashablePatternPuzzle, SemiGroupActionPuzzle},
        search::{
            coordinates::{
                masked_kpuzzle_deriver::MaskedDerivedKPuzzle,
                pattern_deriver::{DerivedPuzzle, PatternDeriver},
            },
            hash_prune_table::HashPruneTable,
            iterative_deepening::iterative_deepening_search::{
                IterativeDeepeningSearch, IterativeDeepeningSearchConstructionOptions,
            },
            search_logger::SearchLogger,
        },
    },
    experimental_lib_api::{derived_puzzle_search_phase::DerivedPuzzleSearchPhase, SearchPhase},
    scramble::{
        puzzles::{
            cube4x4x4::phase2::PIECE_TO_PARTNER_PIECE,
            definitions::cube4x4x4_phase3_target_kpattern,
        },
        scramble_search::move_list_from_vec,
    },
};

use super::phase2::POSITION_IS_LOW;

// pub(crate) fn cube4x4x4_phase3_search()
#[derive(Clone, Debug)]
pub(crate) struct Cube4x4x4Phase3Puzzle {
    masked_derived_puzzle: MaskedDerivedKPuzzle,
}

impl PatternDeriver<KPuzzle> for Cube4x4x4Phase3Puzzle {
    type DerivedPattern = KPattern;

    fn derive_pattern(&self, source_puzzle_pattern: &KPattern) -> Option<Self::DerivedPattern> {
        dbg!(&source_puzzle_pattern);
        let mut pattern = self
            .masked_derived_puzzle
            .derive_pattern(source_puzzle_pattern)?;
        dbg!(&pattern);
        Self::canonicalize_wings(&mut pattern);
        Some(pattern)
    }
}

impl SemiGroupActionPuzzle for Cube4x4x4Phase3Puzzle {
    type Pattern = KPattern;
    type Transformation = KTransformation;

    fn move_order(
        &self,
        r#move: &cubing::alg::Move,
    ) -> Result<crate::_internal::search::move_count::MoveCount, cubing::kpuzzle::InvalidAlgError>
    {
        self.masked_derived_puzzle.move_order(r#move)
    }

    fn puzzle_transformation_from_move(
        &self,
        r#move: &cubing::alg::Move,
    ) -> Result<Self::Transformation, cubing::kpuzzle::InvalidAlgError> {
        self.masked_derived_puzzle
            .puzzle_transformation_from_move(r#move)
    }

    fn do_moves_commute(
        &self,
        move1: &cubing::alg::Move,
        move2: &cubing::alg::Move,
    ) -> Result<bool, cubing::kpuzzle::InvalidAlgError> {
        self.masked_derived_puzzle.do_moves_commute(move1, move2)
    }

    fn pattern_apply_transformation(
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Option<Self::Pattern> {
        let mut pattern = self
            .masked_derived_puzzle
            .pattern_apply_transformation(pattern, transformation_to_apply)?;
        Self::canonicalize_wings(&mut pattern);
        Some(pattern)
    }

    fn pattern_apply_transformation_into(
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    ) -> bool {
        // dbg!(&pattern);
        if !self
            .masked_derived_puzzle
            .pattern_apply_transformation_into(pattern, transformation_to_apply, into_pattern)
        {
            return false;
        }
        Self::canonicalize_wings(into_pattern);
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

impl Cube4x4x4Phase3Puzzle {
    /// This performs canonicalization with respect to outer moves. That is, we
    /// want 3×3×3 pre-moves on a pattern to have no effect on the canonicalized
    /// wing state in phase 3.
    ///
    /// This is similar to "pre-moves" (e.g. in FMC) where you can conjugate by
    /// any combination of 3 rotation moves (24 possibilities total), except
    /// that here you can conjugate by any of 6 face moves (12! * 2^12 states,
    /// ignoring parity considerations).
    fn canonicalize_wings(pattern: &mut KPattern) {
        let orbit_info = &pattern.kpuzzle().clone().data.ordered_orbit_info[1]; // TODO: avoid clone?
                                                                                // assert_eq!(orbit_info.name.0, "WINGS");

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

        let mut piece_mapping: [Option<u8>; 24] = [None; 24];
        let mut next_low_piece: u8 = 0;
        for i in 0..orbit_info.num_pieces {
            if piece_mapping[pattern.get_piece(orbit_info, i) as usize].is_none() {
                piece_mapping[pattern.get_piece(orbit_info, i) as usize] = Some(next_low_piece);
                piece_mapping
                    [PIECE_TO_PARTNER_PIECE[pattern.get_piece(orbit_info, i) as usize] as usize] =
                    Some(PIECE_TO_PARTNER_PIECE[next_low_piece as usize]);

                next_low_piece += 1;
                while next_low_piece < orbit_info.num_pieces
                    && !POSITION_IS_LOW[next_low_piece as usize]
                {
                    next_low_piece += 1;
                }
            }
        }
        for i in 0..orbit_info.num_pieces {
            pattern.set_piece(
                orbit_info,
                i,
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
}

impl HashablePatternPuzzle for Cube4x4x4Phase3Puzzle {
    fn pattern_hash_u64(&self, pattern: &Self::Pattern) -> u64 {
        self.masked_derived_puzzle.pattern_hash_u64(pattern)
    }
}

impl Default for Cube4x4x4Phase3Puzzle {
    fn default() -> Self {
        let masked_derived_puzzle =
            MaskedDerivedKPuzzle::new_from_mask(cube4x4x4_phase3_target_kpattern().clone());
        Self {
            masked_derived_puzzle,
        }
    }
}

impl DerivedPuzzle<KPuzzle> for Cube4x4x4Phase3Puzzle {}

pub(crate) struct Cube4x4x4Phase3Search {
    derived_puzzle_search_phase: DerivedPuzzleSearchPhase<KPuzzle, Cube4x4x4Phase3Puzzle>,
}

impl Default for Cube4x4x4Phase3Search {
    fn default() -> Self {
        let phase3_generator_moves = move_list_from_vec(vec![
            "Uw2", "U", "Lw2", "L", "Fw", "F2", "Rw2", "R", "Bw2", "B2", "Dw2", "D",
        ]);

        let cube4x4x4_phase3_puzzle = Cube4x4x4Phase3Puzzle::default();

        let phase3_iterative_deepening_search =
                IterativeDeepeningSearch::<Cube4x4x4Phase3Puzzle>::try_new_prune_table_construction_shim::<
                    HashPruneTable<Cube4x4x4Phase3Puzzle>,
                >(
                    cube4x4x4_phase3_puzzle.clone(),
                    phase3_generator_moves,
                    vec![cube4x4x4_phase3_target_kpattern().clone()],
                    IterativeDeepeningSearchConstructionOptions {
                        search_logger: SearchLogger {
                            verbosity: VerbosityLevel::Info, // TODO
                        }.into(),
                        ..Default::default()
                    },
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
    ) -> Result<Option<cubing::alg::Alg>, crate::_internal::errors::SearchError> {
        self.derived_puzzle_search_phase
            .first_solution(phase_search_pattern)
    }
}
