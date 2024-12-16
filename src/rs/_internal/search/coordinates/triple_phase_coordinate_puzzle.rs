use std::{cmp::max, marker::PhantomData, sync::Arc};

use cubing::{alg::Move, kpuzzle::InvalidAlgError};

use crate::_internal::{
    canonical_fsm::search_generators::{FlatMoveIndex, MoveTransformationInfo},
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::{
        check_pattern::AlwaysValid,
        idf_search::{DefaultSearchOptimizations, IDFSearchAPIData, SearchOptimizations},
        move_count::MoveCount,
        prune_table_trait::{Depth, PruneTable},
        search_logger::SearchLogger,
    },
};

use super::phase_coordinate_puzzle::{
    PhaseCoordinateIndex, PhaseCoordinatePuzzle, SemanticCoordinate,
};

// TODO: modify the `DoublePhaseCoordinate` implementation so that `TriplePhaseCoordinate` can nest it.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TriplePhaseCoordinate {
    coordinate1: PhaseCoordinateIndex,
    coordinate2: PhaseCoordinateIndex,
    coordinate3: PhaseCoordinateIndex,
}

#[derive(Clone, Debug)]
pub struct TriplePhaseCoordinatePuzzleData<
    TPuzzle: SemiGroupActionPuzzle,
    TSemanticCoordinate1: SemanticCoordinate<TPuzzle>,
    TSemanticCoordinate2: SemanticCoordinate<TPuzzle>,
    TSemanticCoordinate3: SemanticCoordinate<TPuzzle>,
> {
    puzzle1: PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate1>,
    puzzle2: PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate2>,
    puzzle3: PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate3>,
}

#[derive(Clone, Debug)]
pub struct TriplePhaseCoordinatePuzzle<
    TPuzzle: SemiGroupActionPuzzle,
    TSemanticCoordinate1: SemanticCoordinate<TPuzzle>,
    TSemanticCoordinate2: SemanticCoordinate<TPuzzle>,
    TSemanticCoordinate3: SemanticCoordinate<TPuzzle>,
> {
    data: Arc<
        TriplePhaseCoordinatePuzzleData<
            TPuzzle,
            TSemanticCoordinate1,
            TSemanticCoordinate2,
            TSemanticCoordinate3,
        >,
    >,
}

