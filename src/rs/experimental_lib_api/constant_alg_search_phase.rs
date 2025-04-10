use cubing::alg::Alg;

use crate::_internal::{errors::SearchError, puzzle_traits::puzzle_traits::SemiGroupActionPuzzle};

use super::SearchPhase;

pub struct ConstantAlgSearchPhase {
    pub phase_name: String,
    pub alg: Alg,
}

impl<TPuzzle: SemiGroupActionPuzzle> SearchPhase<TPuzzle> for ConstantAlgSearchPhase {
    fn phase_name(&self) -> &str {
        &self.phase_name
    }

    fn first_solution(
        &mut self,
        _phase_search_pattern: &<TPuzzle as SemiGroupActionPuzzle>::Pattern,
    ) -> Result<Option<Alg>, SearchError> {
        Ok(Some(self.alg.clone()))
    }
}
