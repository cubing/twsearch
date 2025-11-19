use cubing::alg::Alg;

use crate::_internal::{
    errors::SearchError, puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::iterative_deepening::search_adaptations::IndividualSearchAdaptations,
};

pub trait SearchPhase<TPuzzle: SemiGroupActionPuzzle>: Send + Sync {
    // This can't be static, due to `dyn` constraints.
    fn phase_name(&self) -> &str;

    fn solutions<'a>(
        &'a mut self,
        phase_search_pattern: &TPuzzle::Pattern,
    ) -> Result<Box<dyn Iterator<Item = Alg> + 'a>, SearchError>;

    // We use `#[allow(…)]` to avoid underscores in the parameter names.
    #[allow(unused_variables)]
    fn individual_search_adaptations(
        &self,
        underived_phase_search_pattern: &TPuzzle::Pattern,
    ) -> Option<IndividualSearchAdaptations<TPuzzle>> {
        None
    }
}