// TODO: make this a trait implementation
impl<
        TPuzzle: SemiGroupActionPuzzle,
        TSemanticCoordinate1: SemanticCoordinate<TPuzzle>,
        TSemanticCoordinate2: SemanticCoordinate<TPuzzle>,
        TSemanticCoordinate3: SemanticCoordinate<TPuzzle>,
    >
    TriplePhaseCoordinatePuzzle<
        TPuzzle,
        TSemanticCoordinate1,
        TSemanticCoordinate2,
        TSemanticCoordinate3,
    >
{
    pub fn new(
        puzzle: TPuzzle,
        start_pattern: TPuzzle::Pattern,
        generator_moves: Vec<Move>,
    ) -> Self {
        let data = TriplePhaseCoordinatePuzzleData::<
            TPuzzle,
            TSemanticCoordinate1,
            TSemanticCoordinate2,
            TSemanticCoordinate3,
        > {
            // TODO: avoid cloning?
            puzzle1: PhaseCoordinatePuzzle::<TPuzzle, TSemanticCoordinate1>::new(
                puzzle.clone(),
                start_pattern.clone(),
                generator_moves.clone(),
            ),
            puzzle2: PhaseCoordinatePuzzle::<TPuzzle, TSemanticCoordinate2>::new(
                puzzle.clone(),
                start_pattern.clone(),
                generator_moves.clone(),
            ),
            puzzle3: PhaseCoordinatePuzzle::<TPuzzle, TSemanticCoordinate3>::new(
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
    ) -> TriplePhaseCoordinate {
        let coordinate1 = self.data.puzzle1.full_pattern_to_phase_coordinate(pattern);
        let coordinate2 = self.data.puzzle2.full_pattern_to_phase_coordinate(pattern);
        let coordinate3 = self.data.puzzle3.full_pattern_to_phase_coordinate(pattern);
        TriplePhaseCoordinate {
            coordinate1,
            coordinate2,
            coordinate3,
        }
    }
}

impl<
        TPuzzle: SemiGroupActionPuzzle,
        TSemanticCoordinate1: SemanticCoordinate<TPuzzle>,
        TSemanticCoordinate2: SemanticCoordinate<TPuzzle>,
        TSemanticCoordinate3: SemanticCoordinate<TPuzzle>,
    > SemiGroupActionPuzzle
    for TriplePhaseCoordinatePuzzle<
        TPuzzle,
        TSemanticCoordinate1,
        TSemanticCoordinate2,
        TSemanticCoordinate3,
    >
{
    type Pattern = TriplePhaseCoordinate;

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
            self.data
                .puzzle1
                .data
                .search_generators_for_phase_coordinate_puzzle
                .flat
                .at(move1_info.flat_move_index),
            self.data
                .puzzle1
                .data
                .search_generators_for_phase_coordinate_puzzle
                .flat
                .at(move2_info.flat_move_index),
        );
        debug_assert_eq!(
            do_moves_commute,
            self.data.puzzle2.do_moves_commute(
                self.data
                    .puzzle2
                    .data
                    .search_generators_for_phase_coordinate_puzzle
                    .flat
                    .at(move1_info.flat_move_index),
                self.data
                    .puzzle2
                    .data
                    .search_generators_for_phase_coordinate_puzzle
                    .flat
                    .at(move2_info.flat_move_index),
            )
        );
        debug_assert_eq!(
            do_moves_commute,
            self.data.puzzle3.do_moves_commute(
                self.data
                    .puzzle3
                    .data
                    .search_generators_for_phase_coordinate_puzzle
                    .flat
                    .at(move1_info.flat_move_index),
                self.data
                    .puzzle3
                    .data
                    .search_generators_for_phase_coordinate_puzzle
                    .flat
                    .at(move2_info.flat_move_index),
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
        ) && self.data.puzzle1.pattern_apply_transformation_into(
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
    TSemanticCoordinate1: SemanticCoordinate<TPuzzle>,
    TSemanticCoordinate2: SemanticCoordinate<TPuzzle>,
    TSemanticCoordinate3: SemanticCoordinate<TPuzzle>,
> {
    tpuzzle: TriplePhaseCoordinatePuzzle<
        TPuzzle,
        TSemanticCoordinate1,
        TSemanticCoordinate2,
        TSemanticCoordinate3,
    >,
}

impl<
        TPuzzle: SemiGroupActionPuzzle,
        TSemanticCoordinate1: SemanticCoordinate<TPuzzle>,
        TSemanticCoordinate2: SemanticCoordinate<TPuzzle>,
        TSemanticCoordinate3: SemanticCoordinate<TPuzzle>,
    >
    PruneTable<
        TriplePhaseCoordinatePuzzle<
            TPuzzle,
            TSemanticCoordinate1,
            TSemanticCoordinate2,
            TSemanticCoordinate3,
        >,
    >
    for TriplePhaseCoordinatePruneTable<
        TPuzzle,
        TSemanticCoordinate1,
        TSemanticCoordinate2,
        TSemanticCoordinate3,
    >
{
    fn new(
        puzzle: TriplePhaseCoordinatePuzzle<
            TPuzzle,
            TSemanticCoordinate1,
            TSemanticCoordinate2,
            TSemanticCoordinate3,
        >,
        _search_api_data: std::sync::Arc<
            IDFSearchAPIData<
                TriplePhaseCoordinatePuzzle<
                    TPuzzle,
                    TSemanticCoordinate1,
                    TSemanticCoordinate2,
                    TSemanticCoordinate3,
                >,
            >,
        >,
        _search_logger: Arc<SearchLogger>,
        _min_size: Option<usize>,
    ) -> Self {
        Self { tpuzzle: puzzle }
    }

    fn lookup(
        &self,
        pattern: &<TriplePhaseCoordinatePuzzle<
            TPuzzle,
            TSemanticCoordinate1,
            TSemanticCoordinate2,
            TSemanticCoordinate3,
        > as SemiGroupActionPuzzle>::Pattern,
    ) -> Depth {
        let depth1 = *self
            .tpuzzle
            .data
            .puzzle1
            .data
            .exact_prune_table
            .at(pattern.coordinate1);
        let depth2 = *self
            .tpuzzle
            .data
            .puzzle2
            .data
            .exact_prune_table
            .at(pattern.coordinate2);
        let depth3 = *self
            .tpuzzle
            .data
            .puzzle3
            .data
            .exact_prune_table
            .at(pattern.coordinate3);
        max(max(depth1, depth2), depth3)
    }

    fn extend_for_search_depth(&mut self, _search_depth: Depth, _approximate_num_entries: usize) {
        // no-op
    }
}

pub struct TriplePhaseCoordinateSearchOptimizations<
    TPuzzle: SemiGroupActionPuzzle,
    TSemanticCoordinate1: SemanticCoordinate<TPuzzle>,
    TSemanticCoordinate2: SemanticCoordinate<TPuzzle>,
    TSemanticCoordinate3: SemanticCoordinate<TPuzzle>,
> {
    phantom_data: PhantomData<(
        TPuzzle,
        TSemanticCoordinate1,
        TSemanticCoordinate2,
        TSemanticCoordinate3,
    )>,
}

impl<
        TPuzzle: SemiGroupActionPuzzle,
        TSemanticCoordinate1: SemanticCoordinate<TPuzzle>,
        TSemanticCoordinate2: SemanticCoordinate<TPuzzle>,
        TSemanticCoordinate3: SemanticCoordinate<TPuzzle>,
    >
    SearchOptimizations<
        TriplePhaseCoordinatePuzzle<
            TPuzzle,
            TSemanticCoordinate1,
            TSemanticCoordinate2,
            TSemanticCoordinate3,
        >,
    >
    for TriplePhaseCoordinateSearchOptimizations<
        TPuzzle,
        TSemanticCoordinate1,
        TSemanticCoordinate2,
        TSemanticCoordinate3,
    >
{
    type PatternValidityChecker = AlwaysValid; // TODO: reconcile this with fallible transformation application.
    type PruneTable = TriplePhaseCoordinatePruneTable<
        TPuzzle,
        TSemanticCoordinate1,
        TSemanticCoordinate2,
        TSemanticCoordinate3,
    >;
}

impl<
        TPuzzle: SemiGroupActionPuzzle,
        TSemanticCoordinate1: SemanticCoordinate<TPuzzle>,
        TSemanticCoordinate2: SemanticCoordinate<TPuzzle>,
        TSemanticCoordinate3: SemanticCoordinate<TPuzzle>,
    >
    DefaultSearchOptimizations<
        TriplePhaseCoordinatePuzzle<
            TPuzzle,
            TSemanticCoordinate1,
            TSemanticCoordinate2,
            TSemanticCoordinate3,
        >,
    >
    for TriplePhaseCoordinatePuzzle<
        TPuzzle,
        TSemanticCoordinate1,
        TSemanticCoordinate2,
        TSemanticCoordinate3,
    >
{
    type Optimizations = TriplePhaseCoordinateSearchOptimizations<
        TPuzzle,
        TSemanticCoordinate1,
        TSemanticCoordinate2,
        TSemanticCoordinate3,
    >;
}
