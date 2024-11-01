use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::Arc,
    time::Instant,
};

use cubing::{
    alg::Move,
    kpuzzle::{InvalidAlgError, InvalidMoveError},
};

use crate::{
    _internal::{
        canonical_fsm::search_generators::{
            FlatMoveIndex, MoveTransformationInfo, SearchGenerators,
        },
        cli::options_impl::MetricEnum,
        puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
        search::{
            check_pattern::AlwaysValid,
            idf_search::{DefaultSearchOptimizations, IDFSearchAPIData, SearchOptimizations},
            indexed_vec::IndexedVec,
            move_count::MoveCount,
            prune_table_trait::{Depth, PruneTable},
            search_logger::SearchLogger,
        },
    },
    whole_number_newtype,
};

pub trait SemanticCoordinate<TPuzzle: SemiGroupActionPuzzle>: Eq + Hash + Clone + Debug
where
    Self: std::marker::Sized,
{
    fn try_new(puzzle: &TPuzzle, pattern: &TPuzzle::Pattern) -> Option<Self>;
}

whole_number_newtype!(PhaseCoordinateIndex, usize);

pub type ExactCoordinatePruneTable = IndexedVec<PhaseCoordinateIndex, Depth>;

#[derive(Debug)]
pub struct PhaseCoordinateLookupTables<
    TPuzzle: SemiGroupActionPuzzle,
    TSemanticCoordinate: SemanticCoordinate<TPuzzle>,
> where
    PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate>: SemiGroupActionPuzzle,
{
    pub(crate) puzzle: TPuzzle,

    pub(crate) semantic_coordinate_to_index: HashMap<TSemanticCoordinate, PhaseCoordinateIndex>,
    pub(crate) move_application_table:
        IndexedVec<PhaseCoordinateIndex, IndexedVec<FlatMoveIndex, Option<PhaseCoordinateIndex>>>,
    pub(crate) exact_prune_table: ExactCoordinatePruneTable,

    pub(crate) search_generators:
        SearchGenerators<PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate>>,

    // This is useful for testing and debugging.
    #[allow(unused)]
    pub(crate) index_to_semantic_coordinate: IndexedVec<PhaseCoordinateIndex, TSemanticCoordinate>, // TODO: support optimizations when the size is known ahead of time

    pub(crate) phantom_data: PhantomData<TSemanticCoordinate>,
}

// TODO: Genericize this to `TPuzzle`.
#[derive(Clone, Debug)]
pub struct PhaseCoordinatePuzzle<
    TPuzzle: SemiGroupActionPuzzle,
    TSemanticCoordinate: SemanticCoordinate<TPuzzle>,
> where
    Self: SemiGroupActionPuzzle,
{
    pub(crate) data: Arc<PhaseCoordinateLookupTables<TPuzzle, TSemanticCoordinate>>,
}

impl<TPuzzle: SemiGroupActionPuzzle, TSemanticCoordinate: SemanticCoordinate<TPuzzle>>
    PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate>
