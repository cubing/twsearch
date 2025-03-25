use std::fmt::Debug;

use cubing::kpuzzle::KPuzzle;

use crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle;

pub trait PatternDeriver<TSourcePuzzle: SemiGroupActionPuzzle = KPuzzle>: Clone + Debug {
    // TODO: split out the `Pattern` type from `SemiGroupActionPuzzle` into a `PuzzleAssociatedPattern` trait? This would make a bunch of code annoying, but may prevent some gnarly type issues that can't be handled otherwise due to https://github.com/rust-lang/rust/issues/20041
    type DerivedPattern: Eq + Clone + Debug;

    // TODO: Should this return a `Result<…>` instead?
    fn derive_pattern(
        &self,
        source_puzzle_pattern: &TSourcePuzzle::Pattern,
    ) -> Option<Self::DerivedPattern>;
}

// pub trait DerivedPatternPuzzle<TSourcePuzzle: SemiGroupActionPuzzle>:
//     SemiGroupActionPuzzle
// {
//     fn derive_pattern(
//         source_puzzle_pattern: &TSourcePuzzle::Pattern,
//     ) -> <Self as SemiGroupActionPuzzle>::Pattern;
// }

pub trait DerivedPatternPuzzle<TSourcePuzzle: SemiGroupActionPuzzle>:
    PatternDeriver<TSourcePuzzle> + SemiGroupActionPuzzle<Pattern = Self::DerivedPattern>
{
}
