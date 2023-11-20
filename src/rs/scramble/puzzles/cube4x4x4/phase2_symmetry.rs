use crate::{
    _internal::{
        options::MetricEnum, PackedKPattern, PackedKPuzzle, PruneTableEntryType, SearchGenerators,
    },
    scramble::puzzles::definitions::{cube4x4x4_packed_kpuzzle, cube4x4x4_phase2_target_pattern},
};

use super::{super::super::scramble_search::generators_from_vec_str, orbit_info::orbit_info};

const NUM_COORDINATES_C8_4D2: usize = 35;
const NUM_COORDINATES_C16_8: usize = 12870;
const NUM_COORDINATES_EP: usize = 2;
const PHASE2_PRUNE_TABLE_SIZE: usize =
    NUM_COORDINATES_C8_4D2 * NUM_COORDINATES_C16_8 * NUM_COORDINATES_EP / 2;
const PHASE2_MOVE_COUNT: usize = 23;
const MOVE_TABLE_UNINITIALIZED_VALUE: Phase2Coordinate = Phase2Coordinate(usize::MAX); // larger than any symcoord

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Phase2Coordinate(usize);

#[derive(Clone, Copy, Debug)]
enum CoordinateTable {
    Coord84,
    Coord168,
    CoordEP,
}

trait Coord {
    fn coordinate_for_pattern(&self, pattern: &PackedKPattern) -> Phase2Coordinate;
    fn move_table(&mut self) -> &mut [[Phase2Coordinate; PHASE2_MOVE_COUNT]];
}

struct Coord84 {
    pack84: [i32; 256], // TODO: should this be unsigned?
    c84move: [[Phase2Coordinate; PHASE2_MOVE_COUNT]; NUM_COORDINATES_C8_4D2],
}

const L_AND_R_CENTER_INDICES: [usize; 8] = [4, 5, 6, 7, 12, 13, 14, 15];
const L_CENTER_PIECE: u8 = 1;

impl Coord for Coord84 {
    fn coordinate_for_pattern(&self, pattern: &PackedKPattern) -> Phase2Coordinate {
        let mut bits = 0;
        // TODO: store this in the struct?
        let centers_orbit_info =
            orbit_info(&pattern.packed_orbit_data.packed_kpuzzle, 2, "CENTERS");
        for idx in L_AND_R_CENTER_INDICES {
            bits *= 2;
            if pattern.get_piece_or_permutation(centers_orbit_info, idx) == L_CENTER_PIECE {
                bits += 1
            }
        }
        Phase2Coordinate(self.pack84[bits] as usize)
    }

    fn move_table(&mut self) -> &mut [[Phase2Coordinate; PHASE2_MOVE_COUNT]] {
        &mut self.c84move
    }
}

impl Default for Coord84 {
    fn default() -> Self {
        Self {
            pack84: [0; 256],
            c84move: [[Phase2Coordinate(0); PHASE2_MOVE_COUNT]; NUM_COORDINATES_C8_4D2],
        }
    }
}

struct Coord168 {
    pack168hi: [i32; 256],
    pack168lo: [i32; 256],
    c168move: [[Phase2Coordinate; PHASE2_MOVE_COUNT]; NUM_COORDINATES_C16_8],
}

impl Coord for Coord168 {
    fn coordinate_for_pattern(&self, pattern: &PackedKPattern) -> Phase2Coordinate {
        let mut bits = 0;
        // TODO: store this in the struct?
        let centers_orbit_info =
            orbit_info(&pattern.packed_orbit_data.packed_kpuzzle, 2, "CENTERS");
        for idx in [0, 1, 2, 3, 8, 9, 10, 11, 16, 17, 18, 19, 20, 21, 22, 23] {
            bits *= 2;
            if pattern.get_piece_or_permutation(centers_orbit_info, idx) == 0 {
                bits += 1
            }
        }
        Phase2Coordinate((self.pack168hi[bits >> 8] + self.pack168lo[bits & 255]) as usize)
    }

    fn move_table(&mut self) -> &mut [[Phase2Coordinate; PHASE2_MOVE_COUNT]] {
        &mut self.c168move
    }
}

impl Default for Coord168 {
    fn default() -> Self {
        Self {
            pack168hi: [0; 256],
            pack168lo: [0; 256],
            c168move: [[Phase2Coordinate(0); PHASE2_MOVE_COUNT]; NUM_COORDINATES_C16_8],
        }
    }
}

struct CoordEP {
    ep_move: [[Phase2Coordinate; PHASE2_MOVE_COUNT]; NUM_COORDINATES_EP],
}

impl Coord for CoordEP {
    fn coordinate_for_pattern(&self, pattern: &PackedKPattern) -> Phase2Coordinate {
        let mut bits = 0;
        let mut r = 0;
        // TODO: store this in the struct?
        let edges_orbit_info = orbit_info(&pattern.packed_orbit_data.packed_kpuzzle, 1, "WINGS");
        for idx in 0..24 {
            if ((bits >> idx) & 1) == 0 {
                let mut cyclen = 0;
                let mut j: usize = idx;
                while ((bits >> j) & 1) == 0 {
                    cyclen += 1;
                    bits |= 1 << j;
                    j = pattern.get_piece_or_permutation(edges_orbit_info, j) as usize;
                }
                r += cyclen + 1;
            }
        }
        Phase2Coordinate((r & 1) as usize)
    }

