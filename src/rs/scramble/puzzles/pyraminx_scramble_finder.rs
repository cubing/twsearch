use cubing::{
    alg::Alg,
    kpuzzle::{KPattern, KPuzzle},
};

use crate::{
    _internal::{
        errors::SearchError,
        search::{
            filter::filtering_decision::FilteringDecision,
            iterative_deepening::{
                individual_search::IndividualSearchOptions,
                iterative_deepening_search::IterativeDeepeningSearch,
            },
            move_count::MoveCount,
            prune_table_trait::Depth,
        },
    },
    scramble::{
        collapse::collapse_adjacent_moves,
        get_kpuzzle::GetKPuzzle,
        randomize::OrbitRandomizationConstraints,
        scramble_finder::{
            scramble_finder::ScrambleFinder,
            solving_based_scramble_finder::{NoScrambleOptions, SolvingBasedScrambleFinder},
        },
        scramble_search::move_list_from_vec,
    },
};

use super::{
    super::randomize::{randomize_orbit, OrbitOrientationConstraint, OrbitPermutationConstraint},
    definitions::pyraminx_kpuzzle,
};

// https://www.worldcubeassociation.org/regulations/#4b3d
const PYRAMINX_SCRAMBLE_FILTERING_MIN_MOVE_COUNT: MoveCount = MoveCount(6);
const PYRAMINX_SCRAMBLE_MIN_SCRAMBLE_ALG_MOVE_COUNT: MoveCount = MoveCount(11);

pub(crate) struct PyraminxScrambleFinder {
    kpuzzle: KPuzzle,
    search: IterativeDeepeningSearch<KPuzzle>,
}

impl Default for PyraminxScrambleFinder {
    fn default() -> Self {
        let kpuzzle = pyraminx_kpuzzle();

        let search = <IterativeDeepeningSearch>::try_new_kpuzzle_with_hash_prune_table_shim(
            kpuzzle.clone(),
            // TODO: the solution is sensitive to this order. We need a more robust API to specify that.
            move_list_from_vec(vec!["u", "l", "r", "b", "U", "L", "R", "B"]),
            vec![kpuzzle.default_pattern()],
            Default::default(),
            Default::default(),
        )
        .unwrap();

        Self {
            kpuzzle: kpuzzle.clone(),
            search,
        }
    }
}

impl ScrambleFinder for PyraminxScrambleFinder {
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
                    max_depth_exclusive: Some(Depth(PYRAMINX_SCRAMBLE_FILTERING_MIN_MOVE_COUNT.0)),
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

impl SolvingBasedScrambleFinder for PyraminxScrambleFinder {
    fn generate_fair_unfiltered_random_pattern(
        &mut self,
        _scramble_options: &Self::ScrambleOptions,
    ) -> KPattern {
        let mut scramble_pattern = self.kpuzzle.default_pattern();
        randomize_orbit(
            &mut scramble_pattern,
            0,
            "EDGES",
            OrbitRandomizationConstraints {
                permutation: Some(OrbitPermutationConstraint::EvenParity),
                orientation: Some(OrbitOrientationConstraint::SumToZero),
                ..Default::default()
            },
        );
        randomize_orbit(
            &mut scramble_pattern,
            1,
            "CORNERS",
            OrbitRandomizationConstraints {
                permutation: Some(OrbitPermutationConstraint::IdentityPermutation),
                ..Default::default()
            },
        );
        randomize_orbit(
            &mut scramble_pattern,
            2,
            "TIPS",
            OrbitRandomizationConstraints {
                permutation: Some(OrbitPermutationConstraint::IdentityPermutation),
                ..Default::default()
            },
        );
        scramble_pattern
    }

    fn solve_pattern(
        &mut self,
        pattern: &KPattern,
        _scramble_options: &Self::ScrambleOptions,
    ) -> Result<Alg, SearchError> {
        Ok(self
            .search
            .search(
                pattern,
                IndividualSearchOptions {
                    min_depth_inclusive: Some(Depth(
                        PYRAMINX_SCRAMBLE_MIN_SCRAMBLE_ALG_MOVE_COUNT.0,
                    )),
                    ..Default::default()
                },
                Default::default(),
            )
            .next()
            .unwrap())
    }

    fn collapse_inverted_alg(&mut self, alg: Alg) -> Alg {
        collapse_adjacent_moves(alg, 3, -1)
    }
}

impl GetKPuzzle for PyraminxScrambleFinder {
    fn get_kpuzzle(&self) -> &KPuzzle {
        &self.kpuzzle
    }
}
