use cubing::{
    alg::{Alg, Move},
    kpuzzle::{KPattern, KPuzzle},
};

use crate::_internal::{
    errors::SearchError,
    search::{
        hash_prune_table::HashPruneTableSizeBounds,
        iterative_deepening::{
            individual_search::IndividualSearchOptions,
            iterative_deepening_search::{
                ImmutableSearchData, ImmutableSearchDataConstructionOptions,
                IterativeDeepeningSearch,
            },
            search_adaptations::StoredSearchAdaptations,
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
        let Ok(immutable_search_data) =
            ImmutableSearchData::try_from_common_options_with_auto_search_generators(
                kpuzzle.clone(),
                generator_moves,
                target_patterns,
                ImmutableSearchDataConstructionOptions {
                    search_logger: options.search_logger.unwrap_or_default().into(),
                    ..Default::default()
                },
            )
        else {
            return Err(SearchError {
                description: format!(
                    "Could not construct `IterativeDeepeningSearch` for phase: {}",
                    phase_name
                ),
            });
        };
        let iterative_deepening_search =
            IterativeDeepeningSearch::<KPuzzle>::new_with_hash_prune_table(
                immutable_search_data,
                StoredSearchAdaptations::default(),
                HashPruneTableSizeBounds::default(),
            );
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

    fn solutions(
        &mut self,
        phase_search_pattern: &KPattern,
    ) -> Result<Box<dyn Iterator<Item = Alg> + '_>, SearchError> {
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
                .immutable_search_data
                .target_patterns
                .iter(),
        )?;
        // TODO: can we avoid a clone of `individual_search_options`?
        Ok(Box::new(self.iterative_deepening_search.search(
            &masked_pattern,
            self.individual_search_options.clone(),
            Default::default(),
        )))
    }
}
