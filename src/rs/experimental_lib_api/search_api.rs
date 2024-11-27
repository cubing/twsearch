use std::sync::Arc;

use crate::_internal::{
    cli::args::{SearchCommandArgs, VerbosityLevel},
    errors::CommandError,
    search::{
        idf_search::{IDFSearch, IndividualSearchOptions, SearchSolutions},
        search_logger::SearchLogger,
    },
};
use cubing::kpuzzle::KPuzzle;

use super::common::{KPuzzleDefinitionSource, PatternSource};

pub fn search(
    definition: impl Into<KPuzzleDefinitionSource>,
    search_pattern: impl Into<PatternSource>,
    search_command_args: SearchCommandArgs,
) -> Result<SearchSolutions, CommandError> {
    if search_command_args.search_args.all_optimal {
        eprintln!("⚠️ --all-optimal was specified, but is not currently implemented. Ignoring.");
    }

    let kpuzzle = definition.into().kpuzzle()?;
    let search_pattern = search_pattern.into().pattern(&kpuzzle)?;
    let target_pattern = match search_command_args
        .def_and_optional_scramble_args
        .experimental_target_pattern
    {
        Some(path_buf) => PatternSource::FilePath(path_buf).pattern(&kpuzzle)?,
        None => kpuzzle.default_pattern(),
    };

    let mut idf_search = <IDFSearch<KPuzzle>>::try_new(
        kpuzzle.clone(),
        target_pattern,
        search_command_args
            .generator_args
            .parse()
            .enumerate_moves_for_kpuzzle(&kpuzzle),
        Arc::new(SearchLogger {
            verbosity: search_command_args
                .verbosity_args
                .verbosity
                .unwrap_or(VerbosityLevel::Error),
        }),
        &search_command_args.metric_args.metric,
        search_command_args.search_args.random_start,
        None,
    )?;

    let solutions = idf_search.search(
        &search_pattern,
        IndividualSearchOptions {
            min_num_solutions: search_command_args.min_num_solutions,
            min_depth: search_command_args.search_args.min_depth,
            max_depth: search_command_args.search_args.max_depth,
            ..Default::default()
        },
    );

    Ok(solutions)
}
