use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::Arc,
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
        cli::args::MetricEnum,
        puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
        search::{
            indexed_vec::IndexedVec,
            move_count::MoveCount,
            prune_table_trait::{Depth, PruneTable},
        },
    },
    whole_number_newtype_generic,
};

pub trait DerivedPattern<TPuzzle: SemiGroupActionPuzzle>: Eq + Hash + Clone + Debug
where
    Self: std::marker::Sized,
{
    fn derived_pattern_name() -> &'static str; // TODO: signature
    fn try_new(puzzle: &TPuzzle, pattern: &TPuzzle::Pattern) -> Option<Self>;
}

whole_number_newtype_generic!(DerivedPatternIndex, usize, SemiGroupActionPuzzle);

// TODO: `U` should have the constraint `U: SemiGroupActionPuzzle`, but this cannot be enforced by the type checker.
pub type ExactDerivedPatternPruneTable<U> = IndexedVec<DerivedPatternIndex<U>, Depth>;

#[derive(Debug)]
pub struct DerivedPatternPuzzleTables<
    TPuzzle: SemiGroupActionPuzzle,
    TDerivedPattern: DerivedPattern<TPuzzle>,
> where
    GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern>: SemiGroupActionPuzzle,
{
    pub(crate) tpuzzle: TPuzzle,

    pub(crate) derived_pattern_to_index: HashMap<
        TDerivedPattern,
        DerivedPatternIndex<GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern>>,
    >,
    #[allow(clippy::type_complexity)] // Can this even be simplified?
    pub(crate) move_application_table: IndexedVec<
        DerivedPatternIndex<GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern>>,
        IndexedVec<
            FlatMoveIndex,
            Option<
                DerivedPatternIndex<GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern>>,
            >,
        >,
    >,
    pub(crate) exact_prune_table: ExactDerivedPatternPruneTable<
        GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern>,
    >,

    pub(crate) search_generators_for_tpuzzle: SearchGenerators<TPuzzle>, // TODO: avoid the need for this
    pub(crate) search_generators_for_derived_pattern_puzzle:
        SearchGenerators<GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern>>,

    // This is useful for testing and debugging.
    #[allow(unused)]
    pub(crate) index_to_derived_pattern: IndexedVec<
        DerivedPatternIndex<GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern>>,
        TDerivedPattern,
    >, // TODO: support optimizations when the size is known ahead of time

    pub(crate) phantom_data: PhantomData<TDerivedPattern>,
}

// TODO: Genericize this to `TPuzzle`.
#[derive(Clone, Debug)]
pub struct GraphEnumeratedDerivedPatternPuzzle<
    TPuzzle: SemiGroupActionPuzzle,
    TDerivedPattern: DerivedPattern<TPuzzle>,
> where
    Self: SemiGroupActionPuzzle,
{
    pub(crate) data: Arc<DerivedPatternPuzzleTables<TPuzzle, TDerivedPattern>>,
}

#[derive(Debug)]
pub enum DerivedPatternConversionError {
    InvalidDerivedPattern,
    InvalidDerivedPatternPuzzle,
}

impl<TPuzzle: SemiGroupActionPuzzle, TDerivedPattern: DerivedPattern<TPuzzle>>
    GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern>
