use std::marker::PhantomData;

use cubing::alg::{Alg, AlgNode, Pause};

use crate::_internal::{
    errors::SearchError, puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::search_logger::SearchLogger,
};

use super::SearchPhase;

pub struct MultiPhaseSearch<TPuzzle: SemiGroupActionPuzzle> {
    tpuzzle: TPuzzle,
    pub phases: Vec<Box<dyn SearchPhase<TPuzzle>>>,
    pub search_logger: SearchLogger,
    pub phantom_data: PhantomData<TPuzzle>,
}

impl<TPuzzle: SemiGroupActionPuzzle> MultiPhaseSearch<TPuzzle> {
    pub fn try_new(
        tpuzzle: TPuzzle,
        phases: Vec<Box<dyn SearchPhase<TPuzzle>>>,
        search_logger: Option<SearchLogger>,
    ) -> Result<Self, SearchError> {
        let search_logger: SearchLogger = search_logger.unwrap_or_default();
        Ok(Self {
            tpuzzle,
            phases,
            search_logger,
            phantom_data: PhantomData,
        })
    }

    pub fn chain_first_solution_for_each_phase(
        &mut self,
        search_pattern: &TPuzzle::Pattern,
    ) -> Result<Alg, SearchError> {
        let mut current_solution: Option<Alg> = None;
        for phase in self.phases.iter_mut() {
            // TODO: avoid formatting unless it will be printed.
            self.search_logger
                .write_info(&format!("Starting phase: {}", phase.phase_name()));
            self.search_logger.write_info(&format!(
                "Solution so far: {}",
                current_solution.clone().unwrap_or_default()
            ));

            // TODO: can we avoid clones?
            let Some(phase_search_pattern) = apply_flat_alg(
                &self.tpuzzle,
                search_pattern.clone(),
                &current_solution.clone().unwrap_or_default(),
            ) else {
                return Err(SearchError {
                    description: format!(
                        "Could not apply alg to search pattern for phase: {}",
                        phase.phase_name()
                    ),
                });
            };

            // dbg!(&phase_search_pattern);
            self.search_logger
                .write_info(&format!("{:#?}", phase_search_pattern));
            let Some(phase_solution) = phase.first_solution(&phase_search_pattern)? else {
                return Err(SearchError {
                    description: format!(
                        "Could not find a solution for phase: {}",
                        phase.phase_name()
                    ),
                });
            };

            // TODO: implement pause riffling.
            current_solution = match current_solution.take() {
                Some(current_solution) => Some(Alg {
                    nodes: [
                        current_solution.nodes,
                        vec![AlgNode::PauseNode(Pause {})],
                        phase_solution.nodes,
                    ]
                    .concat(),
                }),
                None => Some(Alg {
                    nodes: [phase_solution.nodes].concat(),
                }),
            };
        }
        Ok(current_solution.expect("No phase solutions?"))
    }
}

fn apply_flat_alg<TPuzzle: SemiGroupActionPuzzle>(
    tpuzzle: &TPuzzle,
    pattern: TPuzzle::Pattern,
    alg: &Alg,
) -> Option<TPuzzle::Pattern> {
    let mut pattern = pattern;
    for r#move in alg.nodes.iter() {
        match r#move {
            AlgNode::MoveNode(r#move) => {
                let transformation = tpuzzle.puzzle_transformation_from_move(r#move).ok()?;
                pattern = tpuzzle.pattern_apply_transformation(&pattern, &transformation)?;
            }
            AlgNode::PauseNode(_) => {}
            _ => todo!(
                "Phase algs with nodes other than `Move`s and `Pause`s are not currently supported"
            ),
        }
    }
    Some(pattern)
}
