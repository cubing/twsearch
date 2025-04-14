use cubing::{
    alg::Alg,
    kpuzzle::{KPattern, KPuzzle},
};

use crate::{
    _internal::{
        cli::args::VerbosityLevel,
        errors::SearchError,
        search::{
            filter::filtering_decision::FilteringDecision,
            iterative_deepening::iterative_deepening_search::{
                IndividualSearchOptions, IterativeDeepeningSearch,
                IterativeDeepeningSearchConstructionOptions,
            },
            prune_table_trait::Depth,
            search_logger::SearchLogger,
        },
    },
    scramble::{
        collapse::collapse_adjacent_moves,
        get_kpuzzle::GetKPuzzle,
        randomize::{
            randomize_orbit, OrbitOrientationConstraint, OrbitPermutationConstraint,
            OrbitRandomizationConstraints,
        },
        scramble_finder::{
            scramble_finder::ScrambleFinder,
            solving_based_scramble_finder::{NoScrambleOptions, SolvingBasedScrambleFinder},
        },
        scramble_search::move_list_from_vec,
    },
};

use super::definitions::master_tetraminx_kpuzzle;

pub struct MasterTetraminxScrambleFinder {
    kpuzzle: KPuzzle,
    search: IterativeDeepeningSearch<KPuzzle>,
}

impl Default for MasterTetraminxScrambleFinder {
    fn default() -> Self {
        let kpuzzle = master_tetraminx_kpuzzle().clone();
        let search = IterativeDeepeningSearch::try_new_kpuzzle_with_hash_prune_table_shim(
            kpuzzle.clone(),
            move_list_from_vec(vec!["U", "u", "L", "l", "R", "r", "B", "b"]),
            vec![kpuzzle.default_pattern()],
            IterativeDeepeningSearchConstructionOptions {
                search_logger: SearchLogger {
                    verbosity: VerbosityLevel::Info,
                }
                .into(),
                ..Default::default()
            },
            Default::default(),
        )
        .unwrap();
        Self { kpuzzle, search }
    }
}

impl ScrambleFinder for MasterTetraminxScrambleFinder {
    type TPuzzle = KPuzzle;
    type ScrambleOptions = NoScrambleOptions;

    fn filter_pattern(
        &mut self,
        pattern: &KPattern,
        _scramble_options: &Self::ScrambleOptions,
    ) -> FilteringDecision {
        if self
            .search
            .search(
                pattern,
                IndividualSearchOptions {
                    max_depth_exclusive: Some(Depth(4)),
                    ..Default::default()
                },
                Default::default(),
            )
            .next()
            .is_some()
        {
            FilteringDecision::Reject
        } else {
            FilteringDecision::Accept
        }
    }
}

impl SolvingBasedScrambleFinder for MasterTetraminxScrambleFinder {
    fn generate_fair_unfiltered_random_pattern(
        &mut self,
        _scramble_options: &Self::ScrambleOptions,
    ) -> KPattern {
        let mut pattern = master_tetraminx_kpuzzle().default_pattern();

        randomize_orbit(
            &mut pattern,
            0,
            "WINGS",
            OrbitRandomizationConstraints {
                permutation: Some(OrbitPermutationConstraint::EvenParity),
                ..Default::default()
            },
        );
        randomize_orbit(
            &mut pattern,
            1,
            "MIDGES",
            OrbitRandomizationConstraints {
                permutation: Some(OrbitPermutationConstraint::EvenParity),
                orientation: Some(OrbitOrientationConstraint::SumToZero),
                ..Default::default()
            },
        );
        randomize_orbit(
            &mut pattern,
            2,
            "CORNERS",
            OrbitRandomizationConstraints {
                permutation: Some(OrbitPermutationConstraint::IdentityPermutation),
                ..Default::default()
            },
        );
        randomize_orbit(
            &mut pattern,
            3,
            "CENTERS",
            OrbitRandomizationConstraints {
                permutation: Some(OrbitPermutationConstraint::EvenParity),
                ..Default::default()
            },
        );

        pattern
    }

    fn solve_pattern(
        &mut self,
        pattern: &KPattern,
        _scramble_options: &Self::ScrambleOptions,
    ) -> Result<Alg, SearchError> {
        // TODO: min move count?
        let Some(alg) = self
            .search
            .search(pattern, Default::default(), Default::default())
            .next()
        else {
            return Err("Could not find a solution".into());
        };
        Ok(alg)
    }

    fn collapse_inverted_alg(&mut self, alg: Alg) -> Alg {
        collapse_adjacent_moves(alg, 3, -1)
    }
}

impl GetKPuzzle for MasterTetraminxScrambleFinder {
    fn get_kpuzzle(&self) -> &KPuzzle {
        &self.kpuzzle
    }
}
