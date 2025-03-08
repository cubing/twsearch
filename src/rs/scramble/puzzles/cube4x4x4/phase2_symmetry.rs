// use std::{collections::HashMap, ops::Sub, sync::Arc};

// use crate::{
//     _internal::{
//         options::{CustomGenerators, Generators, MetricEnum},
//         GenericPuzzleCore, MoveTransformationInfo, MoveTransformationMultiples,
//         PruneTableEntryType, SearchGenerators, SearchHeuristic,
//     },
//     scramble::puzzles::{definitions::cube4x4x4_kpuzzle, super_pattern::super_pattern},
// };

// use cubing::{
//     alg::{parse_move, AlgParseError, Move},
//     kpuzzle::{InvalidAlgError, KPattern, KPuzzle, KTransformation},
// };

// use super::orbit_info::orbit_info;

// const PHASE1_PRUNE_TABLE_SIZE: usize = 735471; // this is 24 choose 8
// const PHASE1_MOVE_COUNT: usize = 33;
const NUM_COORDINATES_C8_4D2: usize = 35;
// const NUM_COORDINATES_C16_8D2: usize = 6435;
// const NUM_COORDINATES_EP: usize = 2;
// const PHASE2_PRUNE_TABLE_SIZE: usize =
//     NUM_COORDINATES_C8_4D2 * NUM_COORDINATES_C16_8D2 * NUM_COORDINATES_EP;
const PHASE2_MOVE_COUNT: usize = 23;
// const MOVE_TABLE_UNINITIALIZED_VALUE: Phase2Coordinate = Phase2Coordinate(u32::MAX); // larger than any symcoord
// const PACKED_VALUE_UNINITIALIZED_VALUE: PackedValue = PackedValue(u32::MAX);
// const PRUNE_TABLE_UNINITIALIZED_VALUE: PruneTableEntryType = PruneTableEntryType::MAX;

// // TODO: change the 4x4x4 Speffz def to have indistinguishable centers and get rid of this.
// #[derive(Debug, Clone, Copy, PartialEq)]
// pub(crate) enum SideCenter {
//     L,
//     R,
// }

// pub(crate) const PHASE2_SOLVED_SIDE_CENTER_CASES: [[[SideCenter; 4]; 2]; 12] = [
//     // flat faces
//     [
//         [SideCenter::L, SideCenter::L, SideCenter::L, SideCenter::L],
//         [SideCenter::R, SideCenter::R, SideCenter::R, SideCenter::R],
//     ],
//     [
//         [SideCenter::R, SideCenter::R, SideCenter::R, SideCenter::R],
//         [SideCenter::L, SideCenter::L, SideCenter::L, SideCenter::L],
//     ],
//     // horizontal bars
//     [
//         [SideCenter::L, SideCenter::L, SideCenter::R, SideCenter::R],
//         [SideCenter::L, SideCenter::L, SideCenter::R, SideCenter::R],
//     ],
//     [
//         [SideCenter::R, SideCenter::R, SideCenter::L, SideCenter::L],
//         [SideCenter::R, SideCenter::R, SideCenter::L, SideCenter::L],
//     ],
//     [
//         [SideCenter::R, SideCenter::R, SideCenter::L, SideCenter::L],
//         [SideCenter::L, SideCenter::L, SideCenter::R, SideCenter::R],
//     ],
//     [
//         [SideCenter::L, SideCenter::L, SideCenter::R, SideCenter::R],
//         [SideCenter::R, SideCenter::R, SideCenter::L, SideCenter::L],
//     ],
//     // vertical bars
//     [
//         [SideCenter::L, SideCenter::R, SideCenter::R, SideCenter::L],
//         [SideCenter::L, SideCenter::R, SideCenter::R, SideCenter::L],
//     ],
//     [
//         [SideCenter::R, SideCenter::L, SideCenter::L, SideCenter::R],
//         [SideCenter::R, SideCenter::L, SideCenter::L, SideCenter::R],
//     ],
//     [
//         [SideCenter::L, SideCenter::R, SideCenter::R, SideCenter::L],
//         [SideCenter::R, SideCenter::L, SideCenter::L, SideCenter::R],
//     ],
//     [
//         [SideCenter::R, SideCenter::L, SideCenter::L, SideCenter::R],
//         [SideCenter::L, SideCenter::R, SideCenter::R, SideCenter::L],
//     ],
//     // checkerboards
//     [
//         [SideCenter::L, SideCenter::R, SideCenter::L, SideCenter::R],
//         [SideCenter::L, SideCenter::R, SideCenter::L, SideCenter::R],
//     ],
//     [
//         [SideCenter::R, SideCenter::L, SideCenter::R, SideCenter::L],
//         [SideCenter::R, SideCenter::L, SideCenter::R, SideCenter::L],
//     ],
// ];
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

