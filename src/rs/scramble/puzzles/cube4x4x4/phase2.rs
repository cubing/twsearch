use std::io::{stdout, Write};

use cubing::{
    alg::Alg,
    kpuzzle::{KPattern, KPuzzle, KPuzzleOrbitInfo, OrientationWithMod},
};

use crate::{
    _internal::{GenericPuzzle, GenericPuzzleCore, ReplacementSolutionCondition, SearchHeuristic},
    scramble::{
        puzzles::{
            cube4x4x4::orbit_info::orbit_info,
            definitions::{
                cube4x4x4_kpuzzle, cube4x4x4_phase2_target_kpattern,
                cube4x4x4_with_wing_parity_kpuzzle,
            },
        },
        randomize::{basic_parity, BasicParity},
    },
};

use super::phase2_symmetry::{Phase2Puzzle, Phase2SymmetryTables};

const NUM_4X4X4_EDGES: usize = 24;

/**
 * Each pair of edges ("wings") on a solved 4x4x4 has two position:
 *
 * - The "high" position — this includes UBl (the first piece in Speffz) and all the places that the UBl piece can be moved by <U, L, R, D>
 * - The "low" position — the other position.
 *
 * Further:
 *
 * - A piece that starts in a high position is a high piece.
 * - A piece that starts in a high position is a low piece.
 *
 * These orbits are preserved by U, Uw2, D, Dw2, F, Fw2, B, Bw2, R2, Rw2, L2, and Lw2.
 *
 * And:
 *
 * Each high-low pair is assigned an index, which is the position index of the high position/piece in Speffz.
 *
 * This encodes the convention established by: http://cubezzz.dyndns.org/drupal/?q=node/view/73#comment-2588
 */
#[derive(Copy, Clone, PartialEq)]
struct EdgePairIndex(u8);
const EDGE_TO_INDEX: [EdgePairIndex; NUM_4X4X4_EDGES] = [
    // U
    EdgePairIndex(0), // high
    EdgePairIndex(1), // high
    EdgePairIndex(2), // high
    EdgePairIndex(3), // high
    // L
    EdgePairIndex(3),  // low
    EdgePairIndex(11), // low
    EdgePairIndex(23), // low
    EdgePairIndex(17), // low
    // F
    EdgePairIndex(2),  // low
    EdgePairIndex(9),  // high
    EdgePairIndex(20), // low
    EdgePairIndex(11), // high
    // R
    EdgePairIndex(1),  // low
    EdgePairIndex(19), // low
    EdgePairIndex(21), // low
    EdgePairIndex(9),  // low
    // B
    EdgePairIndex(0),  // low
    EdgePairIndex(17), // high
    EdgePairIndex(22), // low
    EdgePairIndex(19), // high
    // D
    EdgePairIndex(20), // high
    EdgePairIndex(21), // high
    EdgePairIndex(22), // high
    EdgePairIndex(23), // high
];

// Checks if either a position or a piece is high (same code for both).
fn is_high(position_or_piece: u8) -> bool {
    EDGE_TO_INDEX[position_or_piece as usize].0 == position_or_piece
}

#[derive(Clone, PartialEq, Debug)]
enum Phase2EdgeOrientation {
    Unknown,
    // Either a high piece in a high position, or a low piece in a low position.
    Oriented,
    // Either a high piece in a low position, or a low piece in a high position.
    Misoriented,
}

fn calculate_wing_parity(pattern: &KPattern) -> BasicParity {
    let wings_orbit_info = orbit_info(pattern.kpuzzle(), 1, "WINGS");
    let wing_parity = basic_parity(
        &unsafe {
            pattern.packed_orbit_data().byte_slice() /* TODO */
        }[wings_orbit_info.pieces_or_permutations_offset..wings_orbit_info.orientations_offset],
    );
    dbg!(&wing_parity);
    wing_parity
}

