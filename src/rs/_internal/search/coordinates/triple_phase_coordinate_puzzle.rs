use std::{cmp::max, sync::Arc};

use cubing::{alg::Move, kpuzzle::InvalidAlgError};

use crate::_internal::{
    canonical_fsm::search_generators::{FlatMoveIndex, MoveTransformationInfo},
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::{
        iterative_deepening::{
            iterative_deepening_search::IterativeDeepeningSearchAPIData,
            search_adaptations::SearchAdaptationsWithoutPruneTable,
        },
        move_count::MoveCount,
        prune_table_trait::{Depth, LegacyConstructablePruneTable, PruneTable},
        search_logger::SearchLogger,
    },
};

use super::graph_enumerated_derived_pattern_puzzle::{
    DerivedPattern, DerivedPatternConversionError, DerivedPatternIndex,
    GraphEnumeratedDerivedPatternPuzzle,
};

// TODO: modify the `DoublePhaseCoordinate` implementation so that `TriplePhaseCoordinate` can nest it.

#[derive(Clone, Debug)]
pub struct TriplePhaseCoordinate<
    TPuzzle: SemiGroupActionPuzzle,
    TDerivedPattern1: DerivedPattern<TPuzzle>,
    TDerivedPattern2: DerivedPattern<TPuzzle>,
    TDerivedPattern3: DerivedPattern<TPuzzle>,
> {
    pub coordinate1:
        DerivedPatternIndex<GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern1>>,
    pub coordinate2:
        DerivedPatternIndex<GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern2>>,
    pub coordinate3:
        DerivedPatternIndex<GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern3>>,
}

// TODO: why can't this be derived?
impl<
        TPuzzle: SemiGroupActionPuzzle,
        TDerivedPattern1: DerivedPattern<TPuzzle>,
        TDerivedPattern2: DerivedPattern<TPuzzle>,
        TDerivedPattern3: DerivedPattern<TPuzzle>,
    > PartialEq
    for TriplePhaseCoordinate<TPuzzle, TDerivedPattern1, TDerivedPattern2, TDerivedPattern3>
{
    fn eq(&self, other: &Self) -> bool {
        self.coordinate1 == other.coordinate1
            && self.coordinate2 == other.coordinate2
            && self.coordinate3 == other.coordinate3
    }
}

// TODO: why can't this be derived?
impl<
        TPuzzle: SemiGroupActionPuzzle,
        TDerivedPattern1: DerivedPattern<TPuzzle>,
        TDerivedPattern2: DerivedPattern<TPuzzle>,
        TDerivedPattern3: DerivedPattern<TPuzzle>,
    > Eq for TriplePhaseCoordinate<TPuzzle, TDerivedPattern1, TDerivedPattern2, TDerivedPattern3>
{
}

#[derive(Clone, Debug)]
pub struct TriplePhaseCoordinatePuzzleData<
    TPuzzle: SemiGroupActionPuzzle,
    TDerivedPattern1: DerivedPattern<TPuzzle>,
    TDerivedPattern2: DerivedPattern<TPuzzle>,
    TDerivedPattern3: DerivedPattern<TPuzzle>,
> {
    pub puzzle1: GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern1>,
    pub puzzle2: GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern2>,
    pub puzzle3: GraphEnumeratedDerivedPatternPuzzle<TPuzzle, TDerivedPattern3>,
}

#[derive(Clone, Debug)]
pub struct TriplePhaseCoordinatePuzzle<
    TPuzzle: SemiGroupActionPuzzle,
    TDerivedPattern1: DerivedPattern<TPuzzle>,
    TDerivedPattern2: DerivedPattern<TPuzzle>,
    TDerivedPattern3: DerivedPattern<TPuzzle>,
> {
    pub data: Arc<
        TriplePhaseCoordinatePuzzleData<
            TPuzzle,
            TDerivedPattern1,
            TDerivedPattern2,
            TDerivedPattern3,
        >,
    >,
}

