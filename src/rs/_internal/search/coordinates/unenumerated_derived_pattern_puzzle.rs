use cubing::{alg::Move, kpuzzle::InvalidAlgError};

use crate::_internal::{
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle, search::move_count::MoveCount,
};

use super::pattern_deriver::{DerivedPuzzle, PatternDeriver};

#[derive(Clone, Debug)]
pub struct UnenumeratedDerivedPatternPuzzle<
    TSourcePuzzle: SemiGroupActionPuzzle,
    TDerivedPuzzle: SemiGroupActionPuzzle,
    TPatternDeriver: PatternDeriver<TSourcePuzzle, DerivedPattern = TDerivedPuzzle::Pattern>,
> {
    pub(crate) source_puzzle: TSourcePuzzle,
    pub(crate) derived_puzzle: TDerivedPuzzle,
    pub(crate) pattern_deriver: TPatternDeriver,
}

#[derive(Debug)]
pub enum DerivedPatternConversionError {
    InvalidDerivedPattern,
    InvalidDerivedPatternPuzzle,
}

impl<
        TSourcePuzzle: SemiGroupActionPuzzle,
        TDerivedPuzzle: SemiGroupActionPuzzle,
        TPatternDeriver: PatternDeriver<TSourcePuzzle, DerivedPattern = TDerivedPuzzle::Pattern>,
    > UnenumeratedDerivedPatternPuzzle<TSourcePuzzle, TDerivedPuzzle, TPatternDeriver>
{
    pub fn new(
        source_puzzle: TSourcePuzzle,
        derived_puzzle: TDerivedPuzzle,
        pattern_deriver: TPatternDeriver,
    ) -> Self {
        Self {
            source_puzzle,
            derived_puzzle,
            pattern_deriver,
        }
    }
}

impl<
        TSourcePuzzle: SemiGroupActionPuzzle,
        TDerivedPuzzle: SemiGroupActionPuzzle,
        TPatternDeriver: PatternDeriver<TSourcePuzzle, DerivedPattern = TDerivedPuzzle::Pattern>,
    > PatternDeriver<TSourcePuzzle>
    for UnenumeratedDerivedPatternPuzzle<TSourcePuzzle, TDerivedPuzzle, TPatternDeriver>
{
    type DerivedPattern = TPatternDeriver::DerivedPattern;

    fn derive_pattern(
        &self,
        source_puzzle_pattern: &<TSourcePuzzle as SemiGroupActionPuzzle>::Pattern,
    ) -> Option<Self::DerivedPattern> {
        self.pattern_deriver.derive_pattern(source_puzzle_pattern)
    }
}

impl<
        TSourcePuzzle: SemiGroupActionPuzzle,
        TDerivedPuzzle: SemiGroupActionPuzzle,
        TPatternDeriver: PatternDeriver<TSourcePuzzle, DerivedPattern = TDerivedPuzzle::Pattern>,
    > SemiGroupActionPuzzle
    for UnenumeratedDerivedPatternPuzzle<TSourcePuzzle, TDerivedPuzzle, TPatternDeriver>
{
    type Pattern = TDerivedPuzzle::Pattern;
    type Transformation = TDerivedPuzzle::Transformation;

    fn move_order(&self, r#move: &cubing::alg::Move) -> Result<MoveCount, InvalidAlgError> {
        self.source_puzzle.move_order(r#move)
    }

    fn puzzle_transformation_from_move(
        &self,
        r#move: &Move,
    ) -> Result<Self::Transformation, InvalidAlgError> {
        self.derived_puzzle.puzzle_transformation_from_move(r#move)
    }

    fn do_moves_commute(&self, move1: &Move, move2: &Move) -> Result<bool, InvalidAlgError> {
        self.derived_puzzle.do_moves_commute(move1, move2)
    }

    fn pattern_apply_transformation(
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
    ) -> Option<Self::Pattern> {
        self.derived_puzzle
            .pattern_apply_transformation(pattern, transformation_to_apply)
    }

    fn pattern_apply_transformation_into(
        &self,
        pattern: &Self::Pattern,
        transformation_to_apply: &Self::Transformation,
        into_pattern: &mut Self::Pattern,
    ) -> bool {
        self.derived_puzzle.pattern_apply_transformation_into(
            pattern,
            transformation_to_apply,
            into_pattern,
        )
    }
}

impl<
        TSourcePuzzle: SemiGroupActionPuzzle,
        TDerivedPuzzle: SemiGroupActionPuzzle,
        TPatternDeriver: PatternDeriver<TSourcePuzzle, DerivedPattern = TDerivedPuzzle::Pattern>,
    > DerivedPuzzle<TSourcePuzzle>
    for UnenumeratedDerivedPatternPuzzle<TSourcePuzzle, TDerivedPuzzle, TPatternDeriver>
{
}
