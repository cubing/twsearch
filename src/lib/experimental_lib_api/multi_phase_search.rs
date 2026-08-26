use std::{iter, marker::PhantomData};

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
        &self,
        search_pattern: &TPuzzle::Pattern,
    ) -> Result<Alg, SearchError> {
        let mut current_solution: Option<Alg> = None;

        // Need to use this to build the vec as Box<...> is not Clone
        let mut phases_solutions_iter: Vec<Option<Box<dyn Iterator<Item = Alg>>>> =
            iter::repeat_with(|| None).take(self.phases.len()).collect();
        let mut phases_current_solutions = vec![None; self.phases.len()];

        let mut idx = 0;
        let phases_length = self.phases.len();
        while idx < phases_length {
            let phase = &self.phases[idx];

            // TODO: avoid formatting unless it will be printed.
            self.options
                .search_logger
                .write_info(&format!("Starting phase: {}", phase.phase_name()));
            self.options.search_logger.write_info(&format!(
                "Solution so far: {}",
                current_solution.clone().unwrap_or_default()
            ));

            let Some(phase_search_pattern) = apply_flat_alg(
                &self.tpuzzle,
                search_pattern,
                current_solution.as_ref().unwrap_or(&Alg::default()),
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

            let mut phase_solutions = phase.solutions(&phase_search_pattern)?;

            let Some(phase_solution) = phase_solutions.next() else {
                self.options.search_logger.write_info(&format!(
                              "Could not find solutions for phase: {}. Trying the next solution from previous phase.",
                             phase.phase_name()
                          ));

                // Roll back the state to the previous phase, and try the next solution found
                let mut prev_phase_next_solution = None;
                while idx > 0 && prev_phase_next_solution.is_none() {
                    prev_phase_next_solution = phases_solutions_iter[idx - 1]
                        .as_mut()
                        .and_then(|v| v.next());
                    phases_current_solutions[idx - 1] = prev_phase_next_solution.clone();

                    phases_current_solutions[idx] = None;
                    phases_solutions_iter[idx] = None;

                    idx -= 1;
                }

                if prev_phase_next_solution.is_none() {
                    return Err(SearchError {
                        description: "Could not find a solution".to_string(),
                    });
                }

                // Cloning everything is not optimal, but this doesn't happen really often so should be fine
                current_solution = Some(Alg {
                    nodes: phases_current_solutions
                        .iter()
                        .take_while(|n| n.is_some())
                        .flatten()
                        .flat_map(|v| v.nodes.clone())
                        .collect(),
                });

                idx += 1;

                continue;
            };

            phases_solutions_iter[idx] = Some(phase_solutions);
            phases_current_solutions[idx] = Some(phase_solution.clone());

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
                    nodes: phase_solution.nodes,
                }),
            };

            idx += 1;
        }
        Ok(current_solution.expect("No phase solutions?"))
    }
}
