use crate::{
    _internal::{options::MetricEnum, PruneTableEntryType, SearchGenerators},
    scramble::puzzles::definitions::{
        cube4x4x4_kpuzzle, cube4x4x4_phase2_target_kpattern, cube4x4x4_with_wing_parity_kpuzzle,
    },
};

use cubing::kpuzzle::{KPattern, KPuzzle, KTransformation};

use super::{
    super::super::scramble_search::generators_from_vec_str,
    orbit_info::orbit_info,
    phase2::{SideCenter, PHASE2_SOLVED_SIDE_CENTER_CASES},
};

const PHASE1_PRUNE_TABLE_SIZE: usize = 735471; // this is 24 choose 8
const PHASE1_MOVE_COUNT: usize = 33;
const NUM_COORDINATES_C8_4D2: usize = 35;
const NUM_COORDINATES_C16_8D2: usize = 6435;
const NUM_COORDINATES_EP: usize = 2;
const PHASE2_PRUNE_TABLE_SIZE: usize =
    NUM_COORDINATES_C8_4D2 * NUM_COORDINATES_C16_8D2 * NUM_COORDINATES_EP;
const PHASE2_MOVE_COUNT: usize = 23;
const MOVE_TABLE_UNINITIALIZED_VALUE: Phase2Coordinate = Phase2Coordinate(usize::MAX); // larger than any symcoord

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Phase2Coordinate(usize);

pub(crate) struct Phase1CoordinateTables {
    kpuzzle: KPuzzle,
    phase1_prune_table: [PruneTableEntryType; PHASE1_PRUNE_TABLE_SIZE],
    pack248hi: [i32; 4096],
    pack248lo: [i32; 4096],
    movelo: [[u32; PHASE1_MOVE_COUNT]; 4096],
    movehi: [[u32; PHASE1_MOVE_COUNT]; 4096],
}

