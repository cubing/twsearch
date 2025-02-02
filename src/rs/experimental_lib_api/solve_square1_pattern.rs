use cubing::{
    alg::Alg,
    kpuzzle::{KPattern, KPatternData},
};

use crate::{
    _internal::errors::SearchError,
    scramble::{square1, square1_unbandaged_kpuzzle},
};

pub fn solve_square1_pattern(pattern: &KPattern) -> Result<Alg, SearchError> {
    square1::solve::Square1Solver::get_globally_shared()
        .lock()
        .unwrap()
        .solve_square1(pattern)
}

// TODO: figure out how to let the WASM API pass JSON data without the need for this.
pub fn solve_square1_pattern_from_json_data(
    kpattern_data: &KPatternData,
) -> Result<Alg, SearchError> {
    let pattern = match KPattern::try_from_data(square1_unbandaged_kpuzzle(), kpattern_data) {
        Ok(pattern) => pattern,
        Err(err) => {
            return Err(SearchError {
                description: err.to_string(),
            })
        }
    };
    solve_square1_pattern(&pattern)
}
