use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    sync::Arc,
    time::Instant,
};

use cubing::kpuzzle::{InvalidAlgError, InvalidMoveError, KPattern, KPuzzle};

use crate::{
    _internal::{
        options::{Generators, MetricEnum},
        puzzle_traits::{MoveCount, SemiGroupActionPuzzle},
        Depth, FlatMoveIndex, IndexedVec, PatternValidityChecker, PruneTable, SearchGenerators,
    },
    scramble::randomize::BasicParity,
    whole_number_newtype,
};

use super::{
    definitions::{square1_square_square_shape_kpattern, square1_unbandaged_kpuzzle},
    mask_pattern::mask,
    square1::{wedge_parity, Phase1Checker},
};

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Square1Phase1Coordinates {
    masked_pattern: KPattern,
    parity: BasicParity,
}

impl Debug for Square1Phase1Coordinates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Phase1Coordinates")
            .field("masked_pattern", &self.masked_pattern.to_data())
            .field("parity", &self.parity)
            .finish()
    }
}

impl Square1Phase1Coordinates {
    pub fn try_new(full_pattern: &KPattern, phase_mask: &KPattern) -> Option<Self> {
        let Ok(masked_pattern) = mask(full_pattern, phase_mask) else {
            panic!("Mask application failed");
        };

        // TODO: this isn't a full validity check for scramble positions.
        if !Phase1Checker::is_valid(&masked_pattern) {
            return None;
        }

        let parity = wedge_parity(full_pattern);
        Some(Self {
            masked_pattern,
            parity,
        })
    }
}

whole_number_newtype!(PhaseCoordinatesIndex, usize);

#[derive(Debug)]
pub struct Square1Phase1LookupTableData {
    coordinates_to_index: HashMap<Square1Phase1Coordinates, PhaseCoordinatesIndex>,
    move_application_table:
        IndexedVec<PhaseCoordinatesIndex, IndexedVec<FlatMoveIndex, Option<PhaseCoordinatesIndex>>>,
    exact_prune_table: IndexedVec<PhaseCoordinatesIndex, Depth>,

    search_generators: SearchGenerators<KPuzzle>, // TODO: avoid `KPuzzle`

    // This is useful for testing and debugging.
    #[allow(unused)]
    pub index_to_coordinates: IndexedVec<PhaseCoordinatesIndex, Square1Phase1Coordinates>, // TODO: support optimizations when the size is known ahead of time
}

#[derive(Debug)]
pub struct Square1Phase1LookupTable {
    pub data: Arc<Square1Phase1LookupTableData>,
}

impl Square1Phase1LookupTable {
    pub fn apply_move(
        &self,
        phase_pattern_index: PhaseCoordinatesIndex,
        flat_move_index: FlatMoveIndex,
    ) -> Option<PhaseCoordinatesIndex> {
        *self
            .data
            .move_application_table
            .at(phase_pattern_index)
            .at(flat_move_index)
    }

    // TODO: report errors for invalid patterns
    pub fn full_pattern_to_coordinates(&self, kpattern: &KPattern) -> PhaseCoordinatesIndex {
        *self
            .data
            .coordinates_to_index
            .get(
                &Square1Phase1Coordinates::try_new(
                    kpattern,
                    square1_square_square_shape_kpattern(),
                )
                .unwrap(),
            )
            .unwrap()
    }
}