// TODO: make this a trait implementation
impl<
        TPuzzle: SemiGroupActionPuzzle,
        TDerivedPattern1: DerivedPattern<TPuzzle>,
        TDerivedPattern2: DerivedPattern<TPuzzle>,
        TDerivedPattern3: DerivedPattern<TPuzzle>,
    > TriplePhaseCoordinatePuzzle<TPuzzle, TDerivedPattern1, TDerivedPattern2, TDerivedPattern3>
{
    pub fn new(
        puzzle: TPuzzle,
        start_pattern: TPuzzle::Pattern,
        generator_moves: Vec<Move>,
    ) -> Self {
        let data = TriplePhaseCoordinatePuzzleData::<
            TPuzzle,
            TDerivedPattern1,
            TDerivedPattern2,
            TDerivedPattern3,
        > {
            // TODO: avoid cloning?
            puzzle1: GraphEnumeratedDerivedPatternPuzzle::<TPuzzle, TDerivedPattern1>::new(
                puzzle.clone(),
                start_pattern.clone(),
                generator_moves.clone(),
            ),
            puzzle2: GraphEnumeratedDerivedPatternPuzzle::<TPuzzle, TDerivedPattern2>::new(
                puzzle.clone(),
                start_pattern.clone(),
                generator_moves.clone(),
            ),
            puzzle3: GraphEnumeratedDerivedPatternPuzzle::<TPuzzle, TDerivedPattern3>::new(
                puzzle,
                start_pattern,
                generator_moves,
            ),
        };
        let data = Arc::new(data);
        Self { data }
    }

    // TODO: report errors for invalid patterns
    pub fn full_pattern_to_phase_coordinate(
        &self,
        pattern: &TPuzzle::Pattern,
    ) -> Result<
        TriplePhaseCoordinate<TPuzzle, TDerivedPattern1, TDerivedPattern2, TDerivedPattern3>,
        DerivedPatternConversionError,
    > {
        let coordinate1 = self.data.puzzle1.full_pattern_to_derived_pattern(pattern)?;
        let coordinate2 = self.data.puzzle2.full_pattern_to_derived_pattern(pattern)?;
        let coordinate3 = self.data.puzzle3.full_pattern_to_derived_pattern(pattern)?;
        Ok(TriplePhaseCoordinate {
            coordinate1,
            coordinate2,
            coordinate3,
        })
    }
}