    fn move_table(&mut self) -> &mut [[Phase2Coordinate; PHASE2_MOVE_COUNT]] {
        &mut self.ep_move
    }
}

impl Default for CoordEP {
    fn default() -> Self {
        Self {
            ep_move: [[Phase2Coordinate(0); PHASE2_MOVE_COUNT]; NUM_COORDINATES_EP],
        }
    }
}

fn bit_count(mut bits: usize) -> i32 {
    let mut r = 0;
    while bits != 0 {
        r += 1;
        bits &= bits - 1;
    }
    r
}

pub(crate) struct Phase2SymmetryTables {
    packed_kpuzzle: PackedKPuzzle,
    phase2_prune_table: [PruneTableEntryType; PHASE2_PRUNE_TABLE_SIZE],
    coord_84: Coord84,
    coord_168: Coord168,
    coord_ep: CoordEP,
}

const PRUNE_TABLE_UNINITIALIZED_VALUE: PruneTableEntryType = PruneTableEntryType::MAX;

impl Phase2SymmetryTables {
    pub(crate) fn new(packed_kpuzzle: PackedKPuzzle) -> Self {
        Self {
            packed_kpuzzle,
            phase2_prune_table: [PRUNE_TABLE_UNINITIALIZED_VALUE; PHASE2_PRUNE_TABLE_SIZE],
            coord_84: Coord84::default(),
            coord_168: Coord168::default(),
            coord_ep: CoordEP::default(),
        }
    }

    pub(crate) fn init_choose_tables(&mut self) {
        let mut at = 0;
        for i in 0..128 {
            if bit_count(i) == 4 {
                self.coord_84.pack84[i] = at;
                self.coord_84.pack84[255 - i] = at;
                at += 1;
            }
        }
        for i in 0..256 {
            self.coord_168.pack168hi[i] = -1;
            self.coord_168.pack168lo[i] = -1;
        }
        at = 0;
        for i in 0..0x10000 {
            if bit_count(i) == 8 {
                if self.coord_168.pack168hi[i >> 8] < 0 {
                    self.coord_168.pack168hi[i >> 8] = at;
                }
                if self.coord_168.pack168lo[i & 255] < 0 {
                    self.coord_168.pack168lo[i & 255] = at - self.coord_168.pack168hi[i >> 8];
                }
                at += 1;
            }
        }
    }

    fn fill_move_table(&mut self, coordinate_table: CoordinateTable, moves: &SearchGenerators) {
        // TODO: double-check if there are any performance penalties for `dyn`.
        let coord_field: &mut dyn Coord = match coordinate_table {
            CoordinateTable::Coord84 => &mut self.coord_84,
            CoordinateTable::Coord168 => &mut self.coord_168,
            CoordinateTable::CoordEP => &mut self.coord_ep,
        };
        {
            for row in coord_field.move_table() {
                row[0] = MOVE_TABLE_UNINITIALIZED_VALUE;
            }
        }
        let mut patterns: Vec<PackedKPattern> = Vec::new();
        patterns.push(match coordinate_table {
            CoordinateTable::CoordEP => self.packed_kpuzzle.default_pattern(),
            _ => cube4x4x4_phase2_target_pattern().clone(),
        });
        let mut patterns_read_idx = 0;
        let mut patterns_write_idx = 1;
        while patterns_read_idx < patterns_write_idx {
            let source_coordinate =
                coord_field.coordinate_for_pattern(&patterns[patterns_read_idx]);
            coord_field.move_table()[source_coordinate.0][0] = Phase2Coordinate(0);
            for (move_idx, m) in moves.flat.iter().enumerate() {
                let new_pattern = patterns[patterns_read_idx]
                    .clone()
                    .apply_transformation(&m.transformation);
                let destination_coordinate = coord_field.coordinate_for_pattern(&new_pattern);
                let move_table = coord_field.move_table();
                move_table[source_coordinate.0][move_idx] = destination_coordinate;
                if move_table[destination_coordinate.0][0] == MOVE_TABLE_UNINITIALIZED_VALUE {
                    move_table[destination_coordinate.0][0] = Phase2Coordinate(0);
                    patterns.push(new_pattern.clone());
                    patterns_write_idx += 1;
                }
                move_table[source_coordinate.0][move_idx] = destination_coordinate;
            }
            patterns_read_idx += 1;
        }

        let tab = coord_field.move_table();
        assert!(patterns_read_idx == tab.len());
        assert!(patterns_write_idx == tab.len());
    }

    pub(crate) fn init_move_tables(&mut self) {
        self.packed_kpuzzle = cube4x4x4_packed_kpuzzle();
        // TODO: deduplicate against earlier constant above
        let phase2_generators =
            generators_from_vec_str(vec!["Uw2", "U", "L", "F", "Rw", "R", "B", "Dw2", "D"]);
        match SearchGenerators::try_new(
            &self.packed_kpuzzle,
            &phase2_generators,
            &MetricEnum::Hand,
            false,
        ) {
            Result::Ok(moves) => {
                self.fill_move_table(CoordinateTable::Coord84, &moves);
                self.fill_move_table(CoordinateTable::Coord168, &moves);
                self.fill_move_table(CoordinateTable::CoordEP, &moves);
            }
            _ => {
                panic!();
            }
        }
    }
}