fn set_wing_parity(pattern: &mut KPattern, wing_parity: BasicParity) {
    let kpuzzle_clone = pattern.kpuzzle().clone();
    let wing_parity_orbit_info = orbit_info(&kpuzzle_clone, 3, "WING_PARITY");
    pattern.set_orientation_with_mod(
        wing_parity_orbit_info,
        0,
        &OrientationWithMod {
            orientation: wing_parity.into(),
            orientation_mod: 0,
        },
    );
}

pub(crate) fn pattern_to_phase2_pattern(pattern: &KPattern) -> KPattern {
    let phase1_kpuzzle = cube4x4x4_kpuzzle();
    let phase2_kpuzzle = cube4x4x4_with_wing_parity_kpuzzle();
    let phase2_target_pattern = cube4x4x4_phase2_target_kpattern();

    let mut new_pattern = phase2_kpuzzle.default_pattern();
    for orbit_info in phase1_kpuzzle.orbit_info_iter() {
        for i in 0..orbit_info.num_pieces {
            remap_piece_for_phase1_or_phase2_search_pattern(
                orbit_info,
                pattern,
                phase2_target_pattern,
                &mut new_pattern,
                i,
            );
        }
    }

    let wing_parity = calculate_wing_parity(pattern);
    set_wing_parity(&mut new_pattern, wing_parity);
    new_pattern
}
pub(crate) struct Phase2ReplacementSolutionCondition {
    pub(crate) phase2_search_full_pattern: <KPuzzle as GenericPuzzleCore>::Pattern,
    pub(crate) _debug_num_checked: usize, // TODO: remove
    pub(crate) _debug_num_centers_rejected: usize, // TODO: remove
    pub(crate) _debug_num_total_rejected: usize, // TODO: remove
    pub(crate) _debug_num_basic_parity_rejected: usize, // TODO: remove
    pub(crate) _debug_num_known_pair_orientation_rejected: usize, // TODO: remove
    pub(crate) _debug_num_edge_parity_rejected: usize, // TODO: remove
}

impl Phase2ReplacementSolutionCondition {
    fn log(&self) {
        if !self._debug_num_total_rejected.is_power_of_two() {
            return;
        }
        println!(
            "{} total phase 2 rejections ({} centers, {} basic parity, {} known pair orientation, {} edge parity)",
            self._debug_num_total_rejected,
            self._debug_num_centers_rejected,
            self._debug_num_basic_parity_rejected,
            self._debug_num_known_pair_orientation_rejected,
            self._debug_num_edge_parity_rejected,
        );
    }

    // fn debug_record_centers_rejection(&mut self) {
    //     self._debug_num_total_rejected += 1;
    //     self._debug_num_centers_rejected += 1;
    //     self.log()
    // }

    // fn debug_record_basic_parity_rejection(&mut self) {
    //     self._debug_num_total_rejected += 1;
    //     self._debug_num_basic_parity_rejected += 1;
    //     self.log()
    // }

    // fn debug_record_known_pair_orientation_rejection(&mut self) {
    //     self._debug_num_total_rejected += 1;
    //     self._debug_num_known_pair_orientation_rejected += 1;
    //     self.log()
    // }

    // fn debug_record_edge_parity_rejection(&mut self) {
    //     self._debug_num_total_rejected += 1;
    //     self._debug_num_edge_parity_rejected += 1;
    //     self.log()
    // }
}

// TODO: change the 4x4x4 Speffz def to have indistinguishable centers and get rid of this.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum SideCenter {
    L,
    R,
}

