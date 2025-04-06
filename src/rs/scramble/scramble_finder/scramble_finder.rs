use crate::_internal::{
    puzzle_traits::puzzle_traits::{HasDefaultPattern, SemiGroupActionPuzzle},
    search::filter::filtering_decision::FilteringDecision,
};

pub trait ScrambleFinder: Default {
    type TPuzzle: SemiGroupActionPuzzle + HasDefaultPattern;
    type ScrambleOptions;

    fn filter_pattern(
        &mut self,
        pattern: &<<Self as ScrambleFinder>::TPuzzle as SemiGroupActionPuzzle>::Pattern,
        scramble_options: &Self::ScrambleOptions,
    ) -> FilteringDecision;
}
