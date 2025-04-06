use cubing::{
    alg::Move,
    kpuzzle::{KPattern, KPuzzle},
};

use crate::_internal::{
    errors::SearchError,
    search::{
        filter::filtering_decision::FilteringDecision,
        iterative_deepening::iterative_deepening_search::{
            IndividualSearchOptions, IterativeDeepeningSearch,
        },
        mask_pattern::apply_mask,
        move_count::MoveCount,
        prune_table_trait::Depth,
    },
};

pub struct CanonicalizingSolvedKPatternDepthFilter {
    canonicalization_mask: KPattern,
    canonicalization_search: IterativeDeepeningSearch<KPuzzle>,
    solved_pattern: KPattern,
    depth_filtering_search: IterativeDeepeningSearch<KPuzzle>,
    // Note: the obvious implementation at this moment would allow accepting the depth dynamically. But we may use it for initialization-time optimizations in the future.
    min_optimal_solution_depth: MoveCount,
}

pub struct CanonicalizingSolvedKPatternDepthFilterConstructionParameters {
    pub canonicalization_mask: KPattern,
    pub canonicalization_generator_moves: Vec<Move>,
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
            &parameters.canonicalization_mask,
            &parameters.solved_pattern,
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
            canonicalization_mask: parameters.canonicalization_mask,
            solved_pattern: parameters.solved_pattern,
            min_optimal_solution_depth: parameters.min_optimal_solution_move_count,
        })
    }

    pub fn depth_filter(&mut self, pattern: &KPattern) -> Result<FilteringDecision, SearchError> {
        let masked_pattern = apply_mask(pattern, &self.canonicalization_mask).map_err(|err| {
            SearchError {
                description: err.description, // TODO: we shouldn't need to manually adapt errors here.
            }
        })?;
        let Some(canonicalizing_alg) = self
            .canonicalization_search
            .search(&masked_pattern, Default::default(), Default::default())
            .next()
        else {
            return Err("Could not canonicalize the puzzle pattern for depth filtering".into());
        };
        // TODO: this shouldn't be necessary, but we have to fix some masking stuff for 4×4×4 first.
        let Ok(pattern_with_canonicalizing_alg) = pattern.apply_alg(&canonicalizing_alg) else {
            return Err(
                "Could not apply the canonicalizing alg to the puzzle pattern for depth filtering"
                    .into(),
            );
        };
        let Ok(canonicalized_pattern) =
            apply_mask(&pattern_with_canonicalizing_alg, &self.solved_pattern)
        else {
            return Err(
                "Could not apply the solved pattern to the puzzle pattern for depth filtering"
                    .into(),
            );
        };
        Ok(
            match self
                .depth_filtering_search
                .search(
                    &canonicalized_pattern,
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