pub(crate) const PHASE2_SOLVED_SIDE_CENTER_CASES: [[[SideCenter; 4]; 2]; 12] = [
    // flat faces
    [
        [SideCenter::L, SideCenter::L, SideCenter::L, SideCenter::L],
        [SideCenter::R, SideCenter::R, SideCenter::R, SideCenter::R],
    ],
    [
        [SideCenter::R, SideCenter::R, SideCenter::R, SideCenter::R],
        [SideCenter::L, SideCenter::L, SideCenter::L, SideCenter::L],
    ],
    // horizontal bars
    [
        [SideCenter::L, SideCenter::L, SideCenter::R, SideCenter::R],
        [SideCenter::L, SideCenter::L, SideCenter::R, SideCenter::R],
    ],
    [
        [SideCenter::R, SideCenter::R, SideCenter::L, SideCenter::L],
        [SideCenter::R, SideCenter::R, SideCenter::L, SideCenter::L],
    ],
    [
        [SideCenter::R, SideCenter::R, SideCenter::L, SideCenter::L],
        [SideCenter::L, SideCenter::L, SideCenter::R, SideCenter::R],
    ],
    [
        [SideCenter::L, SideCenter::L, SideCenter::R, SideCenter::R],
        [SideCenter::R, SideCenter::R, SideCenter::L, SideCenter::L],
    ],
    // vertical bars
    [
        [SideCenter::L, SideCenter::R, SideCenter::R, SideCenter::L],
        [SideCenter::L, SideCenter::R, SideCenter::R, SideCenter::L],
    ],
    [
        [SideCenter::R, SideCenter::L, SideCenter::L, SideCenter::R],
        [SideCenter::R, SideCenter::L, SideCenter::L, SideCenter::R],
    ],
    [
        [SideCenter::L, SideCenter::R, SideCenter::R, SideCenter::L],
        [SideCenter::R, SideCenter::L, SideCenter::L, SideCenter::R],
    ],
    [
        [SideCenter::R, SideCenter::L, SideCenter::L, SideCenter::R],
        [SideCenter::L, SideCenter::R, SideCenter::R, SideCenter::L],
    ],
    // checkerboards
    [
        [SideCenter::L, SideCenter::R, SideCenter::L, SideCenter::R],
        [SideCenter::L, SideCenter::R, SideCenter::L, SideCenter::R],
    ],
    [
        [SideCenter::R, SideCenter::L, SideCenter::R, SideCenter::L],
        [SideCenter::R, SideCenter::L, SideCenter::R, SideCenter::L],
    ],
];

fn is_solve_center_center_case(case: &[[SideCenter; 4]; 2]) -> bool {
    for phase2_solved_side_center_case in PHASE2_SOLVED_SIDE_CENTER_CASES {
        if &phase2_solved_side_center_case == case {
            return true;
        }
    }
    false
}

