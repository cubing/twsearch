use std::sync::Arc;

use cubing::{
    alg::{Alg, AlgNode, Move, Pause},
    kpuzzle::{KPattern, KPuzzle},
};

use crate::_internal::{
    errors::SearchError,
    search::{
        idf_search::{IDFSearch, IDFSearchConstructionOptions, IndividualSearchOptions},
        mask_pattern::apply_mask,
        search_logger::SearchLogger,
    },
};

pub struct SimpleMaskPhase {
    pub idfs: IDFSearch,
    // TODO: allow this to be computed dynamically by the caller via callback?
    pub phase_info: SimpleMaskPhaseInfo,
}

pub struct SimpleMaskMultiphaseSearch {
    pub phases: Vec<SimpleMaskPhase>,
}

pub struct SimpleMaskPhaseInfo {
    pub name: String,
    pub mask: KPattern,
    pub generator_moves: Vec<Move>,
    pub individual_search_options: Option<IndividualSearchOptions>,
}

impl SimpleMaskMultiphaseSearch {
    pub fn try_new(
        kpuzzle: &KPuzzle,
        phases: Vec<SimpleMaskPhaseInfo>,
        mut search_logger: Option<SearchLogger>,
    ) -> Result<Self, SearchError> {
        let search_logger: Arc<SearchLogger> = search_logger.take().unwrap_or_default().into();
        let phases = phases
            .into_iter()
            .map(|phase_info| {
                let Ok(target_pattern) = apply_mask(&kpuzzle.default_pattern(), &phase_info.mask)
                else {
                    return Err(SearchError {
                        description: format!(
                            "Could not apply mask to default pattern for phase: {}",
                            phase_info.name
                        ),
                    });
                };
                let Ok(idfs) = IDFSearch::<KPuzzle>::try_new(
                    kpuzzle.clone(),
                    phase_info.generator_moves.clone(),
                    target_pattern,
                    IDFSearchConstructionOptions {
                        search_logger: search_logger.clone(),
                        ..Default::default()
                    },
                ) else {
                    return Err(SearchError {
                        description: format!(
                            "Could not apply mask to default pattern for phase: {}",
                            phase_info.name
                        ),
                    });
                };
                Ok(SimpleMaskPhase { idfs, phase_info })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { phases })
    }

    pub fn chain_first_solution_for_each_phase(
        &mut self,
        full_search_pattern: &KPattern,
    ) -> Result<Alg, SearchError> {
        let mut current_solution: Option<Alg> = None;
        for phase in self.phases.iter_mut() {
            let search_logger = &phase.idfs.api_data.search_logger;

            // TODO: avoid formatting unless it will be printed.
            search_logger.write_info(&format!("Starting phase: {}", phase.phase_info.name));
            let Ok(phase_search_pattern) = apply_mask(
                &full_search_pattern
                    .apply_alg(current_solution.as_ref().unwrap_or(&Alg::default()))
                    .unwrap(),
                &phase.phase_info.mask,
            ) else {
                return Err(SearchError {
                    description: format!(
                        "Could not apply mask to search pattern for phase: {}",
                        phase.phase_info.name
                    ),
                });
            };
            search_logger.write_info(&format!("{:#?}", phase_search_pattern));
            let Some(phase_solution) = phase
                .idfs
                .search(
                    &phase_search_pattern,
                    phase
                        .phase_info
                        .individual_search_options
                        .clone()
                        .unwrap_or_default(),
                )
                .next()
            else {
                return Err(SearchError {
                    description: format!(
                        "Could not find a solution for phase: {}",
                        phase.phase_info.name
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
            }
        }
        Ok(current_solution.expect("No phase solutions?"))
    }
}
