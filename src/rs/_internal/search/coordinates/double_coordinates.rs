use cubing::{
    alg::Move,
    kpuzzle::{InvalidAlgError, KPuzzle},
};

use crate::_internal::{
    canonical_fsm::search_generators::{FlatMoveIndex, MoveTransformationInfo},
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::move_count::MoveCount,
};

use super::phase_coordinate_puzzle::{
    PhaseCoordinateIndex, PhaseCoordinatePuzzle, SemanticCoordinate,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct DoublePhaseCoordinate {
    coordinate1: PhaseCoordinateIndex,
    coordinate2: PhaseCoordinateIndex,
}

#[derive(Clone, Debug)]
struct DoublePhaseCoordinatePuzzleData<
    TSemanticCoordinate1: SemanticCoordinate<KPuzzle>,
    TSemanticCoordinate2: SemanticCoordinate<KPuzzle>,
> {
    puzzle1: PhaseCoordinatePuzzle<TSemanticCoordinate1>,
    puzzle2: PhaseCoordinatePuzzle<TSemanticCoordinate2>,
}

#[derive(Clone, Debug)]
struct DoublePhaseCoordinatePuzzle<
    TSemanticCoordinate1: SemanticCoordinate<KPuzzle>,
    TSemanticCoordinate2: SemanticCoordinate<KPuzzle>,
> {
    data: DoublePhaseCoordinatePuzzleData<TSemanticCoordinate1, TSemanticCoordinate2>,
}

impl<
        TSemanticCoordinate1: SemanticCoordinate<KPuzzle>,
        TSemanticCoordinate2: SemanticCoordinate<KPuzzle>,
    > SemiGroupActionPuzzle
    for DoublePhaseCoordinatePuzzle<TSemanticCoordinate1, TSemanticCoordinate2>
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
            self.data.puzzle2.do_moves_commute(move1_info, move2_info)
        );
        Ok(do_moves_commute)
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