impl ReplacementSolutionCondition<Phase2Puzzle, Phase2SymmetryTables>
    for Phase2ReplacementSolutionCondition
{
    fn should_accept_solution(
        &mut self,
        candidate_pattern: &<Phase2Puzzle as GenericPuzzleCore>::Pattern,
        search_heuristic: &Phase2SymmetryTables,
        candidate_alg: &Alg,
    ) -> bool {
        if search_heuristic.lookup(candidate_pattern) != 0 {
            return false;
        }

        let kpuzzle_4x4x4 = self.phase2_search_full_pattern.kpuzzle();
        let mut accept;

        // TODO: the rest of this function is old code ported to a new implementation. We can probably make this simpler.

        // dbg!(&candidate_alg.to_string());
        let pattern_with_alg_applied = &self
            .phase2_search_full_pattern
            .apply_alg(candidate_alg)
            .expect("Internal error applying an alg from a search result.");

        /******** Centers ********/

        // TODO: is it more efficient to check this later?

        let centers_orbit_info = &kpuzzle_4x4x4.data.ordered_orbit_info[2];
        assert!(centers_orbit_info.name == "CENTERS".into());

        let piece_at_index_0 = pattern_with_alg_applied.get_piece(centers_orbit_info, 0);
        // for i in 0..24 {
        //     print!(
        //         ",{}",
        //         pattern_with_alg_applied.get_piece(centers_orbit_info, i)
        //     );
        //     stdout().flush();
        // }
        // dbg!(piece_at_index_0);
        accept = piece_at_index_0 == 0;

        if !accept {
            // println!("Rejecting due to centers");
            {
                self._debug_num_centers_rejected += 1;
            }
        }

        /******** Edges ********/

        let wings_orbit_info = &kpuzzle_4x4x4.data.ordered_orbit_info[1];
        assert!(wings_orbit_info.name == "WINGS".into());

        if basic_parity(
            &unsafe {
                pattern_with_alg_applied.packed_orbit_data().byte_slice() /* TODO */
            }[wings_orbit_info.pieces_or_permutations_offset..wings_orbit_info.orientations_offset],
        ) != BasicParity::Even
        {
            println!("Rejecting due to basic_parity");
            // println!("false1: {}", candidate_alg);
            {
                self._debug_num_basic_parity_rejected += 1;
            }
            accept = false;
        }

        let mut edge_parity = 0;
        // Indexed by the value stored in an `EdgePairIndex` (i.e. half of the entries will always be `Unknown`).
        let mut known_pair_orientations = vec![Phase2EdgeOrientation::Unknown; NUM_4X4X4_EDGES];
        let mut known_pair_inc = 1;
        for position in 0..24u8 {
            // dbg!(position);
            let position_is_high = is_high(position);

            let piece = pattern_with_alg_applied.get_piece(wings_orbit_info, position);
            let piece_is_high = is_high(piece);

            let pair_orientation = if piece_is_high == position_is_high {
                Phase2EdgeOrientation::Oriented
            } else {
                edge_parity += 1;
                Phase2EdgeOrientation::Misoriented
            };

            let edge_pair_index: EdgePairIndex = EDGE_TO_INDEX[piece as usize];
            // println!(
            //     "comparin': {}, {}, {}, {}, {}, {}, {:?}",
            //     candidate_alg,
            //     position,
            //     piece,
            //     piece_is_high,
            //     position_is_high,
            //     edge_pair_index.0,
            //     pair_orientation
            // );
            match &known_pair_orientations[edge_pair_index.0 as usize] {
                Phase2EdgeOrientation::Unknown => {
                    // println!(
                    //     "known_pair_orientations[{}] = {:?} ({}, {})",
                    //     edge_pair_index.0, pair_orientation, piece_is_high, position_is_high
                    // );
                    known_pair_orientations[edge_pair_index.0 as usize] = pair_orientation
                }
                known_pair_orientation => {
                    if known_pair_orientation != &pair_orientation {
                        // println!("Rejecting due to known_pair_orientation");
                        // println!("false2 {:?}", known_pair_orientation);
                        {
                            self._debug_num_known_pair_orientation_rejected += known_pair_inc;
                            known_pair_inc = 0;
                        }
                        accept = false;
                    }
                }
            }
        }
        if edge_parity % 4 != 0 {
            // println!("Rejecting due to edge_parity");
            // println!("false3: {}, {}", candidate_alg, edge_parity);
            {
                self._debug_num_edge_parity_rejected += 1;
            }
            accept = false;
        }

        if !accept {
            self._debug_num_total_rejected += 1;
        }

        // println!("true: {}", candidate_alg);
        // dbg!(accept);
        if self._debug_num_total_rejected % 10000 == 0 {
            dbg!("--------");
            dbg!(self._debug_num_checked);
            dbg!(self._debug_num_centers_rejected);
            dbg!(self._debug_num_total_rejected);
            dbg!(self._debug_num_basic_parity_rejected);
            dbg!(self._debug_num_known_pair_orientation_rejected);
            dbg!(self._debug_num_edge_parity_rejected);
        };
        accept
    }
}

pub(crate) fn remap_piece_for_phase1_or_phase2_search_pattern(
    orbit_info: &KPuzzleOrbitInfo,
    from_pattern: &KPattern,
    target_pattern: &KPattern,
    search_pattern: &mut KPattern,
    i: u8,
) {
    let old_piece = from_pattern.get_piece(orbit_info, i);
    let old_piece_mapped = target_pattern.get_piece(orbit_info, old_piece);
    search_pattern.set_piece(orbit_info, i, old_piece_mapped);
    let orientation_with_mod = from_pattern.get_orientation_with_mod(orbit_info, i);
    search_pattern.set_orientation_with_mod(orbit_info, i, orientation_with_mod);
    if orbit_info.name == "CORNERS".into() {
        // TODO: handle this properly by taking into account orientation mod.
        search_pattern.set_orientation_with_mod(
            orbit_info,
            i,
            &OrientationWithMod {
                orientation: 0,
                orientation_mod: 1,
            },
        );
    }
}
