use std::sync::Arc;

use crate::_internal::{
    cli::args::{SearchCommandOptionalArgs, VerbosityLevel},
    errors::CommandError,
    search::{
        idf_search::{IDFSearch, IndividualSearchOptions, SearchSolutions},
        search_logger::SearchLogger,
    },
};
use cubing::kpuzzle::KPuzzle;

use super::common::{KPuzzleSource, PatternSource};

pub fn search(
    definition: impl Into<KPuzzleSource>,
    search_pattern: impl Into<PatternSource>,
    search_command_optional_args: SearchCommandOptionalArgs,
) -> Result<SearchSolutions, CommandError> {
    if search_command_optional_args.search_args.all_optimal {
        eprintln!("⚠️ --all-optimal was specified, but is not currently implemented. Ignoring.");
    }

    let kpuzzle = definition.into().kpuzzle()?;
    let search_pattern = search_pattern.into().pattern(&kpuzzle)?;
    let target_pattern = match search_command_optional_args
        .scramble_and_target_pattern_optional_args
        .experimental_target_pattern
    {
        Some(path_buf) => PatternSource::FilePath(path_buf).pattern(&kpuzzle)?,
        None => kpuzzle.default_pattern(),
    };

    let mut idf_search = <IDFSearch<KPuzzle>>::try_new(
        kpuzzle.clone(),
        target_pattern,
        search_command_optional_args
            .generator_args
            .parse()
            .enumerate_moves_for_kpuzzle(&kpuzzle),
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
        &search_pattern,
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
        experimental_lib_api::{search, KPuzzleSource, PatternSource},
    };

    #[test]
    fn search_api_test() {
        let definition = KPuzzleSource::KPuzzle(cube3x3x3_kpuzzle().clone());
        let search_pattern = PatternSource::AlgAppliedToDefaultPattern(parse_alg!("R U R'"));
        let mut solutions = search(definition, search_pattern, Default::default()).unwrap();
        assert_eq!(solutions.next().unwrap().nodes.len(), 3);

        // TODO: allow this to be reused from above
        let definition = KPuzzleSource::KPuzzle(cube3x3x3_kpuzzle().clone());

        let search_pattern = PatternSource::AlgAppliedToDefaultPattern(parse_alg!("R U R'"));
        let mut solutions = search(
            definition,
            search_pattern,
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
