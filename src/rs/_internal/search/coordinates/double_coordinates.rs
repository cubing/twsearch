use cubing::{alg::Move, kpuzzle::InvalidAlgError};

use crate::_internal::{
    canonical_fsm::search_generators::{FlatMoveIndex, MoveTransformationInfo},
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::move_count::MoveCount,
};

use super::phase_coordinate_puzzle::{
    PhaseCoordinateIndex, PhaseCoordinatePuzzle, SemanticCoordinate,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DoublePhaseCoordinate {
    coordinate1: PhaseCoordinateIndex,
    coordinate2: PhaseCoordinateIndex,
}

#[derive(Clone, Debug)]
pub struct DoublePhaseCoordinatePuzzleData<
    TPuzzle: SemiGroupActionPuzzle,
    TSemanticCoordinate1: SemanticCoordinate<TPuzzle>,
    TSemanticCoordinate2: SemanticCoordinate<TPuzzle>,
> {
    puzzle1: PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate1>,
    puzzle2: PhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate2>,
}

#[derive(Clone, Debug)]
pub struct DoublePhaseCoordinatePuzzle<
    TPuzzle: SemiGroupActionPuzzle,
    TSemanticCoordinate1: SemanticCoordinate<TPuzzle>,
    TSemanticCoordinate2: SemanticCoordinate<TPuzzle>,
> {
    data: DoublePhaseCoordinatePuzzleData<TPuzzle, TSemanticCoordinate1, TSemanticCoordinate2>,
}

impl<
        TPuzzle: SemiGroupActionPuzzle,
        TSemanticCoordinate1: SemanticCoordinate<TPuzzle>,
        TSemanticCoordinate2: SemanticCoordinate<TPuzzle>,
    > SemiGroupActionPuzzle
    for DoublePhaseCoordinatePuzzle<TPuzzle, TSemanticCoordinate1, TSemanticCoordinate2>
{
    type Pattern = DoublePhaseCoordinate;

    type Transformation = FlatMoveIndex;

    fn move_order(&self, r#move: &Move) -> Result<MoveCount, InvalidAlgError> {
        let move_order = self.data.puzzle1.move_order(r#move)?;
        assert_eq!(move_order, self.data.puzzle2.move_order(r#move)?);
        Ok(move_order)
    }

    fn puzzle_transformation_from_move(
        &self,
        r#move: &Move,
    ) -> Result<Self::Transformation, InvalidAlgError> {
        let transformation = self.data.puzzle1.puzzle_transformation_from_move(r#move)?;
        assert_eq!(
            transformation,
            self.data.puzzle2.puzzle_transformation_from_move(r#move)?
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
                .search_generators
                .flat
                .at(move1_info.flat_move_index),
            self.data
                .puzzle1
                .data
                .search_generators
                .flat
                .at(move2_info.flat_move_index),
        );
        assert_eq!(
            do_moves_commute,
            self.data.puzzle2.do_moves_commute(
                self.data
                    .puzzle2
                    .data
                    .search_generators
                    .flat
                    .at(move1_info.flat_move_index),
                self.data
                    .puzzle2
                    .data
                    .search_generators
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
            .puzzle1
            .pattern_apply_transformation(&pattern.coordinate1, transformation_to_apply)?;
        Some(Self::Pattern {
            coordinate1,
            coordinate2,
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
        )
    }
}