impl<
        TPuzzle: SemiGroupActionPuzzle,
        TDerivedPattern1: DerivedPattern<TPuzzle>,
        TDerivedPattern2: DerivedPattern<TPuzzle>,
        TDerivedPattern3: DerivedPattern<TPuzzle>,
    > SemiGroupActionPuzzle
    for TriplePhaseCoordinatePuzzle<TPuzzle, TDerivedPattern1, TDerivedPattern2, TDerivedPattern3>
{
    type Pattern =
        TriplePhaseCoordinate<TPuzzle, TDerivedPattern1, TDerivedPattern2, TDerivedPattern3>;

    type Transformation = FlatMoveIndex;

    fn move_order(&self, r#move: &Move) -> Result<MoveCount, InvalidAlgError> {
        let move_order = self.data.puzzle1.move_order(r#move)?;
        debug_assert_eq!(move_order, self.data.puzzle2.move_order(r#move)?);
        debug_assert_eq!(move_order, self.data.puzzle3.move_order(r#move)?);
        Ok(move_order)
    }

    fn puzzle_transformation_from_move(
        &self,
        r#move: &Move,
    ) -> Result<Self::Transformation, InvalidAlgError> {
        let transformation = self.data.puzzle1.puzzle_transformation_from_move(r#move)?;
        debug_assert_eq!(
            transformation,
            self.data.puzzle2.puzzle_transformation_from_move(r#move)?
        );
        debug_assert_eq!(
            transformation,
            self.data.puzzle3.puzzle_transformation_from_move(r#move)?
        );
        Ok(transformation)
    }

    fn do_moves_commute(
        &self,
        move1_info: &MoveTransformationInfo<Self>,
        move2_info: &MoveTransformationInfo<Self>,
    ) -> bool {
        let do_moves_commute = self.data.puzzle1.do_moves_commute(
            &self
                .data
                .puzzle1
                .data
                .search_generators_for_derived_pattern_puzzle
                .flat[move1_info.flat_move_index],
            &self
                .data
                .puzzle1
                .data
                .search_generators_for_derived_pattern_puzzle
                .flat[move2_info.flat_move_index],
        );
        debug_assert_eq!(
            do_moves_commute,
            self.data.puzzle2.do_moves_commute(
                &self
                    .data
                    .puzzle2
                    .data
                    .search_generators_for_derived_pattern_puzzle
                    .flat[move1_info.flat_move_index],
                &self
                    .data
                    .puzzle2
                    .data
                    .search_generators_for_derived_pattern_puzzle
                    .flat[move2_info.flat_move_index],
            )
        );
        debug_assert_eq!(
            do_moves_commute,
            self.data.puzzle3.do_moves_commute(
                &self
                    .data
                    .puzzle3
                    .data
                    .search_generators_for_derived_pattern_puzzle
                    .flat[move1_info.flat_move_index],
                &self
                    .data
                    .puzzle3
                    .data
                    .search_generators_for_derived_pattern_puzzle
                    .flat[move2_info.flat_move_index],
            )
        );
        do_moves_commute
    }

    fn pattern_apply_transformation(
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Option<Self::Pattern> {
        let coordinate1 = self
            .data
            .puzzle1
            .pattern_apply_transformation(&pattern.coordinate1, transformation_to_apply)?;
        let coordinate2 = self
            .data
            .puzzle2
            .pattern_apply_transformation(&pattern.coordinate2, transformation_to_apply)?;
        let coordinate3 = self
            .data
            .puzzle3
            .pattern_apply_transformation(&pattern.coordinate3, transformation_to_apply)?;

        Some(Self::Pattern {
            coordinate1,
            coordinate2,
            coordinate3,
        })
    }

    fn pattern_apply_transformation_into(
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    ) -> bool {
        self.data.puzzle1.pattern_apply_transformation_into(
            &pattern.coordinate1,
            transformation_to_apply,
            &mut into_pattern.coordinate1,
        ) && self.data.puzzle2.pattern_apply_transformation_into(
            &pattern.coordinate2,
            transformation_to_apply,
            &mut into_pattern.coordinate2,
        ) && self.data.puzzle3.pattern_apply_transformation_into(
            &pattern.coordinate3,
            transformation_to_apply,
            &mut into_pattern.coordinate3,
        )
    }
}

pub struct TriplePhaseCoordinatePruneTable<
    TPuzzle: SemiGroupActionPuzzle,
    TDerivedPattern1: DerivedPattern<TPuzzle>,
    TDerivedPattern2: DerivedPattern<TPuzzle>,
    TDerivedPattern3: DerivedPattern<TPuzzle>,
> {
    tpuzzle:
        TriplePhaseCoordinatePuzzle<TPuzzle, TDerivedPattern1, TDerivedPattern2, TDerivedPattern3>,
}

impl<
        TPuzzle: SemiGroupActionPuzzle,
        TDerivedPattern1: DerivedPattern<TPuzzle>,
        TDerivedPattern2: DerivedPattern<TPuzzle>,
        TDerivedPattern3: DerivedPattern<TPuzzle>,
    >
    LegacyConstructablePruneTable<
        TriplePhaseCoordinatePuzzle<TPuzzle, TDerivedPattern1, TDerivedPattern2, TDerivedPattern3>,
    >
    for TriplePhaseCoordinatePruneTable<
        TPuzzle,
        TDerivedPattern1,
        TDerivedPattern2,
        TDerivedPattern3,
    >
{
    fn new(
        puzzle: TriplePhaseCoordinatePuzzle<
            TPuzzle,
            TDerivedPattern1,
            TDerivedPattern2,
            TDerivedPattern3,
        >,
        _search_api_data: std::sync::Arc<
            IterativeDeepeningSearchAPIData<
                TriplePhaseCoordinatePuzzle<
                    TPuzzle,
                    TDerivedPattern1,
                    TDerivedPattern2,
                    TDerivedPattern3,
                >,
            >,
        >,
        _search_logger: Arc<SearchLogger>,
        _min_size: Option<usize>,
        _search_adaptations_without_prune_table: SearchAdaptationsWithoutPruneTable<
            TriplePhaseCoordinatePuzzle<
                TPuzzle,
                TDerivedPattern1,
                TDerivedPattern2,
                TDerivedPattern3,
            >,
        >,
    ) -> Self {
        Self { tpuzzle: puzzle }
    }
}

impl<
        TPuzzle: SemiGroupActionPuzzle,
        TDerivedPattern1: DerivedPattern<TPuzzle>,
        TDerivedPattern2: DerivedPattern<TPuzzle>,
        TDerivedPattern3: DerivedPattern<TPuzzle>,
    >
    PruneTable<
        TriplePhaseCoordinatePuzzle<TPuzzle, TDerivedPattern1, TDerivedPattern2, TDerivedPattern3>,
    >
    for TriplePhaseCoordinatePruneTable<
        TPuzzle,
        TDerivedPattern1,
        TDerivedPattern2,
        TDerivedPattern3,
    >
{
    fn lookup(
        &self,
        pattern: &<TriplePhaseCoordinatePuzzle<
            TPuzzle,
            TDerivedPattern1,
            TDerivedPattern2,
            TDerivedPattern3,
        > as SemiGroupActionPuzzle>::Pattern,
    ) -> Depth {
        let depth1 = self.tpuzzle.data.puzzle1.data.exact_prune_table[pattern.coordinate1];
        let depth2 = self.tpuzzle.data.puzzle2.data.exact_prune_table[pattern.coordinate2];
        let depth3 = self.tpuzzle.data.puzzle3.data.exact_prune_table[pattern.coordinate3];
        max(max(depth1, depth2), depth3)
    }

    fn extend_for_search_depth(&mut self, _search_depth: Depth, _approximate_num_entries: usize) {
        // no-op
    }
}
