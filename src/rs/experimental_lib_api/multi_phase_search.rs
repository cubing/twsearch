use std::marker::PhantomData;

use cubing::{
    alg::{Alg, AlgNode, Move, Pause},
    kpuzzle::{KPattern, KPuzzle},
};

use crate::_internal::{
    errors::SearchError,
    puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::{
        iterative_deepening::iterative_deepening_search::{
            IndividualSearchOptions, IterativeDeepeningSearch,
            IterativeDeepeningSearchConstructionOptions,
        },
        mask_pattern::apply_mask,
        search_logger::SearchLogger,
    },
};

pub trait SearchPhase<TPuzzle: SemiGroupActionPuzzle>: Send + Sync {
    // This can't be static, due to `dyn` constraints.
    fn phase_name(&self) -> &str;

    // TODO
    // fn solutions(
    //     &mut self,
    //     phase_search_pattern: &KPattern,
    // ) -> Result<Box<dyn Iterator<Item = Alg>>, SearchError>;

    fn first_solution(
        &mut self,
        phase_search_pattern: &TPuzzle::Pattern,
    ) -> Result<Option<Alg>, SearchError>;
}

pub struct KPuzzleSimpleMaskPhase {
    pub phase_name: String,
    pub mask: KPattern,
    pub iterative_deepening_search: IterativeDeepeningSearch,
    // TODO: support passing these in dynamically somehow
    pub individual_search_options: IndividualSearchOptions,
}

impl KPuzzleSimpleMaskPhase {
    pub fn try_new(
        phase_name: String,
        mask: KPattern,
        generator_moves: Vec<Move>,
        search_logger: Option<SearchLogger>,
        individual_search_options: IndividualSearchOptions,
        masked_target_patterns: Option<Vec<KPattern>>,
    ) -> Result<Self, SearchError> {
        let kpuzzle = mask.kpuzzle();
        let target_patterns = match masked_target_patterns {
            Some(masked_target_patterns) => masked_target_patterns,
            None => {
                let Ok(target_pattern) = apply_mask(&kpuzzle.default_pattern(), &mask) else {
                    return Err(SearchError {
                        description: format!(
                            "Could not apply mask to default pattern for phase: {}",
                            phase_name
                        ),
                    });
                };
                vec![target_pattern]
            }
        };
        let Ok(iterative_deepening_search) = IterativeDeepeningSearch::<KPuzzle>::try_new(
            kpuzzle.clone(),
            generator_moves,
            target_patterns,
            IterativeDeepeningSearchConstructionOptions {
                search_logger: search_logger.unwrap_or_default().into(),
                ..Default::default()
            },
        ) else {
            return Err(SearchError {
                description: format!(
                    "Could not apply mask to default pattern for phase: {}",
                    phase_name
                ),
            });
        };
        Ok(Self {
            phase_name,
            mask,
            iterative_deepening_search,
            individual_search_options,
        })
    }

    // fn solutions(
    //     &mut self,
    //     phase_search_pattern: &KPattern,
    // ) -> Result<Box<dyn Iterator<Item = Alg>>, SearchError> {
    //     let Ok(masked_pattern) = apply_mask(phase_search_pattern, &self.mask) else {
    //         return Err(SearchError {
    //             description: format!(
    //                 "Could not apply mask to search pattern for phase: {}",
    //                 self.phase_name()
    //             ),
    //         });
    //     };
    //     // TODO: can we avoid a clone of `individual_search_options`?
    //     let iterator = self
    //         .iterative_deepening_search
    //         .search(&masked_pattern, self.individual_search_options.clone());
    //     Ok(Box::new(iterator))
    // }
}

impl SearchPhase<KPuzzle> for KPuzzleSimpleMaskPhase {
    fn phase_name(&self) -> &str {
        &self.phase_name
    }

    fn first_solution(
        &mut self,
        phase_search_pattern: &KPattern,
    ) -> Result<Option<Alg>, SearchError> {
        let Ok(masked_pattern) = apply_mask(phase_search_pattern, &self.mask) else {
            return Err(SearchError {
                description: format!(
                    "Could not apply mask to search pattern for phase: {}",
                    self.phase_name()
                ),
            });
        };
        // TODO: can we avoid a clone of `individual_search_options`?
        Ok(self
            .iterative_deepening_search
            .search(&masked_pattern, self.individual_search_options.clone())
            .next())
    }
}

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
