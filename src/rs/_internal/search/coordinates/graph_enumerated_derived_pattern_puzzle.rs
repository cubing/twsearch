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
        search::{indexed_vec::IndexedVec, move_count::MoveCount, prune_table_trait::Depth},
    },
    whole_number_newtype_generic,
};

use super::pattern_deriver::PatternDeriver;

whole_number_newtype_generic!(DerivedPatternIndex, usize, SemiGroupActionPuzzle);

// TODO: `U` should have the constraint `U: SemiGroupActionPuzzle`, but this cannot be enforced by the type checker.
pub type ExactDerivedPatternPruneTable<U> = IndexedVec<DerivedPatternIndex<U>, Depth>;

#[derive(Debug)]
pub struct DerivedPatternPuzzleTables<
    TSourcePuzzle: SemiGroupActionPuzzle,
    TPatternDeriver: PatternDeriver<TSourcePuzzle>,
> where
    TPatternDeriver::DerivedPattern: Hash,
{
    pub(crate) tpuzzle: TSourcePuzzle,

    pub(crate) pattern_deriver: TPatternDeriver,

    pub(crate) derived_pattern_to_index: HashMap<
        TPatternDeriver::DerivedPattern,
        DerivedPatternIndex<GraphEnumeratedDerivedPatternPuzzle<TSourcePuzzle, TPatternDeriver>>,
    >,
    #[allow(clippy::type_complexity)] // Can this even be simplified?
    pub(crate) move_application_table: IndexedVec<
        DerivedPatternIndex<GraphEnumeratedDerivedPatternPuzzle<TSourcePuzzle, TPatternDeriver>>,
        IndexedVec<
            FlatMoveIndex,
            Option<
                DerivedPatternIndex<
                    GraphEnumeratedDerivedPatternPuzzle<TSourcePuzzle, TPatternDeriver>,
                >,
            >,
        >,
    >,
    pub(crate) exact_prune_table: ExactDerivedPatternPruneTable<
        GraphEnumeratedDerivedPatternPuzzle<TSourcePuzzle, TPatternDeriver>,
    >,

    // TODO: avoid the need for this?
    pub(crate) search_generators_for_derived_pattern_puzzle:
        SearchGenerators<GraphEnumeratedDerivedPatternPuzzle<TSourcePuzzle, TPatternDeriver>>,

    // This is useful for testing and debugging.
    #[allow(unused)]
    pub(crate) index_to_derived_pattern: IndexedVec<
        DerivedPatternIndex<GraphEnumeratedDerivedPatternPuzzle<TSourcePuzzle, TPatternDeriver>>,
        TPatternDeriver::DerivedPattern,
    >, // TODO: support optimizations when the size is known ahead of time
}

/// T
#[derive(Clone, Debug)]
pub struct GraphEnumeratedDerivedPatternPuzzle<
    TSourcePuzzle: SemiGroupActionPuzzle,
    TPatternDeriver: PatternDeriver<TSourcePuzzle>,
> where
    Self: SemiGroupActionPuzzle,
    TPatternDeriver::DerivedPattern: Hash,
{
    pub(crate) data: Arc<DerivedPatternPuzzleTables<TSourcePuzzle, TPatternDeriver>>,
}

#[derive(Debug)]
pub enum DerivedPatternConversionError {
    InvalidDerivedPattern,
    InvalidDerivedPatternPuzzle,
}

impl<TSourcePuzzle: SemiGroupActionPuzzle, TPatternDeriver: PatternDeriver<TSourcePuzzle>>
    GraphEnumeratedDerivedPatternPuzzle<TSourcePuzzle, TPatternDeriver>