where
    Self: SemiGroupActionPuzzle<Transformation = FlatMoveIndex>,
{
    pub fn new(
        puzzle: TPuzzle,
        start_pattern: TPuzzle::Pattern,
        generator_moves: Vec<Move>,
    ) -> Self {
        let start_time = Instant::now();

        let random_start = false; // TODO: for scrambles, we may want this to be true
        let search_generators =
            SearchGenerators::try_new(&puzzle, generator_moves, &MetricEnum::Hand, random_start)
                .expect("Couldn't build SearchGenerators while building PhaseLookupTable");

        let mut fringe = VecDeque::<(TPuzzle::Pattern, Depth)>::new();
        fringe.push_back((start_pattern, Depth(0)));

        let mut index_to_semantic_coordinate =
            IndexedVec::<PhaseCoordinateIndex, TSemanticCoordinate>::default();
        let mut semantic_coordinate_to_index =
            HashMap::<TSemanticCoordinate, PhaseCoordinateIndex>::default();
        let mut exact_prune_table = IndexedVec::<PhaseCoordinateIndex, Depth>::default();

        let mut index_to_representative_pattern =
            IndexedVec::<PhaseCoordinateIndex, TPuzzle::Pattern>::default();

        while let Some((representative_pattern, depth)) = fringe.pop_front() {
            let Some(lookup_pattern) =
                TSemanticCoordinate::try_new(&puzzle, &representative_pattern)
            else {
                continue;
            };

            if semantic_coordinate_to_index.contains_key(&lookup_pattern) {
                // TODO: consider avoiding putting things in the fringe that are already in the fringe
                // or lookup table.
                continue;
            }

            let index = index_to_semantic_coordinate.len();
            index_to_semantic_coordinate.push(lookup_pattern.clone());
            semantic_coordinate_to_index.insert(lookup_pattern, PhaseCoordinateIndex(index));
            exact_prune_table.push(depth);

            for move_transformation_info in &search_generators.flat.0 {
                let Some(new_pattern) = puzzle.pattern_apply_transformation(
                    &representative_pattern,
                    &move_transformation_info.transformation,
                ) else {
                    continue;
                };
                fringe.push_back((new_pattern, depth + Depth(1)));
            }

            // Note that this is safe to do at the end of this loop because we use BFS rather than DFS.
            index_to_representative_pattern.push(representative_pattern);
        }
        eprintln!(
            "PhaseLookupTable has size {}",
            index_to_semantic_coordinate.len()
        );

        let mut move_application_table: IndexedVec<
            PhaseCoordinateIndex,
            IndexedVec<FlatMoveIndex, Option<PhaseCoordinateIndex>>,
        > = IndexedVec::default();
        for (phase_pattern_index, _) in index_to_semantic_coordinate.iter() {
            let representative = index_to_representative_pattern.at(phase_pattern_index);
            let mut table_row =
                IndexedVec::<FlatMoveIndex, Option<PhaseCoordinateIndex>>::default();
            for move_transformation_info in &search_generators.flat.0 {
                let new_lookup_pattern = match puzzle.pattern_apply_transformation(
                    representative,
                    &move_transformation_info.transformation,
                ) {
                    Some(new_representative) => {
                        TSemanticCoordinate::try_new(&puzzle, &new_representative)
                            .map(|new_lookup_pattern| {
                                semantic_coordinate_to_index
                                    .get(&new_lookup_pattern)
                                    .expect("Inconsistent pattern enumeration")
                            })
                            .copied()
                    }
                    None => None,
                };
                table_row.push(new_lookup_pattern);
            }
            move_application_table.push(table_row);
        }

        eprintln!(
            "Built phase lookup table in: {:?}",
            Instant::now() - start_time
        );

        // TODO: Why can't we reuse the static `puzzle_transformation_from_move`?
        // TODO: come up with a cleaner way for `SearchGenerators` to share the same move classes.
        fn puzzle_transformation_from_move<TPuzzle: SemiGroupActionPuzzle>(
            r#move: &cubing::alg::Move,
            by_move: &HashMap<Move, MoveTransformationInfo<TPuzzle>>,
        ) -> Result<FlatMoveIndex, InvalidAlgError> {
            let Some(move_transformation_info) = by_move.get(r#move) else {
                return Err(InvalidAlgError::InvalidMove(InvalidMoveError {
                    description: format!("Invalid move: {}", r#move),
                }));
            };
            Ok(move_transformation_info.flat_move_index)
        }

        let search_generators: SearchGenerators<
            PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate>,
        > = search_generators
            .transfer_move_classes::<Self>(puzzle_transformation_from_move)
            .unwrap();

        let data = Arc::new(
            PhaseCoordinateLookupTables::<TPuzzle, TSemanticCoordinate> {
                puzzle,
                index_to_semantic_coordinate,
                semantic_coordinate_to_index,
                move_application_table,
                exact_prune_table,
                search_generators,
                phantom_data: PhantomData,
            },
        );
        Self { data }
    }

    // TODO: report errors for invalid patterns
    pub fn full_pattern_to_phase_coordinate(
        &self,
        pattern: &TPuzzle::Pattern,
    ) -> PhaseCoordinateIndex {
        *self
            .data
            .semantic_coordinate_to_index
            .get(&TSemanticCoordinate::try_new(&self.data.puzzle, pattern).unwrap())
            .unwrap()
    }
}

fn puzzle_transformation_from_move<
    TPuzzle: SemiGroupActionPuzzle<Transformation = FlatMoveIndex>,
>(
    r#move: &cubing::alg::Move,
    by_move: &HashMap<Move, MoveTransformationInfo<TPuzzle>>,
) -> Result<FlatMoveIndex, InvalidAlgError> {
    let Some(move_transformation_info) = by_move.get(r#move) else {
        return Err(InvalidAlgError::InvalidMove(InvalidMoveError {
            description: format!("Invalid move: {}", r#move),
        }));
    };
    Ok(move_transformation_info.flat_move_index)
}

impl<TPuzzle: SemiGroupActionPuzzle, TSemanticCoordinate: SemanticCoordinate<TPuzzle>>
    SemiGroupActionPuzzle for PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate>
{
    type Pattern = PhaseCoordinateIndex;

    type Transformation = FlatMoveIndex;

    fn move_order(&self, r#move: &cubing::alg::Move) -> Result<MoveCount, InvalidAlgError> {
        self.data.puzzle.move_order(r#move)
    }

    fn puzzle_transformation_from_move(
        &self,
        r#move: &cubing::alg::Move,
    ) -> Result<Self::Transformation, InvalidAlgError> {
        puzzle_transformation_from_move(r#move, &self.data.search_generators.by_move)
    }

    fn do_moves_commute(
        &self,
        move1_info: &MoveTransformationInfo<Self>,
        move2_info: &MoveTransformationInfo<Self>,
    ) -> bool {
        // TODO: Place this in a trait for the semantic coordinate or
        eprintln!("Using a naïve move commutation heuristic");
        move1_info.r#move.quantum == move2_info.r#move.quantum
    }

    fn pattern_apply_transformation(
        // TODO: this is a hack to allow `Phase2Puzzle` to access its tables, ideally we would avoid this.
        // Then again, this might turn out to be necessary for similar high-performance implementations.
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Option<Self::Pattern> {
        *self
            .data
            .move_application_table
            .at(*pattern)
            .at(*transformation_to_apply)
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

pub struct PhaseCoordinatePruneTable<
    TPuzzle: SemiGroupActionPuzzle,
    TSemanticCoordinate: SemanticCoordinate<TPuzzle>,
> {
    tpuzzle: PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate>, // TODO: store just the prune table here
}

impl<TPuzzle: SemiGroupActionPuzzle, TSemanticCoordinate: SemanticCoordinate<TPuzzle>>
    PruneTable<PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate>>
    for PhaseCoordinatePruneTable<TPuzzle, TSemanticCoordinate>
{
    fn new(
        puzzle: PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate>,
        _search_api_data: Arc<
            IDFSearchAPIData<PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate>>,
        >,
        _search_logger: Arc<SearchLogger>,
        _min_size: Option<usize>,
    ) -> Self {
        Self { tpuzzle: puzzle }
    }

    fn lookup(
        &self,
        pattern: &<PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate> as SemiGroupActionPuzzle>::Pattern,
    ) -> Depth {
        *self.tpuzzle.data.exact_prune_table.at(*pattern)
    }

    fn extend_for_search_depth(&mut self, _search_depth: Depth, _approximate_num_entries: usize) {
        // no-op
    }
}

// TODO: simplify the default for below.
pub struct PhaseCoordinateLookupSearchOptimizations<
    TPuzzle: SemiGroupActionPuzzle,
    TSemanticCoordinate: SemanticCoordinate<TPuzzle>,
> {
    phantom_data: PhantomData<(TPuzzle, TSemanticCoordinate)>,
}

impl<TPuzzle: SemiGroupActionPuzzle, TSemanticCoordinate: SemanticCoordinate<TPuzzle>>
    SearchOptimizations<PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate>>
    for PhaseCoordinateLookupSearchOptimizations<TPuzzle, TSemanticCoordinate>
{
    type PatternValidityChecker = AlwaysValid; // TODO: reconcile this with fallible transformation application.
    type PruneTable = PhaseCoordinatePruneTable<TPuzzle, TSemanticCoordinate>;
}

impl<TPuzzle: SemiGroupActionPuzzle, TSemanticCoordinate: SemanticCoordinate<TPuzzle>>
    DefaultSearchOptimizations<PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate>>
    for PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate>
{
    type Optimizations = PhaseCoordinateLookupSearchOptimizations<TPuzzle, TSemanticCoordinate>;
}
