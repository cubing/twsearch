use cubing::{
    alg::Move,
    kpuzzle::{KPattern, KPuzzle},
};

use crate::_internal::{
    errors::SearchError,
    search::{
        filter::filtering_decision::FilteringDecision,
        iterative_deepening::{
            individual_search::IndividualSearchOptions,
            iterative_deepening_search::IterativeDeepeningSearch,
            target_pattern_signature::check_target_pattern_basic_consistency,
        },
        mask_pattern::apply_mask,
        move_count::MoveCount,
        prune_table_trait::Depth,
    },
};

// TODO: reuse data stucture between canonicalization and depth search.
pub struct CanonicalizingSolvedKPatternDepthFilter {
    canonicalization_mask: KPattern,
    canonicalization_search: IterativeDeepeningSearch<KPuzzle>,
    max_canonicalizing_move_count_exclusive: MoveCount,

    depth_filtering_search: IterativeDeepeningSearch<KPuzzle>,
    // Note: the obvious implementation at this moment would allow accepting the depth dynamically. But we may use it for initialization-time optimizations in the future.
    min_optimal_solution_depth: MoveCount,
}

// TODO: reuse data stucture between canonicalization and depth search.
pub struct CanonicalizingSolvedKPatternDepthFilterConstructionParameters {
    pub canonicalization_mask: KPattern,
    pub canonicalization_generator_moves: Vec<Move>,
    pub max_canonicalizing_move_count_below: MoveCount,

    pub solved_pattern: KPattern,
    pub depth_filtering_generator_moves: Vec<Move>,
    // Note: the obvious implementation at this moment would allow accepting the depth dynamically. But we may use it for initialization-time optimizations in the future.
    pub min_optimal_solution_move_count: MoveCount,
}

impl CanonicalizingSolvedKPatternDepthFilter {
    pub fn try_new(
        parameters: CanonicalizingSolvedKPatternDepthFilterConstructionParameters,
    ) -> Result<Self, SearchError> {
        // let pattern_deriver = MaskedKPuzzleDeriver::new(canonicalization_mask.clone());
        // let canonicalization_puzzle = GraphEnumeratedDerivedPatternPuzzle::new(
        //     kpuzzle,
        //     pattern_deriver,
        //     canonical_masked_solved_pattern,
        //     canonicalization_generators,
        // );
        // let canonicalization_prune_table =
        //     GraphEnumeratedDerivedPatternPuzzlePruneTable::new(canonicalization_puzzle);
        let kpuzzle = parameters.canonicalization_mask.kpuzzle().clone();
        let canonical_masked_solved_pattern = apply_mask(
            &parameters.solved_pattern,
            &parameters.canonicalization_mask,
        )
        .unwrap();
        // TODO: use exact prune table (`GraphEnumeratedDerivedPatternPuzzlePruneTable`).
        let canonicalization_search =
            IterativeDeepeningSearch::<KPuzzle>::try_new_kpuzzle_with_hash_prune_table_shim(
                kpuzzle.clone(),
                parameters.canonicalization_generator_moves,
                vec![canonical_masked_solved_pattern],
                Default::default(),
                Default::default(),
            )?;
        let depth_filtering_search =
            IterativeDeepeningSearch::<KPuzzle>::try_new_kpuzzle_with_hash_prune_table_shim(
                kpuzzle,
                parameters.depth_filtering_generator_moves,
                vec![parameters.solved_pattern.clone()],
                Default::default(),
                None,
            )
            .unwrap();
        Ok(Self {
            canonicalization_search,
            depth_filtering_search,
            max_canonicalizing_move_count_exclusive: parameters.max_canonicalizing_move_count_below,
            canonicalization_mask: parameters.canonicalization_mask,
            min_optimal_solution_depth: parameters.min_optimal_solution_move_count,
        })
    }

    pub fn depth_filter(&mut self, pattern: &KPattern) -> Result<FilteringDecision, SearchError> {
        let masked_pattern = apply_mask(pattern, &self.canonicalization_mask).map_err(|err| {
            SearchError {
                description: err.description, // TODO: we shouldn't need to manually adapt errors here.
            }
        })?;

        check_target_pattern_basic_consistency::<KPuzzle>(
            &masked_pattern,
            &mut self.canonicalization_search.api_data.target_patterns.iter(),
        )?;
        let Some(canonicalizing_alg) = self
            .canonicalization_search
            .search(
                &masked_pattern,
                IndividualSearchOptions {
                    max_depth_exclusive: Some(Depth(
                        self.max_canonicalizing_move_count_exclusive.0,
                    )),
                    ..Default::default()
                },
                Default::default(),
            )
            .next()
        else {
            return Err("Could not canonicalize the puzzle pattern for depth filtering".into());
        };
        // dbg!(canonicalizing_alg.to_string());
        let Ok(pattern_with_canonicalizing_alg) = pattern.apply_alg(&canonicalizing_alg) else {
            return Err(
                "Could not apply the canonicalizing alg to the puzzle pattern for depth filtering"
                    .into(),
            );
        };

        check_target_pattern_basic_consistency::<KPuzzle>(
            &pattern_with_canonicalizing_alg,
            &mut self.depth_filtering_search.api_data.target_patterns.iter(),
        )?;
        Ok(
            match self
                .depth_filtering_search
                .search(
                    &pattern_with_canonicalizing_alg,
                    IndividualSearchOptions {
                        max_depth_exclusive: Some(Depth(self.min_optimal_solution_depth.0)),
                        ..Default::default()
                    },
                    Default::default(),
                )
                .next()
            {
                Some(_) => FilteringDecision::Reject,
                None => FilteringDecision::Accept,
            },
        )
    }
}
