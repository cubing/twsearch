use std::sync::Arc;

use crate::_internal::{
    canonical_fsm::search_generators::{
        Generators, SearchGenerators, SearchGeneratorsConstructorOptions,
    },
    errors::TwipsError,
    notation::metric::TurnMetric,
    search::{
        hash_prune_table::HashPruneTableSizeBounds,
        iterative_deepening::{
            individual_search::IndividualSearchOptions,
            iterative_deepening_search::{
                ImmutableSearchData, ImmutableSearchDataConstructionOptions,
                IterativeDeepeningSearch, OwnedIterativeDeepeningSearchCursor,
            },
            search_adaptations::StoredSearchAdaptations,
        },
        search_logger::{SearchLogger, VerbosityLevel},
    },
};
use cubing::kpuzzle::{KPattern, KPuzzle};

#[derive(Debug, Default)]
pub struct SearchOptions {
    pub target_pattern: Option<KPattern>,
    // TODO: make this optional, or move it out of `SearchOptions`.
    pub generators: Generators,
    pub metric: Option<TurnMetric>,
    pub random_start: Option<bool>,
    pub verbosity: Option<VerbosityLevel>,
    pub individual_search_options: IndividualSearchOptions,
}

impl From<&SearchOptions> for SearchGeneratorsConstructorOptions {
    fn from(value: &SearchOptions) -> Self {
        Self {
            metric: value.metric,
            random_start: value.random_start,
        }
    }
}

/// Note: the `search_command_optional_args` argument is not yet ergonomic, and will be refactored.
///
/// Usage example:
///
/// ```
/// use cubing::{alg::parse_alg, puzzles::cube3x3x3_kpuzzle};
/// use twips::experimental_lib_api::{search};
///
/// let kpuzzle = cube3x3x3_kpuzzle();
/// let search_pattern = kpuzzle
///     .default_pattern()
///     .apply_alg(parse_alg!("R U R'"))
///     .expect("Invalid alg for puzzle.");
/// let solutions =
///     search(kpuzzle, &search_pattern, Default::default()).expect("Search failed.");
/// for solution in solutions.take(5) {
///     println!("{}", solution);
/// }
/// ```
pub fn search(
    kpuzzle: &KPuzzle,
    search_pattern: &KPattern,
    options: SearchOptions,
) -> Result<OwnedIterativeDeepeningSearchCursor, TwipsError> {
    let generator_moves = options.generators.enumerate_moves_for_kpuzzle(kpuzzle);
    let search_generators = SearchGenerators::try_new(kpuzzle, generator_moves, (&options).into())?;
    let target_pattern = match options.target_pattern {
        Some(target_pattern) => target_pattern,
        None => kpuzzle.default_pattern(),
    };
    let iterative_deepening_search = <IterativeDeepeningSearch<KPuzzle>>::new_with_hash_prune_table(
        ImmutableSearchData::try_from_common_options(
            kpuzzle.clone(),
            search_generators,
            vec![target_pattern], // TODO: support multiple target patterns in API
            ImmutableSearchDataConstructionOptions {
                search_logger: Arc::new(SearchLogger {
                    verbosity: options.verbosity.unwrap_or_default(),
                }),
                ..Default::default()
            },
        )?,
        StoredSearchAdaptations::default(),
        HashPruneTableSizeBounds::default(),
    );

    let solutions = iterative_deepening_search.owned_search(
        search_pattern,
        options.individual_search_options,
        Default::default(),
    );

    Ok(solutions)
}

#[cfg(test)]
mod tests {
    use cubing::{
        alg::{parse_alg, parse_move},
        puzzles::cube3x3x3_kpuzzle,
    };

    use crate::{
        _internal::canonical_fsm::search_generators::Generators,
        experimental_lib_api::{search, search_api::SearchOptions},
    };

    #[test]
    fn search_api_test() {
        let kpuzzle = cube3x3x3_kpuzzle();
        let search_pattern = kpuzzle
            .default_pattern()
            .apply_alg(parse_alg!("R U R'"))
            .expect("Invalid alg for puzzle.");
        let mut solutions =
            search(kpuzzle, &search_pattern, Default::default()).expect("Search failed.");
        assert_eq!(solutions.next().expect("No solutions.").nodes.len(), 3);

        let mut solutions = search(
            kpuzzle,
            &search_pattern,
            SearchOptions {
                generators: Generators::from(vec![
                    parse_move!("R").clone(),
                    parse_move!("U").clone(),
                ]),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(solutions.next().unwrap().nodes.len(), 3);
    }
}