impl Phase1CoordinateTables {
    fn initpack248(&mut self) {
        self.pack248hi = [-1; 4096];
        self.pack248lo = [-1; 4096];
        let mut i: usize = 255;
        let mut at = 0;
        while i < 0x1000000 {
            let b = bit_count(i);
            if b == 8 {
                if self.pack248hi[i >> 12] < 0 {
                    self.pack248hi[i >> 12] = at;
                }
                if self.pack248lo[i & 0xfff] < 0 {
                    self.pack248lo[i & 0xfff] = at - self.pack248hi[i >> 12];
                }
                at += 1;
            }
            // we increment i this way so we don't spin 2^24 times
            if b >= 8 {
                i += i & ((!i) + 1); // want i & -i but can't - to a usize
            } else {
                i += (!i) & (i + 1);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum CoordinateTable {
    Coord84,
    Coord168,
    CoordEP,
}

trait Coord {
    fn coordinate_for_pattern(&self, pattern: &KPattern) -> Phase2Coordinate;
    fn move_table(&mut self) -> &mut [[Phase2Coordinate; PHASE2_MOVE_COUNT]];
}

struct Coord84 {
    pack84: [i32; 256], // TODO: should this be unsigned?
    c84move: [[Phase2Coordinate; PHASE2_MOVE_COUNT]; NUM_COORDINATES_C8_4D2],
}

const L_AND_R_CENTER_INDICES: [u8; 8] = [4, 5, 6, 7, 12, 13, 14, 15];
const L_CENTER_PIECE: u8 = 1;

impl Coord for Coord84 {
    fn coordinate_for_pattern(&self, pattern: &KPattern) -> Phase2Coordinate {
        let mut bits = 0;
        // TODO: store this in the struct?
        let centers_orbit_info = orbit_info(pattern.kpuzzle(), 2, "CENTERS");
        for idx in L_AND_R_CENTER_INDICES {
            bits *= 2;
            if pattern.get_piece(centers_orbit_info, idx) == L_CENTER_PIECE {
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
    pack168hi: [i32; 128],
    pack168lo: [i32; 256],
    c168move: [[Phase2Coordinate; PHASE2_MOVE_COUNT]; NUM_COORDINATES_C16_8D2],
}

impl Coord for Coord168 {
    fn coordinate_for_pattern(&self, pattern: &KPattern) -> Phase2Coordinate {
        let mut bits = 0;
        // TODO: store this in the struct?
        let centers_orbit_info = orbit_info(pattern.kpuzzle(), 2, "CENTERS");
        for idx in [0, 1, 2, 3, 8, 9, 10, 11, 16, 17, 18, 19, 20, 21, 22, 23] {
            bits *= 2;
            if pattern.get_piece(centers_orbit_info, idx) == 0 {
                bits += 1
            }
        }
        if bits >= 32768 {
            bits = 65535 - bits;
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
            pack168hi: [-1; 128],
            pack168lo: [-1; 256],
            c168move: [[Phase2Coordinate(0); PHASE2_MOVE_COUNT]; NUM_COORDINATES_C16_8D2],
        }
    }
}

struct CoordEP {
    ep_move: [[Phase2Coordinate; PHASE2_MOVE_COUNT]; NUM_COORDINATES_EP],
}

impl Coord for CoordEP {
    fn coordinate_for_pattern(&self, pattern: &KPattern) -> Phase2Coordinate {
        let mut bits = 0;
        let mut r = 0;
        // TODO: store this in the struct?
        let edges_orbit_info = orbit_info(pattern.kpuzzle(), 1, "WINGS");
        for idx in 0..24u8 {
            if ((bits >> idx) & 1) == 0 {
                let mut cyclen = 0;
                let mut j: u8 = idx;
                while ((bits >> j) & 1) == 0 {
                    cyclen += 1;
                    bits |= 1 << j;
                    j = pattern.get_piece(edges_orbit_info, j);
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
    kpuzzle: KPuzzle,
    phase2_prune_table: [PruneTableEntryType; PHASE2_PRUNE_TABLE_SIZE],
    coord_84: Coord84,
    coord_168: Coord168,
    coord_ep: CoordEP,
}

const PRUNE_TABLE_UNINITIALIZED_VALUE: PruneTableEntryType = PruneTableEntryType::MAX;

pub(crate) struct Phase2CoordSet {
    c84: Phase2Coordinate, c168: Phase2Coordinate, ep: Phase2Coordinate,
}

pub(crate) const Phase2SolvedState: Phase2CoordSet =
       Phase2CoordSet {c84: Phase2Coordinate(0), c168: Phase2Coordinate(0), ep: Phase2Coordinate(0)};

impl Phase2SymmetryTables {
    pub(crate) fn new(kpuzzle: KPuzzle) -> Self {
        Self {
            kpuzzle,
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
        at = 0;
        for i in 0..0x8000 {
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

    // TODO: Remove
    #[allow(dead_code)]
    fn show_pattern(pattern: &KPattern) {
        dbg!(pattern.to_data());
    }

    // TODO: Remove
    #[allow(dead_code)]
    fn show_transformation(transformation: &KTransformation) {
        dbg!(transformation.to_data());
    }

    fn fill_move_table(
        &mut self,
        coordinate_table: CoordinateTable,
        moves: &SearchGenerators<KPuzzle>,
    ) {
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
        let mut patterns: Vec<KPattern> = Vec::new();
        patterns.push(match coordinate_table {
            CoordinateTable::CoordEP => cube4x4x4_with_wing_parity_kpuzzle().default_pattern(),
            _ => cube4x4x4_phase2_target_kpattern().clone(),
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
        self.kpuzzle = cube4x4x4_kpuzzle().clone();
        // TODO: deduplicate against earlier constant above
        let phase2_generators =
            generators_from_vec_str(vec!["Uw2", "U", "L", "F", "Rw", "R", "B", "Dw2", "D"]);
        match SearchGenerators::try_new(
            cube4x4x4_with_wing_parity_kpuzzle(),
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

    fn pack_coords(c84: Phase2Coordinate, c168: Phase2Coordinate, ep: Phase2Coordinate) -> usize {
        c84.0 + NUM_COORDINATES_C8_4D2 * (c168.0 + NUM_COORDINATES_C16_8D2 * ep.0)
    }

    pub(crate) fn init_prune_table(&mut self) {
        for i in 0..self.phase2_prune_table.len() {
            self.phase2_prune_table[i] = 255;
        }
        for sol in PHASE2_SOLVED_SIDE_CENTER_CASES {
            let mut c84: Phase2Coordinate = Phase2Coordinate(0);
            for i1 in sol {
                for i2 in i1 {
                    c84.0 *= 2;
                    if i2 == SideCenter::L {
                        c84.0 += 1;
                    }
                }
            }
            if c84.0 > 127 {
                c84.0 = 255 - c84.0;
            }
            self.phase2_prune_table
                [Self::pack_coords(c84, Phase2Coordinate(0), Phase2Coordinate(0))] = 0;
        }
        for d in 0..255 {
            let mut written = 0;
            for epsrc in 0..NUM_COORDINATES_EP {
                for c168src in 0..NUM_COORDINATES_C16_8D2 {
                    for c84src in 0..NUM_COORDINATES_C8_4D2 {
                        if self.phase2_prune_table[Self::pack_coords(
                            Phase2Coordinate(c84src),
                            Phase2Coordinate(c168src),
                            Phase2Coordinate(epsrc),
                        )] == d
                        {
                            for m in 0..PHASE2_MOVE_COUNT {
                                let dst = Self::pack_coords(
                                    self.coord_84.c84move[c84src][m],
                                    self.coord_168.c168move[c168src][m],
                                    self.coord_ep.ep_move[epsrc][m],
                                );
                                if self.phase2_prune_table[dst] == 255 {
                                    self.phase2_prune_table[dst] = d + 1;
                                    written += 1;
                                }
                            }
                        }
                    }
                }
            }
            if written == 0 {
                break;
            }
        }
    }
}