use std::ops::Sub;

use crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle;

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

// #[derive(Clone, Copy, Debug)]
// enum CoordinateTable {
//     Coord84,
//     Coord168,
//     CoordEP,
// }

// trait Coord {
//     fn coordinate_for_pattern(&self, pattern: &KPattern) -> Phase2Coordinate;
//     fn move_table(&mut self) -> &mut [[Phase2Coordinate; PHASE2_MOVE_COUNT]];
// }

#[derive(Debug)]
pub(crate) struct Coord84 {
    pack84: [PackedValue; 256], // TODO: should this be unsigned?
    c84move: [[Phase2Coordinate; PHASE2_MOVE_COUNT]; NUM_COORDINATES_C8_4D2],
}

impl SemiGroupActionPuzzle for Coord84 {
    type Pattern;
    type Transformation;

    fn move_order(
        &self,
        r#move: &cubing::alg::Move,
    ) -> Result<crate::_internal::search::move_count::MoveCount, cubing::kpuzzle::InvalidAlgError>
    {
        todo!()
    }

    fn puzzle_transformation_from_move(
        &self,
        r#move: &cubing::alg::Move,
    ) -> Result<Self::Transformation, cubing::kpuzzle::InvalidAlgError> {
        todo!()
    }

    fn do_moves_commute(
        &self,
        move1_info: &crate::_internal::canonical_fsm::search_generators::MoveTransformationInfo<
            Self,
        >,
        move2_info: &crate::_internal::canonical_fsm::search_generators::MoveTransformationInfo<
            Self,
        >,
    ) -> bool {
        todo!()
    }