// TODO: turn into a static `new()` method.
pub fn build_phase1_lookup_table(
    kpuzzle: KPuzzle,
    generators: &Generators,
    phase_mask: &KPattern,
) -> (Square1Phase1LookupTable, SearchGenerators<KPuzzle>) {
    let start_time = Instant::now();
    let random_start = false; // TODO: for scrambles, we may want this to be true
    let search_generators = SearchGenerators::try_new(
        &kpuzzle,
        generators.enumerate_moves_for_kpuzzle(&kpuzzle),
        &MetricEnum::Hand,
        random_start,
    )
    .expect("Couldn't build SearchGenerators while building PhaseLookupTable");

    // (lookup pattern, depth)
    let mut fringe = VecDeque::<(KPattern, usize)>::new();
    fringe.push_back((kpuzzle.default_pattern(), 0));

    let mut index_to_coordinates =
        IndexedVec::<PhaseCoordinatesIndex, Square1Phase1Coordinates>::default();
    let mut coordinates_to_index =
        HashMap::<Square1Phase1Coordinates, PhaseCoordinatesIndex>::default();
    let mut exact_prune_table = IndexedVec::<PhaseCoordinatesIndex, Depth>::default();

    let mut index_to_representative_full_pattern =
        IndexedVec::<PhaseCoordinatesIndex, KPattern>::default();

    while let Some((full_pattern, depth)) = fringe.pop_front() {
        let Some(lookup_pattern) = Square1Phase1Coordinates::try_new(&full_pattern, phase_mask)
        else {
            continue;
        };

        if coordinates_to_index.contains_key(&lookup_pattern) {
            // TODO: consider avoiding putting things in the fringe that are already in the fringe
            // or lookup table.
            continue;
        }

        let index = index_to_coordinates.len();
        index_to_coordinates.push(lookup_pattern.clone());
        coordinates_to_index.insert(lookup_pattern, PhaseCoordinatesIndex(index));
        exact_prune_table.push(Depth(depth));

        for move_transformation_info in &search_generators.flat {
            fringe.push_back((
                full_pattern.apply_transformation(&move_transformation_info.transformation),
                depth + 1,
            ));
        }

        // Note that this is safe to do at the end of this loop because we use BFS rather than DFS.
        index_to_representative_full_pattern.push(full_pattern);
    }
    println!("PhaseLookupTable has size {}", index_to_coordinates.len());

    let mut move_application_table: IndexedVec<
        PhaseCoordinatesIndex,
        IndexedVec<FlatMoveIndex, Option<PhaseCoordinatesIndex>>,
    > = IndexedVec::default();
    for (phase_pattern_index, _) in index_to_coordinates.iter() {
        let representative = index_to_representative_full_pattern.at(phase_pattern_index);
        let mut table_row = IndexedVec::<FlatMoveIndex, Option<PhaseCoordinatesIndex>>::default();
        for move_transformation_info in &search_generators.flat {
            let new_representative =
                representative.apply_transformation(&move_transformation_info.transformation);
            let new_lookup_pattern =
                Square1Phase1Coordinates::try_new(&new_representative, phase_mask).map(
                    |new_lookup_pattern| {
                        coordinates_to_index
                            .get(&new_lookup_pattern)
                            .expect("Inconsistent pattern enumeration")
                    },
                );
            table_row.push(new_lookup_pattern.copied());
        }
        move_application_table.push(table_row);
    }

    println!(
        "Built phase lookup table in: {:?}",
        Instant::now() - start_time
    );

    // dbg!(exact_prune_table);

    let data = Arc::new(Square1Phase1LookupTableData {
        index_to_coordinates,
        coordinates_to_index,
        move_application_table,
        exact_prune_table,
        search_generators: search_generators.clone(),
    });
    (Square1Phase1LookupTable { data }, search_generators)
}

impl Clone for Square1Phase1LookupTable {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl SemiGroupActionPuzzle for Square1Phase1LookupTable {
    type Pattern = PhaseCoordinatesIndex;

    type Transformation = FlatMoveIndex;

