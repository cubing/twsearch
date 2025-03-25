use std::marker::PhantomData;

use cubing::{alg::Move, kpuzzle::InvalidAlgError};
use num_integer::lcm;

use crate::_internal::{
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::{
        coordinates::pattern_deriver::{DerivedPatternPuzzle, PatternDeriver},
        move_count::MoveCount,
    },
};

use super::CompoundPuzzle;

#[derive(Clone, Debug)]
pub struct CompoundDerivedPuzzle<
    TPuzzle: SemiGroupActionPuzzle,
    TDerivedPatternPuzzle0: DerivedPatternPuzzle<TPuzzle>,
    TDerivedPatternPuzzle1: DerivedPatternPuzzle<TPuzzle>,
> {
    pub compound_puzzle: CompoundPuzzle<TDerivedPatternPuzzle0, TDerivedPatternPuzzle1>,
    pub phantom_data: PhantomData<TPuzzle>,
}

impl<
        TPuzzle: SemiGroupActionPuzzle,
        TDerivedPatternPuzzle0: DerivedPatternPuzzle<TPuzzle>,
        TDerivedPatternPuzzle1: DerivedPatternPuzzle<TPuzzle>,
    > PatternDeriver<TPuzzle>
    for CompoundDerivedPuzzle<TPuzzle, TDerivedPatternPuzzle0, TDerivedPatternPuzzle1>
{
    type DerivedPattern = <CompoundPuzzle<
        TDerivedPatternPuzzle0,
        TDerivedPatternPuzzle1,
    > as SemiGroupActionPuzzle>::Pattern;

    fn derive_pattern(
        &self,
        source_puzzle_pattern: &<TPuzzle as SemiGroupActionPuzzle>::Pattern,
    ) -> Option<Self::DerivedPattern> {
        let pattern0: TDerivedPatternPuzzle0::Pattern = self
            .compound_puzzle
            .tpuzzle0
            .derive_pattern(source_puzzle_pattern)?;
        let pattern1: TDerivedPatternPuzzle1::Pattern = self
            .compound_puzzle
            .tpuzzle1
            .derive_pattern(source_puzzle_pattern)?;
        Some((pattern0, pattern1))
    }
}

impl<
        TPuzzle: SemiGroupActionPuzzle,
        TDerivedPatternPuzzle0: DerivedPatternPuzzle<TPuzzle>,
        TDerivedPatternPuzzle1: DerivedPatternPuzzle<TPuzzle>,
    > DerivedPatternPuzzle<TPuzzle>
    for CompoundDerivedPuzzle<TPuzzle, TDerivedPatternPuzzle0, TDerivedPatternPuzzle1>
{
}

impl<
        TPuzzle: SemiGroupActionPuzzle,
        TDerivedPatternPuzzle0: DerivedPatternPuzzle<TPuzzle>,
        TDerivedPatternPuzzle1: DerivedPatternPuzzle<TPuzzle>,
    > SemiGroupActionPuzzle
    for CompoundDerivedPuzzle<TPuzzle, TDerivedPatternPuzzle0, TDerivedPatternPuzzle1>
{
    type Pattern = <CompoundPuzzle<
    TDerivedPatternPuzzle0,
    TDerivedPatternPuzzle1,
> as SemiGroupActionPuzzle>::Pattern;
    type Transformation = <CompoundPuzzle<
    TDerivedPatternPuzzle0,
    TDerivedPatternPuzzle1,
> as SemiGroupActionPuzzle>::Transformation;

    fn move_order(
        &self,
        r#move: &cubing::alg::Move,
    ) -> Result<crate::_internal::search::move_count::MoveCount, cubing::kpuzzle::InvalidAlgError>
    {
        Ok(MoveCount(lcm(
            self.compound_puzzle.tpuzzle0.move_order(r#move)?.0,
            self.compound_puzzle.tpuzzle1.move_order(r#move)?.0,
        )))
    }

    fn puzzle_transformation_from_move(
        &self,
        r#move: &cubing::alg::Move,
    ) -> Result<Self::Transformation, cubing::kpuzzle::InvalidAlgError> {
        self.compound_puzzle.puzzle_transformation_from_move(r#move)
    }

    fn do_moves_commute(&self, move1: &Move, move2: &Move) -> Result<bool, InvalidAlgError> {
        Ok(self
            .compound_puzzle
            .tpuzzle0
            .do_moves_commute(move1, move2)?
            && self
                .compound_puzzle
                .tpuzzle1
                .do_moves_commute(move1, move2)?)
    }

    fn pattern_apply_transformation(
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Option<Self::Pattern> {
        self.compound_puzzle
            .pattern_apply_transformation(pattern, transformation_to_apply)
    }

    fn pattern_apply_transformation_into(
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    ) -> bool {
        self.compound_puzzle.pattern_apply_transformation_into(
            pattern,
            transformation_to_apply,
            into_pattern,
        )
    }
}
