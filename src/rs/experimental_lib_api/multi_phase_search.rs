use std::marker::PhantomData;

use cubing::alg::{Alg, AlgNode, Pause};

use crate::{
    _internal::{
        errors::SearchError, puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
        search::search_logger::SearchLogger,
    },
    scramble::apply_flat_alg::apply_flat_alg,
};

use super::SearchPhase;

#[derive(Default)]
pub struct MultiPhaseSearchOptions {
    pub search_logger: SearchLogger,
    pub include_pause_between_phases: bool,
}

pub struct MultiPhaseSearch<TPuzzle: SemiGroupActionPuzzle> {
    tpuzzle: TPuzzle,
    pub phases: Vec<Box<dyn SearchPhase<TPuzzle>>>,
    options: MultiPhaseSearchOptions,
    pub phantom_data: PhantomData<TPuzzle>,
}

impl<TPuzzle: SemiGroupActionPuzzle> MultiPhaseSearch<TPuzzle> {
    pub fn try_new(
        tpuzzle: TPuzzle,
        phases: Vec<Box<dyn SearchPhase<TPuzzle>>>,
        options: MultiPhaseSearchOptions,
    ) -> Result<Self, SearchError> {
        Ok(Self {
            tpuzzle,
            phases,
            options,
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
            self.options
                .search_logger
                .write_info(&format!("Starting phase: {}", phase.phase_name()));
            self.options.search_logger.write_info(&format!(
                "Solution so far: {}",
                current_solution.clone().unwrap_or_default()
            ));

            // TODO: can we avoid clones?
            let Some(phase_search_pattern) = apply_flat_alg(
                &self.tpuzzle,
                &search_pattern.clone(),
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
            self.options.search_logger.write_info(&format!(
                "phase_search_pattern: {:#?}",
                phase_search_pattern
            ));
            let Some(phase_solution) = phase.solutions(&phase_search_pattern)?.next() else {
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
                        if self.options.include_pause_between_phases {
                            vec![AlgNode::PauseNode(Pause {})]
                        } else {
                            vec![]
                        },
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
