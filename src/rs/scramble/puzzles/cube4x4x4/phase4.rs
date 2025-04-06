use cubing::{
    alg::Alg,
    kpuzzle::{KPattern, KPuzzle, OrientationWithMod},
    puzzles::cube3x3x3_kpuzzle,
};

use crate::{
    _internal::{
        errors::SearchError, puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
        search::coordinates::pattern_deriver::PatternDeriver,
    },
    experimental_lib_api::SearchPhase,
    scramble::{
        puzzles::{
            cube4x4x4::wings::POSITION_IS_PRIMARY,
            definitions::cube4x4x4_kpuzzle,
            two_phase_3x3x3_scramble_finder::{
                TwoPhase3x3x3PrefixOrSuffixConstraints, TwoPhase3x3x3ScrambleFinder,
                TwoPhase3x3x3ScrambleOptions,
            },
        },
        solving_based_scramble_finder::SolvingBasedScrambleFinder,
    },
};

use super::{phase2::transfer_orbit, wings::NUM_WINGS};

#[derive(Clone, Debug, Default)]
pub(crate) struct Cube4x4x4Phase4Puzzle {}

const REID_PIECE_TO_PRIMARY_WING_POSITION: [u8; 12] = [2, 1, 0, 3, 20, 21, 22, 23, 9, 11, 19, 17];

// Note: this does not specify orientation.
const WING_TO_REID_PIECE: [u8; NUM_WINGS as usize] = [
    2, 1, 0, 3, 3, 9, 7, 11, 0, 8, 4, 9, 1, 10, 5, 8, 2, 11, 6, 10, 4, 5, 6, 7,
];

impl PatternDeriver<KPuzzle> for Cube4x4x4Phase4Puzzle {
    type DerivedPattern = KPattern;

    fn derive_pattern(
        &self,
        source_puzzle_pattern: &<KPuzzle as crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle>::Pattern,
    ) -> Option<Self::DerivedPattern> {
        let kpuzzle_4x4x4 = cube4x4x4_kpuzzle();
        let kpuzzle_3x3x3 = cube3x3x3_kpuzzle();
        let mut pattern = kpuzzle_3x3x3.default_pattern();
        {
            let cube4x4x4_corners_orbit_info = &kpuzzle_4x4x4.data.ordered_orbit_info[0];
            debug_assert_eq!(cube4x4x4_corners_orbit_info.name.0, "CORNERS");

            let cube3x3x3_corners_orbit_info = &kpuzzle_3x3x3.data.ordered_orbit_info[1];
            debug_assert_eq!(cube3x3x3_corners_orbit_info.name.0, "CORNERS");

            transfer_orbit(
                source_puzzle_pattern,
                cube4x4x4_corners_orbit_info,
                &mut pattern,
                cube3x3x3_corners_orbit_info,
            );
        }
        {
            let cube4x4x4_wings_orbit_info = &kpuzzle_4x4x4.data.ordered_orbit_info[1];
            debug_assert_eq!(cube4x4x4_wings_orbit_info.name.0, "WINGS");

            let cube3x3x3_edges_orbit_info = &kpuzzle_3x3x3.data.ordered_orbit_info[0];
            debug_assert_eq!(cube3x3x3_edges_orbit_info.name.0, "EDGES");

            for cube3x3x3_edge_position in 0..cube3x3x3_edges_orbit_info.num_pieces {
                // 4
                let primary_wing_position =
                    REID_PIECE_TO_PRIMARY_WING_POSITION[cube3x3x3_edge_position as usize];
                let piece_in_primary_wing_position = source_puzzle_pattern
                    .get_piece(cube4x4x4_wings_orbit_info, primary_wing_position);
                let reid_piece = WING_TO_REID_PIECE[piece_in_primary_wing_position as usize];
                let orientation = if POSITION_IS_PRIMARY[piece_in_primary_wing_position as usize] {
                    0
                } else {
                    1
                };
                pattern.set_piece(
                    cube3x3x3_edges_orbit_info,
                    cube3x3x3_edge_position,
                    reid_piece,
                );
                pattern.set_orientation_with_mod(
                    cube3x3x3_edges_orbit_info,
                    cube3x3x3_edge_position,
                    &OrientationWithMod::new_using_default_orientation_mod(orientation),
                );
            }
        }
        Some(pattern)
    }
}

#[derive(Default)]
pub(crate) struct Cube4x4x4Phase4Search {
    // TODO: share this with `ScrambleFinderCacher` in a way that also allows deallocating automatically if the `Cube4x4x4Phase4Search` is the only one who's used it
    cube3x3x3_scramble_finder: TwoPhase3x3x3ScrambleFinder,
    cube4x4x4_phase4_puzzle: Cube4x4x4Phase4Puzzle,
}

impl SearchPhase<KPuzzle> for Cube4x4x4Phase4Search {
    fn phase_name(&self) -> &str {
        "Reduced 3×3×3"
    }

    fn first_solution(
        &mut self,
        phase_search_pattern: &<KPuzzle as SemiGroupActionPuzzle>::Pattern,
    ) -> Result<Option<Alg>, SearchError> {
        let Some(pattern) = self
            .cube4x4x4_phase4_puzzle
            .derive_pattern(phase_search_pattern)
        else {
            return Err("Could not derive 3×3×3 pattern".into());
        };
        Ok(Some(self.cube3x3x3_scramble_finder.solve_pattern(
            &pattern,
            &TwoPhase3x3x3ScrambleOptions {
                prefix_or_suffix_constraints: TwoPhase3x3x3PrefixOrSuffixConstraints::None,
            },
        )?))
    }
}
