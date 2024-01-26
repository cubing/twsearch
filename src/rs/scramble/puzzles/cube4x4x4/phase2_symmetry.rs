use std::{collections::HashMap, ops::Sub};

use crate::{
    _internal::{
        options::{CustomGenerators, Generators, MetricEnum},
        GenericPuzzleCore, MoveTransformationInfo, MoveTransformationMultiples,
        PruneTableEntryType, SearchGenerators,
    },
    scramble::puzzles::definitions::cube4x4x4_kpuzzle,
};

use cubing::{
    alg::{parse_move, AlgParseError, Move},
    kpuzzle::{InvalidAlgError, KPattern, KPuzzle, KTransformation},
};

use super::{
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
const MOVE_TABLE_UNINITIALIZED_VALUE: Phase2Coordinate = Phase2Coordinate(u32::MAX); // larger than any symcoord
const PACKED_VALUE_UNINITIALIZED_VALUE: PackedValue = PackedValue(u32::MAX);
const PRUNE_TABLE_UNINITIALIZED_VALUE: PruneTableEntryType = PruneTableEntryType::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Phase2Coordinate(u32);

impl Phase2Coordinate {
    pub fn usize(&self) -> usize {
        self.0 as usize
    }
}

impl Sub for Phase2Coordinate {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PackedValue(u32);

impl Sub for PackedValue {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl PackedValue {
    pub fn usize(&self) -> usize {
        self.0 as usize
    }
}

pub(crate) struct Phase1CoordinateTables {
    kpuzzle: KPuzzle,
    phase1_prune_table: [PruneTableEntryType; PHASE1_PRUNE_TABLE_SIZE],
    pack248hi: [PackedValue; 4096],
    pack248lo: [PackedValue; 4096],
}

impl Phase1CoordinateTables {
    fn initpack248(&mut self) {
        self.pack248hi = [PACKED_VALUE_UNINITIALIZED_VALUE; 4096];
        self.pack248lo = [PACKED_VALUE_UNINITIALIZED_VALUE; 4096];
        let mut i: usize = PRUNE_TABLE_UNINITIALIZED_VALUE as usize;
        let mut at = PACKED_VALUE_UNINITIALIZED_VALUE;
        while i < 0x1000000 {
            let b = i.count_ones();
            if b == 8 {
                if self.pack248hi[i >> 12] == PACKED_VALUE_UNINITIALIZED_VALUE {
                    self.pack248hi[i >> 12] = at;
                }
                if self.pack248lo[i & 0xfff] == PACKED_VALUE_UNINITIALIZED_VALUE {
                    self.pack248lo[i & 0xfff] = at - self.pack248hi[i >> 12];
                }
                at.0 += 1;
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

#[derive(Debug)]
pub(crate) struct Coord84 {
    pack84: [PackedValue; 256], // TODO: should this be unsigned?
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
        Phase2Coordinate(self.pack84[bits].0)
    }

    fn move_table(&mut self) -> &mut [[Phase2Coordinate; PHASE2_MOVE_COUNT]] {
        &mut self.c84move
    }
}

impl Default for Coord84 {
    fn default() -> Self {
        Self {
            pack84: [PackedValue(0); 256], // Note: this is *not* `PACKED_VALUE_UNINITIALIZED_VALUE`, because the table is filled in differently.
            c84move: [[Phase2Coordinate(0); PHASE2_MOVE_COUNT]; NUM_COORDINATES_C8_4D2],
        }
    }
}

#[derive(Debug)]
pub(crate) struct Coord168 {
    pack168hi: [PackedValue; 128],
    pack168lo: [PackedValue; 256],
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
        Phase2Coordinate(
            self.pack168hi[bits >> 8].0
                + self.pack168lo[bits & PRUNE_TABLE_UNINITIALIZED_VALUE as usize].0,
        )
    }

    fn move_table(&mut self) -> &mut [[Phase2Coordinate; PHASE2_MOVE_COUNT]] {
        &mut self.c168move
    }
}

impl Default for Coord168 {
    fn default() -> Self {
        Self {
            pack168hi: [PACKED_VALUE_UNINITIALIZED_VALUE; 128],
            pack168lo: [PACKED_VALUE_UNINITIALIZED_VALUE; 256],
            c168move: [[Phase2Coordinate(0); PHASE2_MOVE_COUNT]; NUM_COORDINATES_C16_8D2],
        }
    }
}

#[derive(Debug)]
pub(crate) struct CoordEP {
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
        Phase2Coordinate(r & 1)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Phase2CoordTuple {
    c84: Phase2Coordinate,
    c168: Phase2Coordinate,
    ep: Phase2Coordinate,
}

pub(crate) const PHASE2_SOLVED_STATE: Phase2CoordTuple = Phase2CoordTuple {
    c84: Phase2Coordinate(0),
    c168: Phase2Coordinate(0),
    ep: Phase2Coordinate(0),
};

impl Phase2CoordTuple {
    pub fn pack(&self) -> PackedValue {
        pack_coords(self.c84, self.c168, self.ep)
    }
}

fn pack_coords(c84: Phase2Coordinate, c168: Phase2Coordinate, ep: Phase2Coordinate) -> PackedValue {
    // TODO: check that `as u32` doesn't cost anything compared to having `u32` constant values.
    PackedValue(
        c84.0
            + (NUM_COORDINATES_C8_4D2 as u32) * (c168.0 + (NUM_COORDINATES_C16_8D2 as u32) * ep.0),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Phase2IndexedMove(pub usize);

#[derive(Debug)]
pub(crate) struct Phase2Puzzle {
    pub(crate) search_generators: SearchGenerators<Self>,
    pub(crate) move_to_transformation: HashMap<Move, Phase2IndexedMove>,
    pub(crate) transformation_to_move: HashMap<Phase2IndexedMove, Move>,
    pub(crate) coord_84: Coord84,
    pub(crate) coord_168: Coord168,
    pub(crate) coord_ep: CoordEP,
}

impl Phase2Puzzle {
    fn new() -> Self {
        let grouped_moves = vec![
            // Note: the first entry of each group must be the quantum move.
            vec![parse_move!("U"), parse_move!("U2"), parse_move!("U'")],
            vec![parse_move!("Uw2")],
            vec![parse_move!("L"), parse_move!("L2"), parse_move!("L'")],
            // no Lw moves
            vec![parse_move!("F"), parse_move!("F2"), parse_move!("F'")],
            // no Fw moves
            vec![parse_move!("R"), parse_move!("R2"), parse_move!("R'")],
            vec![parse_move!("Rw"), parse_move!("Rw2"), parse_move!("Rw'")],
            vec![parse_move!("B"), parse_move!("B2"), parse_move!("B'")],
            // no Bw moves
            vec![parse_move!("D"), parse_move!("D2"), parse_move!("D'")],
            vec![parse_move!("Dw2")],
        ];

        let mut grouped_multiples = Vec::<MoveTransformationMultiples<Self>>::default();
        let mut move_to_transformation = HashMap::<Move, Phase2IndexedMove>::default();
        let mut transformation_to_move = HashMap::<Phase2IndexedMove, Move>::default();

        let mut indexed_move: Phase2IndexedMove = Phase2IndexedMove(0);
        for group in grouped_moves {
            grouped_multiples.push(
                group
                    .into_iter()
                    .map(|r#move| {
                        move_to_transformation.insert(r#move.clone(), indexed_move);
                        transformation_to_move.insert(indexed_move, r#move.clone());
                        MoveTransformationInfo {
                            r#move,
                            transformation: indexed_move,
                        }
                    })
                    .collect(),
            );
            indexed_move.0 += 1
        }

        let search_generators = SearchGenerators::from_grouped(grouped_multiples, false);

        let coord_84 = Coord84::default();
        let coord_168 = Coord168::default();
        let coord_ep = CoordEP::default();

        Self {
            search_generators,
            move_to_transformation,
            transformation_to_move,
            coord_84,
            coord_168,
            coord_ep,
        }
    }

    pub(crate) fn coordinate_for_pattern(&self, pattern: &KPattern) -> Phase2CoordTuple {
        Phase2CoordTuple {
            c84: self.coord_84.coordinate_for_pattern(pattern),
            c168: self.coord_168.coordinate_for_pattern(pattern),
            ep: self.coord_ep.coordinate_for_pattern(pattern),
        }
    }
}

impl GenericPuzzleCore for Phase2Puzzle {
    type Pattern = Phase2CoordTuple;
    type Transformation = Phase2IndexedMove;

    fn puzzle_default_pattern(&self) -> Self::Pattern {
        PHASE2_SOLVED_STATE
    }

    fn puzzle_transformation_from_move(
        &self,
        r#move: &cubing::alg::Move,
    ) -> Result<Self::Transformation, cubing::kpuzzle::InvalidAlgError> {
        match self.move_to_transformation.get(r#move) {
            Some(transformation) => Ok(*transformation),
            None => Err(InvalidAlgError::AlgParse(
                // TODO: This should be an `InvalidMoveError`, but this is part of the `cubing::kpuzzle` interface even though it's not exported?
                AlgParseError {
                    description: format!("Move does not exist on this puzzle: {}", r#move),
                },
            )),
        }
    }

    fn pattern_apply_transformation(
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Self::Pattern {
        let c84 = self.coord_84.c84move[pattern.c84.usize()][transformation_to_apply.0];
        let c168 = self.coord_168.c168move[pattern.c168.usize()][transformation_to_apply.0];
        let ep = self.coord_ep.ep_move[pattern.ep.usize()][transformation_to_apply.0];
        Phase2CoordTuple { c84, c168, ep }
    }

    fn pattern_apply_transformation_into(
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    ) {
        into_pattern.c84 = self.coord_84.c84move[pattern.c84.usize()][transformation_to_apply.0];
        into_pattern.c168 =
            self.coord_168.c168move[pattern.c168.usize()][transformation_to_apply.0];
        into_pattern.ep = self.coord_ep.ep_move[pattern.ep.usize()][transformation_to_apply.0];
    }

    fn pattern_hash_u64(pattern: &Self::Pattern) -> u64 {
        pattern.pack().0 as u64
    }
}

pub(crate) struct Phase2SymmetryTables {
    pub(crate) phase2_puzzle: Phase2Puzzle,
    pub(crate) phase2_prune_table: [PruneTableEntryType; PHASE2_PRUNE_TABLE_SIZE],
}

impl Phase2SymmetryTables {
    pub(crate) fn new() -> Self {
        let mut phase2_symmetry_tables = Self {
            phase2_puzzle: Phase2Puzzle::new(),
            phase2_prune_table: [PRUNE_TABLE_UNINITIALIZED_VALUE; PHASE2_PRUNE_TABLE_SIZE],
        };
        phase2_symmetry_tables.init_choose_tables();
        phase2_symmetry_tables.init_move_tables();
        phase2_symmetry_tables.init_prune_table();
        phase2_symmetry_tables
    }

    pub(crate) fn init_choose_tables(&mut self) {
        let mut at = PackedValue(0);
        for i in 0..0x80usize {
            if i.count_ones() == 4 {
                self.phase2_puzzle.coord_84.pack84[i] = at;
                self.phase2_puzzle.coord_84.pack84[0xff - i] = at;
                at.0 += 1;
            }
        }
        at.0 = 0;
        for i in 0..0x8000usize {
            if i.count_ones() == 8 {
                if self.phase2_puzzle.coord_168.pack168hi[i >> 8]
                    == PACKED_VALUE_UNINITIALIZED_VALUE
                {
                    self.phase2_puzzle.coord_168.pack168hi[i >> 8] = at;
                }
                if self.phase2_puzzle.coord_168.pack168lo
                    [i & PRUNE_TABLE_UNINITIALIZED_VALUE as usize]
                    == PACKED_VALUE_UNINITIALIZED_VALUE
                {
                    self.phase2_puzzle.coord_168.pack168lo
                        [i & PRUNE_TABLE_UNINITIALIZED_VALUE as usize] =
                        at - self.phase2_puzzle.coord_168.pack168hi[i >> 8];
                }
                at.0 += 1;
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
        search_generators: &SearchGenerators<KPuzzle>,
    ) {
        // TODO: double-check if there are any performance penalties for `dyn`.
        let coord_field: &mut dyn Coord = match coordinate_table {
            CoordinateTable::Coord84 => &mut self.phase2_puzzle.coord_84,
            CoordinateTable::Coord168 => &mut self.phase2_puzzle.coord_168,
            CoordinateTable::CoordEP => &mut self.phase2_puzzle.coord_ep,
        };
        {
            for row in coord_field.move_table() {
                row[0] = MOVE_TABLE_UNINITIALIZED_VALUE;
            }
        }
        let mut patterns: Vec<KPattern> = Vec::new();
        patterns.push(cube4x4x4_kpuzzle().default_pattern());
        let mut patterns_read_idx = 0;
        let mut patterns_write_idx = 1;
        while patterns_read_idx < patterns_write_idx {
            let source_coordinate =
                coord_field.coordinate_for_pattern(&patterns[patterns_read_idx]);
            coord_field.move_table()[source_coordinate.usize()][0] = Phase2Coordinate(0);
            for (move_idx, m) in search_generators.flat.iter().enumerate() {
                let new_pattern = patterns[patterns_read_idx]
                    .clone()
                    .apply_transformation(&m.transformation);
                let destination_coordinate = coord_field.coordinate_for_pattern(&new_pattern);
                let move_table = coord_field.move_table();
                move_table[source_coordinate.usize()][move_idx] = destination_coordinate;
                if move_table[destination_coordinate.usize()][0] == MOVE_TABLE_UNINITIALIZED_VALUE {
                    move_table[destination_coordinate.usize()][0] = Phase2Coordinate(0);
                    patterns.push(new_pattern.clone());
                    patterns_write_idx += 1;
                }
                move_table[source_coordinate.usize()][move_idx] = destination_coordinate;
            }
            patterns_read_idx += 1;
        }

        let tab = coord_field.move_table();
        assert!(patterns_read_idx == tab.len());
        assert!(patterns_write_idx == tab.len());
    }

    pub(crate) fn init_move_tables(&mut self) {
        let phase2_generators = Generators::Custom(CustomGenerators {
            moves: self
                .phase2_puzzle
                .search_generators
                .grouped
                .iter()
                .map(|group| group[0].r#move.clone())
                .collect(),
            algs: vec![],
        });
        // TODO: deduplicate this with external code.
        match SearchGenerators::try_new(
            cube4x4x4_kpuzzle(),
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

    pub(crate) fn init_prune_table(&mut self) {
        for i in 0..self.phase2_prune_table.len() {
            self.phase2_prune_table[i] = PRUNE_TABLE_UNINITIALIZED_VALUE;
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
                c84 = Phase2Coordinate(PRUNE_TABLE_UNINITIALIZED_VALUE as u32) - c84;
            }
            self.phase2_prune_table
                [pack_coords(c84, Phase2Coordinate(0), Phase2Coordinate(0)).usize()] = 0;
        }
        for d in 0..PRUNE_TABLE_UNINITIALIZED_VALUE {
            let mut written = 0;
            for epsrc in 0..NUM_COORDINATES_EP {
                for c168src in 0..NUM_COORDINATES_C16_8D2 {
                    for c84src in 0..NUM_COORDINATES_C8_4D2 {
                        if self.phase2_prune_table[pack_coords(
                            Phase2Coordinate(c84src as u32),
                            Phase2Coordinate(c168src as u32),
                            Phase2Coordinate(epsrc as u32),
                        )
                        .usize()]
                            == d
                        {
                            for m in 0..PHASE2_MOVE_COUNT {
                                let dst = pack_coords(
                                    self.phase2_puzzle.coord_84.c84move[c84src][m],
                                    self.phase2_puzzle.coord_168.c168move[c168src][m],
                                    self.phase2_puzzle.coord_ep.ep_move[epsrc][m],
                                );
                                if self.phase2_prune_table[dst.usize()]
                                    == PRUNE_TABLE_UNINITIALIZED_VALUE
                                {
                                    self.phase2_prune_table[dst.usize()] = d + 1;
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
