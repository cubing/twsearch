use cubing::{
    alg::{Alg, Move},
    kpuzzle::{KPattern, KPuzzle},
};

use crate::_internal::{
    errors::SearchError,
    search::{
        iterative_deepening::{
            iterative_deepening_search::{
                IndividualSearchOptions, IterativeDeepeningSearch,
                IterativeDeepeningSearchConstructionOptions,
            },
            target_pattern_signature::check_target_pattern_basic_consistency,
        },
        mask_pattern::apply_mask,
        search_logger::SearchLogger,
    },
};

use super::SearchPhase;

#[derive(Default)]
pub struct KPuzzleSimpleMaskPhaseConstructionOptions {
    pub search_logger: Option<SearchLogger>,
    pub individual_search_options: Option<IndividualSearchOptions>,
    /// If unspecified, [KPuzzleSimpleMaskPhase::try_new] will compute a target pattern by applying the mask to the KPuzzle's default pattern.
    pub masked_target_patterns: Option<Vec<KPattern>>,
}

pub struct KPuzzleSimpleMaskPhase {
    pub phase_name: String,
    pub mask: KPattern,
    pub iterative_deepening_search: IterativeDeepeningSearch,
    // TODO: support passing these in dynamically somehow
    pub individual_search_options: IndividualSearchOptions,
}

// TODO
unsafe impl Sync for KPuzzleSimpleMaskPhase {}
// TODO
unsafe impl Send for KPuzzleSimpleMaskPhase {}

impl KPuzzleSimpleMaskPhase {
    pub fn try_new(
        phase_name: String,
        mask: KPattern,
        generator_moves: Vec<Move>,
        options: KPuzzleSimpleMaskPhaseConstructionOptions,
    ) -> Result<Self, SearchError> {
        let kpuzzle = mask.kpuzzle();
        let target_patterns = match options.masked_target_patterns {
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
        let Ok(iterative_deepening_search) =
            IterativeDeepeningSearch::<KPuzzle>::try_new_kpuzzle_with_hash_prune_table_shim(
                kpuzzle.clone(),
                generator_moves,
                target_patterns,
                IterativeDeepeningSearchConstructionOptions {
                    search_logger: options.search_logger.unwrap_or_default().into(),
                    ..Default::default()
                },
                None,
            )
        else {
            return Err(SearchError {
                description: format!(
                    "Could not construct `IterativeDeepeningSearch` for phase: {}",
                    phase_name
                ),
            });
        };
        Ok(Self {
            phase_name,
            mask,
            iterative_deepening_search,
            individual_search_options: options.individual_search_options.unwrap_or_default(),
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
        check_target_pattern_basic_consistency::<KPuzzle>(
            &masked_pattern,
            &mut self
                .iterative_deepening_search
                .api_data
                .target_patterns
                .iter(),
        )?;
        // TODO: can we avoid a clone of `individual_search_options`?
        Ok(self
            .iterative_deepening_search
            .search_with_default_individual_search_adaptations(
                &masked_pattern,
                self.individual_search_options.clone(),
            )
            .next())
    }
}
