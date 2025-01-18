use cubing::alg::Move;

use crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle;

use super::phase_coordinate_puzzle::{PhaseCoordinateConversionError, PhaseCoordinateIndex};

pub trait DerivedPuzzle<FromPuzzle: SemiGroupActionPuzzle>: SemiGroupActionPuzzle {
    fn new(
        puzzle: FromPuzzle,
        start_pattern: FromPuzzle::Pattern,
        generator_moves: Vec<Move>,
    ) -> Self;

    fn full_pattern_to_phase_coordinate(
        &self,
        pattern: &FromPuzzle::Pattern,
    ) -> Result<PhaseCoordinateIndex<Self>, PhaseCoordinateConversionError>;
}
