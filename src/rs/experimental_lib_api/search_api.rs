use std::sync::Arc;

use crate::_internal::{
    cli::args::{SearchCommandOptionalArgs, VerbosityLevel},
    errors::CommandError,
    search::{
        idf_search::{IDFSearch, IndividualSearchOptions, SearchSolutions},
        search_logger::SearchLogger,
    },
};
use cubing::kpuzzle::{KPattern, KPuzzle};

use super::common::PatternSource;

/// Note: the `search_command_optional_args` argument is not yet ergonomic, and will be refactored.
///
/// Usage example:
///
/// ```
/// use cubing::{alg::parse_alg, puzzles::cube2x2x2_kpuzzle};
/// use twsearch::{
///     _internal::cli::args::{GeneratorArgs, GodsAlgorithmOptionalArgs}, // TODO
///     experimental_lib_api::gods_algorithm,
/// };
///
/// let kpuzzle = cube2x2x2_kpuzzle();
/// let table = gods_algorithm(
///     kpuzzle,
///     GodsAlgorithmOptionalArgs {
///         generator_args: GeneratorArgs {
///             generator_moves_string: Some("U,F2,R".to_owned()), // TODO: make this semantic
///             ..Default::default()
///         },
///         ..Default::default()
///     },
/// )
/// .unwrap();
///
/// // Looking up any pattern is now O(1).
/// let depth = table.pattern_to_depth.get(
///     &kpuzzle
///         .default_pattern()
///         .apply_alg(&parse_alg!("U' F R' F' U R' U' R F2 U'"))
///         .unwrap(),
/// );
/// dbg!(depth);
/// ```
pub fn search(
    kpuzzle: &KPuzzle,
    search_pattern: &KPattern,
    search_command_optional_args: SearchCommandOptionalArgs,
) -> Result<SearchSolutions, CommandError> {
    if search_command_optional_args.search_args.all_optimal {
        eprintln!("⚠️ --all-optimal was specified, but is not currently implemented. Ignoring.");
    }

    let target_pattern = match search_command_optional_args
        .scramble_and_target_pattern_optional_args
        .experimental_target_pattern
    {
        Some(path_buf) => PatternSource::FilePath(path_buf).pattern(kpuzzle)?,
        None => kpuzzle.default_pattern(),
    };

    let mut idf_search = <IDFSearch<KPuzzle>>::try_new(
        kpuzzle.clone(),
        target_pattern,
        search_command_optional_args
            .generator_args
            .parse()
            .enumerate_moves_for_kpuzzle(kpuzzle),
        Arc::new(SearchLogger {
            verbosity: search_command_optional_args
                .verbosity_args
                .verbosity
                .unwrap_or(VerbosityLevel::Error),
        }),
        &search_command_optional_args.metric_args.metric,
        search_command_optional_args.search_args.random_start,
        None,
    )?;

    let solutions = idf_search.search(
        search_pattern,
        IndividualSearchOptions {
            min_num_solutions: search_command_optional_args.min_num_solutions,
            min_depth: search_command_optional_args.search_args.min_depth,
            max_depth: search_command_optional_args.search_args.max_depth,
            ..Default::default()
        },
    );

    Ok(solutions)
}

#[cfg(test)]
mod tests {
    use cubing::{alg::parse_alg, puzzles::cube3x3x3_kpuzzle};

    use crate::{
        _internal::cli::args::{GeneratorArgs, SearchCommandOptionalArgs},
        experimental_lib_api::search,
    };

    #[test]
    fn search_api_test() {
        let kpuzzle = cube3x3x3_kpuzzle();
        let search_pattern = kpuzzle
            .default_pattern()
            .apply_alg(&parse_alg!("R U R'"))
            .expect("Invalid alg for puzzle.");
        let mut solutions =
            search(kpuzzle, &search_pattern, Default::default()).expect("Search failed.");
        assert_eq!(solutions.next().expect("No solutions.").nodes.len(), 3);

        let mut solutions = search(
            kpuzzle,
            &search_pattern,
            SearchCommandOptionalArgs {
                generator_args: GeneratorArgs {
                    generator_moves_string: Some("R,U".to_owned()), // TODO: make this semantic
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(solutions.next().unwrap().nodes.len(), 3);
    }
}