where
    Self: SemiGroupActionPuzzle<Transformation = FlatMoveIndex>,
{
    pub fn new(
        puzzle: TPuzzle,
        start_pattern: TPuzzle::Pattern,
        generator_moves: Vec<Move>,
    ) -> Self {
        let random_start = false; // TODO: for scrambles, we may want this to be true
        let search_generators =
            SearchGenerators::try_new(&puzzle, generator_moves, &MetricEnum::Hand, random_start)
                .expect(
                    "Couldn't build SearchGenerators while building DerivedPatternPuzzlePuzzle",
                );

        let mut fringe = VecDeque::<(TPuzzle::Pattern, Depth)>::new();
        fringe.push_back((start_pattern, Depth(0)));

        let mut index_to_derived_pattern =
            IndexedVec::<DerivedPatternIndex<Self>, TDerivedPattern>::default();
        let mut derived_pattern_to_index =
            HashMap::<TDerivedPattern, DerivedPatternIndex<Self>>::default();
        let mut exact_prune_table = IndexedVec::<DerivedPatternIndex<Self>, Depth>::default();

        let mut index_to_representative_pattern =
            IndexedVec::<DerivedPatternIndex<Self>, TPuzzle::Pattern>::default();

        // TODO: Reuse `GodsAlgorithmTable` to enumerate patterns?
        while let Some((representative_pattern, depth)) = fringe.pop_front() {
            let Some(lookup_pattern) = TDerivedPattern::try_new(&puzzle, &representative_pattern)
            else {
                continue;
            };

            if derived_pattern_to_index.contains_key(&lookup_pattern) {
                // TODO: consider avoiding putting things in the fringe that are already in the fringe
                // or lookup table.
                continue;
            }

            let index = index_to_derived_pattern.len();
            index_to_derived_pattern.push(lookup_pattern.clone());
            derived_pattern_to_index.insert(lookup_pattern, DerivedPatternIndex::from(index));
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

        // TODO: place this behind `SearchLogger`.
        // eprintln!(
        //     "[DerivedPatternPuzzlePuzzle] {} has {} patterns.",
        //     TDerivedPattern::phase_name(),
        //     index_to_derived_pattern.len()
        // );

        let mut move_application_table: IndexedVec<
            DerivedPatternIndex<Self>,
            IndexedVec<FlatMoveIndex, Option<DerivedPatternIndex<Self>>>,
        > = IndexedVec::default();
        for (derived_pattern_index, _) in index_to_derived_pattern.iter() {
            let representative = &index_to_representative_pattern[derived_pattern_index];
            let mut table_row =
                IndexedVec::<FlatMoveIndex, Option<DerivedPatternIndex<Self>>>::default();
            for move_transformation_info in &search_generators.flat.0 {
                let new_lookup_pattern = match puzzle.pattern_apply_transformation(
                    representative,
                    &move_transformation_info.transformation,
                ) {
                    Some(new_representative) => {
                        TDerivedPattern::try_new(&puzzle, &new_representative)
                            .map(|new_lookup_pattern| {
                                derived_pattern_to_index
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

        // eprintln!(
        //     "Built derived pattern lookup table in: {:?}",
        //     Instant::now() - start_time
        // );

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

        let search_generators_for_derived_pattern_puzzle: SearchGenerators<
            GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern>,
        > = search_generators
            .transfer_move_classes::<Self>(puzzle_transformation_from_move)
            .unwrap();

        let data = Arc::new(DerivedPatternPuzzleTables::<TPuzzle, TDerivedPattern> {
            tpuzzle: puzzle,
            index_to_derived_pattern,
            derived_pattern_to_index,
            move_application_table,
            exact_prune_table,
            search_generators_for_tpuzzle: search_generators,
            search_generators_for_derived_pattern_puzzle,
            phantom_data: PhantomData,
        });

        Self { data }
    }

    // TODO: report errors for invalid patterns
    pub fn full_pattern_to_derived_pattern(
        &self,
        pattern: &TPuzzle::Pattern,
    ) -> Result<DerivedPatternIndex<Self>, DerivedPatternConversionError> {
        let Some(derived_pattern) = TDerivedPattern::try_new(&self.data.tpuzzle, pattern) else {
            return Err(DerivedPatternConversionError::InvalidDerivedPattern);
        };
        let Some(derived_pattern_index) = self.data.derived_pattern_to_index.get(&derived_pattern)
        else {
            return Err(DerivedPatternConversionError::InvalidDerivedPatternPuzzle);
        };
        Ok(*derived_pattern_index)
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

impl<TPuzzle: SemiGroupActionPuzzle, TDerivedPattern: DerivedPattern<TPuzzle>> SemiGroupActionPuzzle
    for GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern>
{
    type Pattern = DerivedPatternIndex<Self>;

    type Transformation = FlatMoveIndex;

    fn move_order(&self, r#move: &cubing::alg::Move) -> Result<MoveCount, InvalidAlgError> {
        self.data.tpuzzle.move_order(r#move)
    }

    fn puzzle_transformation_from_move(
        &self,
        r#move: &cubing::alg::Move,
    ) -> Result<Self::Transformation, InvalidAlgError> {
        puzzle_transformation_from_move(
            r#move,
            &self
                .data
                .search_generators_for_derived_pattern_puzzle
                .by_move,
        )
    }

    fn do_moves_commute(
        &self,
        move1_info: &MoveTransformationInfo<Self>,
        move2_info: &MoveTransformationInfo<Self>,
    ) -> bool {
        let move1_info = &self.data.search_generators_for_tpuzzle.flat[move1_info.flat_move_index];
        let move2_info = &self.data.search_generators_for_tpuzzle.flat[move2_info.flat_move_index];
        self.data.tpuzzle.do_moves_commute(move1_info, move2_info)
    }

    fn pattern_apply_transformation(
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Option<Self::Pattern> {
        self.data.move_application_table[*pattern][*transformation_to_apply]
    }

    fn pattern_apply_transformation_into(
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

pub struct DerivedPatternPuzzlePruneTable<
    TPuzzle: SemiGroupActionPuzzle,
    TDerivedPattern: DerivedPattern<TPuzzle>,
> {
    tpuzzle: GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern>, // TODO: store just the prune table here
}

impl<TPuzzle: SemiGroupActionPuzzle, TDerivedPattern: DerivedPattern<TPuzzle>>
    DerivedPatternPuzzlePruneTable<TPuzzle, TDerivedPattern>
{
    pub fn new(puzzle: GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern>) -> Self {
        Self { tpuzzle: puzzle }
    }
}

impl<TPuzzle: SemiGroupActionPuzzle, TDerivedPattern: DerivedPattern<TPuzzle>>
    PruneTable<GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern>>
    for DerivedPatternPuzzlePruneTable<TPuzzle, TDerivedPattern>
{
    fn lookup(
        &self,
        pattern: &<GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern> as SemiGroupActionPuzzle>::Pattern,
    ) -> Depth {
        self.tpuzzle.data.exact_prune_table[*pattern]
    }

    fn extend_for_search_depth(&mut self, _search_depth: Depth, _approximate_num_entries: usize) {
        // no-op
    }
}