where
    Self: SemiGroupActionPuzzle<Transformation = FlatMoveIndex>,
    TPatternDeriver::DerivedPattern: Hash,
{
    pub fn new(
        puzzle: TSourcePuzzle,
        pattern_deriver: TPatternDeriver,
        start_pattern: <TSourcePuzzle as SemiGroupActionPuzzle>::Pattern,
        generator_moves: Vec<Move>,
    ) -> Self {
        let random_start = false; // TODO: for scrambles, we may want this to be true
        let search_generators =
            SearchGenerators::try_new(&puzzle, generator_moves, &MetricEnum::Hand, random_start)
                .expect(
                    "Couldn't build SearchGenerators while building DerivedPatternPuzzlePuzzle",
                );

        let mut fringe = VecDeque::<(TSourcePuzzle::Pattern, Depth)>::new();
        fringe.push_back((start_pattern, Depth(0)));

        let mut index_to_derived_pattern =
            IndexedVec::<DerivedPatternIndex<Self>, TPatternDeriver::DerivedPattern>::default();
        let mut derived_pattern_to_index =
            HashMap::<TPatternDeriver::DerivedPattern, DerivedPatternIndex<Self>>::default();
        let mut exact_prune_table = IndexedVec::<DerivedPatternIndex<Self>, Depth>::default();

        let mut index_to_representative_pattern =
            IndexedVec::<DerivedPatternIndex<Self>, TSourcePuzzle::Pattern>::default();

        // TODO: Reuse `GodsAlgorithmTable` to enumerate patterns?
        while let Some((representative_pattern, depth)) = fringe.pop_front() {
            let Some(lookup_pattern) = pattern_deriver.derive_pattern(&representative_pattern)
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
                    Some(new_representative) => pattern_deriver
                        .derive_pattern(&new_representative)
                        .map(|new_lookup_pattern| {
                            derived_pattern_to_index
                                .get(&new_lookup_pattern)
                                .expect("Inconsistent pattern enumeration")
                        })
                        .copied(),
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
        fn puzzle_transformation_from_move<TSourcePuzzle: SemiGroupActionPuzzle>(
            r#move: &cubing::alg::Move,
            by_move: &HashMap<Move, MoveTransformationInfo<TSourcePuzzle>>,
        ) -> Result<FlatMoveIndex, InvalidAlgError> {
            let Some(move_transformation_info) = by_move.get(r#move) else {
                return Err(InvalidAlgError::InvalidMove(InvalidMoveError {
                    description: format!("Invalid move: {}", r#move),
                }));
            };
            Ok(move_transformation_info.flat_move_index)
        }

        let search_generators_for_derived_pattern_puzzle: SearchGenerators<
            GraphEnumeratedDerivedPatternPuzzle<TSourcePuzzle, TPatternDeriver>,
        > = search_generators
            .transfer_move_classes::<Self>(puzzle_transformation_from_move)
            .unwrap();

        let data = Arc::new(
            DerivedPatternPuzzleTables::<TSourcePuzzle, TPatternDeriver> {
                tpuzzle: puzzle,
                pattern_deriver,
                index_to_derived_pattern,
                derived_pattern_to_index,
                move_application_table,
                exact_prune_table,
                search_generators_for_derived_pattern_puzzle,
            },
        );

        Self { data }
    }

    // TODO: report errors for invalid patterns
    pub fn full_pattern_to_derived_pattern(
        &self,
        pattern: &<TSourcePuzzle as SemiGroupActionPuzzle>::Pattern,
    ) -> Result<DerivedPatternIndex<Self>, DerivedPatternConversionError> {
        let Some(derived_pattern) = self.data.pattern_deriver.derive_pattern(pattern) else {
            return Err(DerivedPatternConversionError::InvalidDerivedPattern);
        };
        let Some(derived_pattern_index) = self.data.derived_pattern_to_index.get(&derived_pattern)
        else {
            return Err(DerivedPatternConversionError::InvalidDerivedPatternPuzzle);
        };
        Ok(*derived_pattern_index)
    }
}

fn puzzle_transformation_from_move<TSourcePuzzle: SemiGroupActionPuzzle>(
    r#move: &cubing::alg::Move,
    by_move: &HashMap<Move, MoveTransformationInfo<TSourcePuzzle>>,
) -> Result<FlatMoveIndex, InvalidAlgError> {
    let Some(move_transformation_info) = by_move.get(r#move) else {
        return Err(InvalidAlgError::InvalidMove(InvalidMoveError {
            description: format!("Invalid move: {}", r#move),
        }));
    };
    Ok(move_transformation_info.flat_move_index)
}

impl<TSourcePuzzle: SemiGroupActionPuzzle, TPatternDeriver: PatternDeriver<TSourcePuzzle>>
    SemiGroupActionPuzzle for GraphEnumeratedDerivedPatternPuzzle<TSourcePuzzle, TPatternDeriver>
where
    TPatternDeriver::DerivedPattern: Hash,
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

    fn do_moves_commute(&self, move1: &Move, move2: &Move) -> Result<bool, InvalidAlgError> {
        self.data.tpuzzle.do_moves_commute(move1, move2)
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
