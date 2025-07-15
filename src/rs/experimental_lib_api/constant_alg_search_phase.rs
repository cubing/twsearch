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

    fn solutions(
        &mut self,
        _phase_search_pattern: &<TPuzzle as SemiGroupActionPuzzle>::Pattern,
    ) -> Result<Box<dyn Iterator<Item = Alg>>, SearchError> {
        Ok(Box::new(vec![self.alg.clone()].into_iter()))
    }
}