    fn pattern_apply_transformation(
        // TODO: this is a hack to allow `Phase2Puzzle` to access its tables, ideally we would avoid this.
        // Then again, this might turn out to be necessary for similar high-performance implementations.
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Option<Self::Pattern> {
        todo!()
    }

    fn pattern_apply_transformation_into(
        // TODO: this is a hack to allow `Phase2Puzzle` to access its tables, ideally we would avoid this.
        // Then again, this might turn out to be necessary for similar high-performance implementations.
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    ) -> bool {
        todo!()
    }
}

// const L_AND_R_CENTER_INDICES: [u8; 8] = [4, 5, 6, 7, 12, 13, 14, 15];
// const R_LOWEST_CENTER_PIECE_INDEX: u8 = 8;

// impl Coord for Coord84 {
//     fn coordinate_for_pattern(&self, pattern: &KPattern) -> Phase2Coordinate {
//         let mut bits = 0;
//         let mut sum: u32 = 0;
//         // TODO: store this in the struct?
//         let centers_orbit_info = orbit_info(pattern.kpuzzle(), 2, "CENTERS");
//         for idx in L_AND_R_CENTER_INDICES {
//             sum += pattern.get_piece(centers_orbit_info, idx) as u32;
//             bits *= 2;
//             if pattern.get_piece(centers_orbit_info, idx) < R_LOWEST_CENTER_PIECE_INDEX {
//                 bits += 1
//             }
//         }
//         if sum != 76 {
//             dbg!(sum);
//             panic!("Called coord84 on the wrong kind of kpuzzle");
//         }
//         Phase2Coordinate(self.pack84[bits].0)
//     }

//     fn move_table(&mut self) -> &mut [[Phase2Coordinate; PHASE2_MOVE_COUNT]] {
//         &mut self.c84move
//     }
// }

// impl Default for Coord84 {
//     fn default() -> Self {
//         Self {
//             pack84: [PackedValue(0); 256], // Note: this is *not* `PACKED_VALUE_UNINITIALIZED_VALUE`, because the table is filled in differently.
//             c84move: [[Phase2Coordinate(0); PHASE2_MOVE_COUNT]; NUM_COORDINATES_C8_4D2],
//         }
//     }
// }

// #[derive(Debug)]
// pub(crate) struct Coord168 {
//     pack168hi: [PackedValue; 128],
//     pack168lo: [PackedValue; 256],
//     c168move: [[Phase2Coordinate; PHASE2_MOVE_COUNT]; NUM_COORDINATES_C16_8D2],
// }

// impl Coord for Coord168 {
//     fn coordinate_for_pattern(&self, pattern: &KPattern) -> Phase2Coordinate {
//         let mut bits = 0;
//         // TODO: store this in the struct?
//         let centers_orbit_info = orbit_info(pattern.kpuzzle(), 2, "CENTERS");
//         let mut sum: u32 = 0;
//         for idx in [
//             0, 1, 2, 3, 20, 21, 22, 23, // U and D
//             8, 9, 10, 11, 16, 17, 18, 19, // F and B
//         ] {
//             sum += pattern.get_piece(centers_orbit_info, idx) as u32;
//             bits *= 2;
//             let piece = pattern.get_piece(centers_orbit_info, idx);
//             if !(4..20).contains(&piece) {
//                 bits += 1
//             }
//         }
//         if bits >= 32768 {
//             bits = 65535 - bits;
//         }
//         if sum != 200 {
//             panic!("Called coord168 on the wrong kind of kpuzzle");
//         }
//         Phase2Coordinate(
//             self.pack168hi[bits >> 8].0
//                 + self.pack168lo[bits & PRUNE_TABLE_UNINITIALIZED_VALUE as usize].0,
//         )
//     }

//     fn move_table(&mut self) -> &mut [[Phase2Coordinate; PHASE2_MOVE_COUNT]] {
//         &mut self.c168move
//     }
// }

// impl Default for Coord168 {
//     fn default() -> Self {
//         Self {
//             pack168hi: [PACKED_VALUE_UNINITIALIZED_VALUE; 128],
//             pack168lo: [PACKED_VALUE_UNINITIALIZED_VALUE; 256],
//             c168move: [[Phase2Coordinate(0); PHASE2_MOVE_COUNT]; NUM_COORDINATES_C16_8D2],
//         }
//     }
// }

// #[derive(Debug)]
// pub(crate) struct CoordEP {
//     ep_move: [[Phase2Coordinate; PHASE2_MOVE_COUNT]; NUM_COORDINATES_EP],
// }

// impl Coord for CoordEP {
//     fn coordinate_for_pattern(&self, pattern: &KPattern) -> Phase2Coordinate {
//         let mut bits = 0;
//         let mut r = 0;
//         let mut sum: u32 = 0;
//         // TODO: store this in the struct?
//         let edges_orbit_info = orbit_info(pattern.kpuzzle(), 1, "WINGS");
//         for idx in 0..24u8 {
//             sum += pattern.get_piece(edges_orbit_info, idx) as u32;
//             if ((bits >> idx) & 1) == 0 {
//                 let mut cyclen = 0;
//                 let mut j: u8 = idx;
//                 while ((bits >> j) & 1) == 0 {
//                     cyclen += 1;
//                     bits |= 1 << j;
//                     j = pattern.get_piece(edges_orbit_info, j);
//                 }
//                 r += cyclen + 1;
//             }
//         }
//         if sum != 276 {
//             panic!("Coord for coordep called on bad kpuzzle type");
//         }
//         Phase2Coordinate(r & 1)
//     }

//     fn move_table(&mut self) -> &mut [[Phase2Coordinate; PHASE2_MOVE_COUNT]] {
//         &mut self.ep_move
//     }
// }

// impl Default for CoordEP {
//     fn default() -> Self {
//         Self {
//             ep_move: [[Phase2Coordinate(0); PHASE2_MOVE_COUNT]; NUM_COORDINATES_EP],
//         }
//     }
// }

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub(crate) struct Phase2CoordTuple {
//     c84: Phase2Coordinate,
//     c168: Phase2Coordinate,
//     ep: Phase2Coordinate,
// }

// pub(crate) const PHASE2_SOLVED_STATE: Phase2CoordTuple = Phase2CoordTuple {
//     c84: Phase2Coordinate(0),
//     c168: Phase2Coordinate(0),
//     ep: Phase2Coordinate(0),
// };

// impl Phase2CoordTuple {
//     pub fn pack(&self) -> PackedValue {
//         pack_coords(self.c84, self.c168, self.ep)
//     }
// }

// fn pack_coords(c84: Phase2Coordinate, c168: Phase2Coordinate, ep: Phase2Coordinate) -> PackedValue {
//     // TODO: check that `as u32` doesn't cost anything compared to having `u32` constant values.
//     PackedValue(
//         c84.0
//             + (NUM_COORDINATES_C8_4D2 as u32) * (c168.0 + (NUM_COORDINATES_C16_8D2 as u32) * ep.0),
//     )
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub(crate) struct Phase2IndexedTransformation(pub usize);

// #[derive(Debug)]
// pub(crate) struct Phase2PuzzleData {
//     pub(crate) search_generators: SearchGenerators<Phase2Puzzle>,
//     pub(crate) move_to_transformation: HashMap<Move, Phase2IndexedTransformation>,
//     pub(crate) transformation_to_move: HashMap<Phase2IndexedTransformation, Move>,
//     pub(crate) coord_84: Coord84,
//     pub(crate) coord_168: Coord168,
//     pub(crate) coord_ep: CoordEP,
// }

// impl Phase2PuzzleData {
//     fn new() -> Self {
//         let grouped_moves = vec![
//             // Note: the first entry of each group must be the quantum move.
//             vec![parse_move!("U"), parse_move!("U2"), parse_move!("U'")],
//             vec![parse_move!("Uw2")],
//             vec![parse_move!("L"), parse_move!("L2"), parse_move!("L'")],
//             // no Lw moves
//             vec![parse_move!("F"), parse_move!("F2"), parse_move!("F'")],
//             // no Fw moves
//             vec![parse_move!("R"), parse_move!("R2"), parse_move!("R'")],
//             vec![parse_move!("Rw"), parse_move!("Rw2"), parse_move!("Rw'")],
//             vec![parse_move!("B"), parse_move!("B2"), parse_move!("B'")],
//             // no Bw moves
//             vec![parse_move!("D"), parse_move!("D2"), parse_move!("D'")],
//             vec![parse_move!("Dw2")],
//         ];

//         let mut grouped_multiples = Vec::<MoveTransformationMultiples<Phase2Puzzle>>::default();
//         let mut move_to_transformation = HashMap::<Move, Phase2IndexedTransformation>::default();
//         let mut transformation_to_move: HashMap<Phase2IndexedTransformation, Move> =
//             HashMap::<Phase2IndexedTransformation, Move>::default();

//         let mut indexed_move: Phase2IndexedTransformation = Phase2IndexedTransformation(0);
//         for group in grouped_moves {
//             grouped_multiples.push(
//                 group
//                     .into_iter()
//                     .map(|r#move| {
//                         move_to_transformation.insert(r#move.clone(), indexed_move);
//                         transformation_to_move.insert(indexed_move, r#move.clone());
//                         let move_transformation_info = MoveTransformationInfo {
//                             r#move,
//                             transformation: indexed_move,
//                         };
//                         indexed_move.0 += 1;
//                         move_transformation_info
//                     })
//                     .collect(),
//             );
//         }

//         let search_generators = SearchGenerators::from_grouped(grouped_multiples, false);

//         let coord_84 = Coord84::default();
//         let coord_168 = Coord168::default();
//         let coord_ep = CoordEP::default();

//         Self {
//             search_generators,
//             move_to_transformation,
//             transformation_to_move,
//             coord_84,
//             coord_168,
//             coord_ep,
//         }
//     }
// }

// #[derive(Clone, Debug)]
// pub(crate) struct Phase2Puzzle {
//     pub(crate) data: Arc<Phase2PuzzleData>,
// }

// impl Phase2Puzzle {
//     pub(crate) fn coordinate_for_pattern(&self, pattern: &KPattern) -> Phase2CoordTuple {
//         Phase2CoordTuple {
//             c84: self.data.coord_84.coordinate_for_pattern(pattern),
//             c168: self.data.coord_168.coordinate_for_pattern(pattern),
//             ep: self.data.coord_ep.coordinate_for_pattern(pattern),
//         }
//     }
// }

// impl GenericPuzzleCore for Phase2Puzzle {
//     type Pattern = Phase2CoordTuple;
//     type Transformation = Phase2IndexedTransformation;

//     fn puzzle_default_pattern(&self) -> Self::Pattern {
//         PHASE2_SOLVED_STATE
//     }

//     fn puzzle_transformation_from_move(
//         &self,
//         r#move: &cubing::alg::Move,
//     ) -> Result<Self::Transformation, cubing::kpuzzle::InvalidAlgError> {
//         match self.data.move_to_transformation.get(r#move) {
//             Some(transformation) => Ok(*transformation),
//             None => Err(InvalidAlgError::AlgParse(
//                 // TODO: This should be an `InvalidMoveError`, but this is part of the `cubing::kpuzzle` interface even though it's not exported?
//                 AlgParseError {
//                     description: format!("Move does not exist on this puzzle: {}", r#move),
//                 },
//             )),
//         }
//     }

//     fn pattern_apply_transformation(
//         &self,
//         pattern: &Self::Pattern,
//         transformation_to_apply: &Self::Transformation,
//     ) -> Self::Pattern {
//         let c84 = self.data.coord_84.c84move[pattern.c84.usize()][transformation_to_apply.0];
//         let c168 = self.data.coord_168.c168move[pattern.c168.usize()][transformation_to_apply.0];
//         let ep = self.data.coord_ep.ep_move[pattern.ep.usize()][transformation_to_apply.0];
//         Phase2CoordTuple { c84, c168, ep }
//     }

//     fn pattern_apply_transformation_into(
//         &self,
//         pattern: &Self::Pattern,
//         transformation_to_apply: &Self::Transformation,
//         into_pattern: &mut Self::Pattern,
//     ) {
//         into_pattern.c84 =
//             self.data.coord_84.c84move[pattern.c84.usize()][transformation_to_apply.0];
//         into_pattern.c168 =
//             self.data.coord_168.c168move[pattern.c168.usize()][transformation_to_apply.0];
//         into_pattern.ep = self.data.coord_ep.ep_move[pattern.ep.usize()][transformation_to_apply.0];
//     }

//     fn pattern_hash_u64(pattern: &Self::Pattern) -> u64 {
//         pattern.pack().0 as u64
//     }
// }

// pub(crate) struct Phase2SymmetryTables {
//     pub(crate) phase2_prune_table: [PruneTableEntryType; PHASE2_PRUNE_TABLE_SIZE],
// }

// impl SearchHeuristic<Phase2Puzzle> for Phase2SymmetryTables {
//     fn extend_for_search_depth(&mut self, search_depth: usize, _approximate_num_entries: usize) {
//         // no-op
//         eprintln!(
//             "No extension needed for extend_for_search_depth({}, …)",
//             search_depth
//         );
//     }

//     fn lookup(&self, pattern: &<Phase2Puzzle as GenericPuzzleCore>::Pattern) -> usize {
//         self.phase2_prune_table[pattern.pack().usize()] as usize
//     }
// }

// impl Phase2SymmetryTables {
//     pub(crate) fn initialize() -> (Self, Phase2Puzzle) {
//         let mut phase2_puzzle_data = Phase2PuzzleData::new();
//         let mut phase2_symmetry_tables = Self {
//             phase2_prune_table: [PRUNE_TABLE_UNINITIALIZED_VALUE; PHASE2_PRUNE_TABLE_SIZE],
//         };
//         phase2_symmetry_tables.init_choose_tables(&mut phase2_puzzle_data);
//         phase2_symmetry_tables.init_move_tables(&mut phase2_puzzle_data);
//         phase2_symmetry_tables.init_prune_table(&mut phase2_puzzle_data);
//         (
//             phase2_symmetry_tables,
//             Phase2Puzzle {
//                 data: Arc::new(phase2_puzzle_data),
//             },
//         )
//     }

//     pub(crate) fn init_choose_tables(&mut self, phase2_puzzle_data: &mut Phase2PuzzleData) {
//         let mut at = PackedValue(0);
//         for i in 0..0x80usize {
//             if i.count_ones() == 4 {
//                 phase2_puzzle_data.coord_84.pack84[i] = at;
//                 phase2_puzzle_data.coord_84.pack84[0xff - i] = at;
//                 at.0 += 1;
//             }
//         }
//         at.0 = 0;
//         for i in 0..0x8000usize {
//             if i.count_ones() == 8 {
//                 if phase2_puzzle_data.coord_168.pack168hi[i >> 8]
//                     == PACKED_VALUE_UNINITIALIZED_VALUE
//                 {
//                     phase2_puzzle_data.coord_168.pack168hi[i >> 8] = at;
//                 }
//                 if phase2_puzzle_data.coord_168.pack168lo
//                     [i & PRUNE_TABLE_UNINITIALIZED_VALUE as usize]
//                     == PACKED_VALUE_UNINITIALIZED_VALUE
//                 {
//                     phase2_puzzle_data.coord_168.pack168lo
//                         [i & PRUNE_TABLE_UNINITIALIZED_VALUE as usize] =
//                         at - phase2_puzzle_data.coord_168.pack168hi[i >> 8];
//                 }
//                 at.0 += 1;
//             }
//         }
//     }

//     // TODO: Remove
//     #[allow(dead_code)]
//     fn show_pattern(pattern: &KPattern) {
//         dbg!(pattern.to_data());
//     }

//     // TODO: Remove
//     #[allow(dead_code)]
//     fn show_transformation(transformation: &KTransformation) {
//         dbg!(transformation.to_data());
//     }

//     fn fill_move_table(
//         &mut self,
//         phase2_puzzle_data: &mut Phase2PuzzleData,
//         coordinate_table: CoordinateTable,
//         search_generators: &SearchGenerators<KPuzzle>,
//     ) {
//         // TODO: double-check if there are any performance penalties for `dyn`.
//         let coord_field: &mut dyn Coord = match coordinate_table {
//             CoordinateTable::Coord84 => &mut phase2_puzzle_data.coord_84,
//             CoordinateTable::Coord168 => &mut phase2_puzzle_data.coord_168,
//             CoordinateTable::CoordEP => &mut phase2_puzzle_data.coord_ep,
//         };
//         {
//             for row in coord_field.move_table() {
//                 row[0] = MOVE_TABLE_UNINITIALIZED_VALUE;
//             }
//         }
//         let mut patterns: Vec<KPattern> = Vec::new();
//         patterns.push(super_pattern(cube4x4x4_kpuzzle()));
//         let mut patterns_read_idx = 0;
//         let mut patterns_write_idx = 1;
//         while patterns_read_idx < patterns_write_idx {
//             let source_coordinate =
//                 coord_field.coordinate_for_pattern(&patterns[patterns_read_idx]);
//             coord_field.move_table()[source_coordinate.usize()][0] = Phase2Coordinate(0);
//             for (move_idx, m) in search_generators.flat.iter().enumerate() {
//                 let new_pattern = patterns[patterns_read_idx]
//                     .clone()
//                     .apply_transformation(&m.transformation);
//                 let destination_coordinate = coord_field.coordinate_for_pattern(&new_pattern);
//                 let move_table = coord_field.move_table();
//                 move_table[source_coordinate.usize()][move_idx] = destination_coordinate;
//                 if move_table[destination_coordinate.usize()][0] == MOVE_TABLE_UNINITIALIZED_VALUE {
//                     move_table[destination_coordinate.usize()][0] = Phase2Coordinate(0);
//                     patterns.push(new_pattern.clone());
//                     patterns_write_idx += 1;
//                 }
//                 move_table[source_coordinate.usize()][move_idx] = destination_coordinate;
//             }
//             patterns_read_idx += 1;
//         }

//         let tab = coord_field.move_table();
//         assert!(patterns_read_idx == tab.len());
//         assert!(patterns_write_idx == tab.len());
//     }

//     pub(crate) fn init_move_tables(&mut self, phase2_puzzle_data: &mut Phase2PuzzleData) {
//         let phase2_generators = Generators::Custom(CustomGenerators {
//             moves: phase2_puzzle_data
//                 .search_generators
//                 .grouped
//                 .iter()
//                 .map(|group| group[0].r#move.clone())
//                 .collect(),
//             algs: vec![],
//         });
//         match SearchGenerators::try_new(
//             cube4x4x4_kpuzzle(),
//             &phase2_generators,
//             &MetricEnum::Hand,
//             false,
//             false,
//         ) {
//             Result::Ok(search_generators) => {
//                 self.fill_move_table(
//                     phase2_puzzle_data,
//                     CoordinateTable::Coord84,
//                     &search_generators,
//                 );
//                 self.fill_move_table(
//                     phase2_puzzle_data,
//                     CoordinateTable::Coord168,
//                     &search_generators,
//                 );
//                 self.fill_move_table(
//                     phase2_puzzle_data,
//                     CoordinateTable::CoordEP,
//                     &search_generators,
//                 );
//             }
//             _ => {
//                 panic!();
//             }
//         }
//     }

//     pub(crate) fn init_prune_table(&mut self, phase2_puzzle_data: &mut Phase2PuzzleData) {
//         for i in 0..self.phase2_prune_table.len() {
//             self.phase2_prune_table[i] = PRUNE_TABLE_UNINITIALIZED_VALUE;
//         }
//         for sol in PHASE2_SOLVED_SIDE_CENTER_CASES {
//             let mut c84: Phase2Coordinate = Phase2Coordinate(0);
//             for i1 in sol {
//                 for i2 in i1 {
//                     c84.0 *= 2;
//                     if i2 == SideCenter::L {
//                         c84.0 += 1;
//                     }
//                 }
//             }
//             if c84.0 > 127 {
//                 c84 = Phase2Coordinate(PRUNE_TABLE_UNINITIALIZED_VALUE as u32) - c84;
//             }
//             let c84 = Phase2Coordinate(phase2_puzzle_data.coord_84.pack84[c84.0 as usize].0);
//             self.phase2_prune_table
//                 [pack_coords(c84, Phase2Coordinate(0), Phase2Coordinate(0)).usize()] = 0;
//         }
//         for d in 0..PRUNE_TABLE_UNINITIALIZED_VALUE {
//             let mut written = 0;
//             for epsrc in 0..NUM_COORDINATES_EP {
//                 for c168src in 0..NUM_COORDINATES_C16_8D2 {
//                     for c84src in 0..NUM_COORDINATES_C8_4D2 {
//                         if self.phase2_prune_table[pack_coords(
//                             Phase2Coordinate(c84src as u32),
//                             Phase2Coordinate(c168src as u32),
//                             Phase2Coordinate(epsrc as u32),
//                         )
//                         .usize()]
//                             == d
//                         {
//                             for m in 0..PHASE2_MOVE_COUNT {
//                                 let dst = pack_coords(
//                                     phase2_puzzle_data.coord_84.c84move[c84src][m],
//                                     phase2_puzzle_data.coord_168.c168move[c168src][m],
//                                     phase2_puzzle_data.coord_ep.ep_move[epsrc][m],
//                                 );
//                                 if self.phase2_prune_table[dst.usize()]
//                                     == PRUNE_TABLE_UNINITIALIZED_VALUE
//                                 {
//                                     self.phase2_prune_table[dst.usize()] = d + 1;
//                                     written += 1;
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//             if written == 0 {
//                 break;
//             }
//         }
//     }
// }
