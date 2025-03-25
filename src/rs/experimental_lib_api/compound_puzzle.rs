use cubing::{alg::Move, kpuzzle::InvalidAlgError};
use num_integer::lcm;

use crate::_internal::{
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle, search::move_count::MoveCount,
};

#[derive(Clone, Debug)]
pub struct CompoundPuzzle<TPuzzle0: SemiGroupActionPuzzle, TPuzzle1: SemiGroupActionPuzzle> {
    pub tpuzzle0: TPuzzle0,
    pub tpuzzle1: TPuzzle1,
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
        dbg!(&self.tpuzzle1);
        dbg!(&r#move);
        dbg!(&self.tpuzzle1.puzzle_transformation_from_move(r#move));
        Ok((
            self.tpuzzle0.puzzle_transformation_from_move(r#move)?,
            self.tpuzzle1.puzzle_transformation_from_move(r#move)?,
        ))
    }

    fn do_moves_commute(&self, move1: &Move, move2: &Move) -> Result<bool, InvalidAlgError> {
        Ok(self.tpuzzle0.do_moves_commute(move1, move2)?
            && self.tpuzzle1.do_moves_commute(move1, move2)?)
    }

    fn pattern_apply_transformation(
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
