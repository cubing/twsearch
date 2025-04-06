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
        prune_table_trait::Depth,
    },
};

pub struct CanonicalizingSolvedKPatternDepthFilter {
    canonicalization_mask: KPattern,
    canonicalization_search: IterativeDeepeningSearch<KPuzzle>,
    solved_pattern: KPattern,
    depth_filtering_search: IterativeDeepeningSearch<KPuzzle>,
    // Note: the obvious implementation at this moment would allow accepting the depth dynamically. But we may use it for initialization-time optimizations in the future.
    min_optimal_solution_depth: Depth,
}

impl CanonicalizingSolvedKPatternDepthFilter {
    pub fn try_new(
        canonicalization_mask: KPattern,
        canonicalization_generator_moves: Vec<Move>,
        solved_pattern: KPattern,
        depth_filtering_generator_moves: Vec<Move>,
        min_optimal_solution_depth: Depth,
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
        let kpuzzle = canonicalization_mask.kpuzzle().clone();
        let canonical_masked_solved_pattern =
            apply_mask(&canonicalization_mask, &solved_pattern).unwrap();
        // TODO: use exact prune table (`GraphEnumeratedDerivedPatternPuzzlePruneTable`).
        let canonicalization_search =
            IterativeDeepeningSearch::<KPuzzle>::try_new_kpuzzle_with_hash_prune_table_shim(
                kpuzzle.clone(),
                canonicalization_generator_moves,
                vec![canonical_masked_solved_pattern],
                Default::default(),
                Default::default(),
            )?;
        let depth_filtering_search =
            IterativeDeepeningSearch::<KPuzzle>::try_new_kpuzzle_with_hash_prune_table_shim(
                kpuzzle,
                depth_filtering_generator_moves,
                vec![solved_pattern.clone()],
                Default::default(),
                None,
            )
            .unwrap();
        Ok(Self {
            canonicalization_mask,
            canonicalization_search,
            solved_pattern,
            depth_filtering_search,
            min_optimal_solution_depth,
        })
    }

    pub fn depth_filter(&mut self, pattern: &KPattern) -> Result<FilteringDecision, SearchError> {
        dbg!("depth_filter");
        dbg!(&pattern);
        dbg!(&self.canonicalization_mask);
        let masked_pattern = apply_mask(pattern, &self.canonicalization_mask).map_err(|err| {
            SearchError {
                description: err.description, // TODO: we shouldn't need to manually adapt errors here.
            }
        })?;
        dbg!(&masked_pattern);
        dbg!(self.canonicalization_mask == masked_pattern);
        let Some(canonicalizing_alg) = self
            .canonicalization_search
            .search(&masked_pattern, Default::default(), Default::default())
            .next()
        else {
            return Err("Could not canonicalize the puzzle pattern for depth filtering".into());
        };
        dbg!(canonicalizing_alg.to_string());
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
        dbg!(&canonicalized_pattern);
        Ok(
            match self
                .depth_filtering_search
                .search(
                    &canonicalized_pattern,
                    IndividualSearchOptions {
                        max_depth: Some(self.min_optimal_solution_depth),
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