    fn move_order(&self, r#move: &cubing::alg::Move) -> Result<MoveCount, InvalidAlgError> {
        square1_unbandaged_kpuzzle().move_order(r#move) // TODO
    }

    fn puzzle_transformation_from_move(
        &self,
        r#move: &cubing::alg::Move,
    ) -> Result<Self::Transformation, InvalidAlgError> {
        let Some(by_move) = self.data.search_generators.by_move.get(r#move) else {
            return Err(InvalidAlgError::InvalidMove(InvalidMoveError {
                description: format!("Invalid move: {}", r#move),
            }));
        };
        Ok(by_move.1.flat_move_index)
    }

    fn do_moves_commute(
        &self,
        move1_info: &crate::_internal::MoveTransformationInfo<Self>,
        move2_info: &crate::_internal::MoveTransformationInfo<Self>,
    ) -> bool {
        move1_info.r#move.quantum == move2_info.r#move.quantum
    }

    fn pattern_apply_transformation(
        // TODO: this is a hack to allow `Phase2Puzzle` to access its tables, ideally we would avoid this.
        // Then again, this might turn out to be necessary for similar high-performance implementations.
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Option<Self::Pattern> {
        println!("{:?}, {:?}", pattern, transformation_to_apply);
        self.apply_move(*pattern, *transformation_to_apply)
    }

    fn pattern_apply_transformation_into(
        // TODO: this is a hack to allow `Phase2Puzzle` to access its tables, ideally we would avoid this.
        // Then again, this might turn out to be necessary for similar high-performance implementations.
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    ) -> bool {
        let Some(pattern) = self.pattern_apply_transformation(pattern, transformation_to_apply)
        else {
            return false;
        };
        *into_pattern = pattern;
        true
    }
}

pub struct Square1Phase1PruneTable {
    tpuzzle: Square1Phase1LookupTable, // TODO: store just the prune table here
}

impl PruneTable<Square1Phase1LookupTable> for Square1Phase1PruneTable {
    fn new(
        tpuzzle: Square1Phase1LookupTable,
        _search_api_data: Arc<crate::_internal::IDFSearchAPIData<Square1Phase1LookupTable>>,
        _search_logger: Arc<crate::_internal::SearchLogger>,
        _min_size: Option<usize>,
    ) -> Self {
        Self { tpuzzle }
    }

    fn lookup(
        &self,
        coordinates: &<Square1Phase1LookupTable as SemiGroupActionPuzzle>::Pattern,
    ) -> Depth {
        *self.tpuzzle.data.exact_prune_table.at(*coordinates)
    }

    fn extend_for_search_depth(&mut self, _search_depth: Depth, _approximate_num_entries: usize) {
        // nothing
    }
}

#[cfg(test)]
mod tests {
    use cubing::alg::parse_move;

    use super::build_phase1_lookup_table;
    use crate::{
        _internal::FlatMoveIndex,
        scramble::{
            puzzles::{
                definitions::{square1_square_square_shape_kpattern, square1_unbandaged_kpuzzle},
                square1::wedge_parity,
                square1_phase1_lookup_table::{PhaseCoordinatesIndex, Square1Phase1Coordinates},
            },
            randomize::BasicParity,
            scramble_search::generators_from_vec_str,
        },
    };

    #[test]
    fn phase_lookup_table_test() {
        let kpuzzle = square1_unbandaged_kpuzzle();
        let generators = generators_from_vec_str(vec!["U_SQ_", "D_SQ_", "_SLASH_"]);

        let (phase_lookup_table, _search_generators) = build_phase1_lookup_table(
            kpuzzle.clone(),
            &generators,
            &square1_square_square_shape_kpattern().to_owned(),
        );
        let cube_pattern_index = PhaseCoordinatesIndex(0);

        #[allow(non_snake_case)]
        let U_SQ_move_index = FlatMoveIndex(0);

        #[allow(non_snake_case)]
        let U_SQ_pattern_index = phase_lookup_table
            .apply_move(cube_pattern_index, U_SQ_move_index)
            .unwrap();
        dbg!(U_SQ_pattern_index);

        let lookup_pattern = phase_lookup_table
            .data
            .index_to_coordinates
            .at(U_SQ_pattern_index);

        assert_eq!(
            &U_SQ_pattern_index,
            phase_lookup_table
                .data
                .coordinates_to_index
                .get(lookup_pattern)
                .unwrap(),
        );

        let other_pattern = kpuzzle
            .default_pattern()
            .apply_move(&parse_move!("U_SQ_1"))
            .unwrap();

        dbg!(&other_pattern);
        dbg!(wedge_parity(&other_pattern));
        assert_eq!(BasicParity::Odd, wedge_parity(&other_pattern));

        let other_lookup_pattern = &Square1Phase1Coordinates::try_new(
            &other_pattern,
            square1_square_square_shape_kpattern(),
        )
        .unwrap();

        assert_eq!(
            &U_SQ_pattern_index,
            phase_lookup_table
                .data
                .coordinates_to_index
                .get(other_lookup_pattern)
                .unwrap()
        );
    }
}
