use crate::_internal::{
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::iterative_deepening::iterative_deepening_search::SolutionMoves,
};

use super::filtering_decision::FilteringDecision;

pub trait SearchSolutionFilter<TPuzzle: SemiGroupActionPuzzle> {
    fn filter_solution(
        pattern: &TPuzzle::Pattern,
        solution_moves: &SolutionMoves,
    ) -> FilteringDecision;
}

pub struct SearchSolutionFilterNoOp;

impl<TPuzzle: SemiGroupActionPuzzle> SearchSolutionFilter<TPuzzle> for SearchSolutionFilterNoOp {
    fn filter_solution(
        _pattern: &TPuzzle::Pattern,
        _solution_moves: &SolutionMoves,
    ) -> FilteringDecision {
        FilteringDecision::Accept
    }
}
