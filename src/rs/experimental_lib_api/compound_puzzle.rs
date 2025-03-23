use cubing::alg::Move;
use num_integer::lcm;

use crate::_internal::{
    canonical_fsm::search_generators::{MoveTransformationInfo, SearchGenerators},
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::move_count::MoveCount,
};

#[derive(Clone, Debug)]
pub struct CompoundPuzzle<TPuzzle0: SemiGroupActionPuzzle, TPuzzle1: SemiGroupActionPuzzle> {
    pub tpuzzle0: TPuzzle0,
    pub tpuzzle1: TPuzzle1,
    pub search_generators_t0: SearchGenerators<TPuzzle0>, // TODO
    pub search_generators_t1: SearchGenerators<TPuzzle1>, // TODO
}

impl<TPuzzle0: SemiGroupActionPuzzle, TPuzzle1: SemiGroupActionPuzzle> SemiGroupActionPuzzle
    for CompoundPuzzle<TPuzzle0, TPuzzle1>
{
    type Pattern = (TPuzzle0::Pattern, TPuzzle1::Pattern);
    type Transformation = (TPuzzle0::Transformation, TPuzzle1::Transformation);

    fn move_order(&self, r#move: &Move) -> Result<MoveCount, cubing::kpuzzle::InvalidAlgError> {
        Ok(MoveCount(lcm(
            self.tpuzzle0.move_order(r#move)?.0,
            self.tpuzzle1.move_order(r#move)?.0,
        )))
    }

    fn puzzle_transformation_from_move(
        &self,
        r#move: &Move,
    ) -> Result<Self::Transformation, cubing::kpuzzle::InvalidAlgError> {
        Ok((
            self.tpuzzle0.puzzle_transformation_from_move(r#move)?,
            self.tpuzzle1.puzzle_transformation_from_move(r#move)?,
        ))
    }

    fn do_moves_commute(
        &self,
        move1_info: &MoveTransformationInfo<Self>,
        move2_info: &MoveTransformationInfo<Self>,
    ) -> bool {
        self.tpuzzle0.do_moves_commute(
            &self.search_generators_t0.flat[move1_info.flat_move_index],
            &self.search_generators_t0.flat[move2_info.flat_move_index],
        ) && self.tpuzzle1.do_moves_commute(
            &self.search_generators_t1.flat[move1_info.flat_move_index],
            &self.search_generators_t1.flat[move2_info.flat_move_index],
        )
    }

    fn pattern_apply_transformation(
        // TODO: this is a hack to allow `Phase2Puzzle` to access its tables, ideally we would avoid this.
        // Then again, this might turn out to be necessary for similar high-performance implementations.
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Option<Self::Pattern> {
        let pattern0 = self
            .tpuzzle0
            .pattern_apply_transformation(&pattern.0, &transformation_to_apply.0)?;
        let pattern1 = self
            .tpuzzle1
            .pattern_apply_transformation(&pattern.1, &transformation_to_apply.1)?;
        Some((pattern0, pattern1))
    }

    fn pattern_apply_transformation_into(
        // TODO: this is a hack to allow `Phase2Puzzle` to access its tables, ideally we would avoid this.
        // Then again, this might turn out to be necessary for similar high-performance implementations.
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    ) -> bool {
        self.tpuzzle0.pattern_apply_transformation_into(
            &pattern.0,
            &transformation_to_apply.0,
            &mut into_pattern.0,
        ) && self.tpuzzle1.pattern_apply_transformation_into(
            &pattern.1,
            &transformation_to_apply.1,
            &mut into_pattern.1,
        )
    }
}
