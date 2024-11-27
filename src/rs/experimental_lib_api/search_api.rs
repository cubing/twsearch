use std::{process::exit, sync::Arc};

use crate::{
    _internal::{
        cli::{
            args::{SearchCommandArgs, VerbosityLevel},
            io::read_to_json,
        },
        errors::{ArgumentError, CommandError},
        search::{
            idf_search::{IDFSearch, IndividualSearchOptions, SearchSolutions},
            search_logger::SearchLogger,
        },
    },
    experimental_lib_api::common::parse_def_file_and_start_or_target_pattern_file,
};
use cubing::{
    alg::Alg,
    kpuzzle::{KPattern, KPatternData, KPuzzle},
};

pub fn search(search_command_args: SearchCommandArgs) -> Result<SearchSolutions, CommandError> {
    if search_command_args.search_args.all_optimal {
        eprintln!("⚠️ --all-optimal was specified, but is not currently implemented. Ignoring.");
    }

    let (kpuzzle, target_pattern) = parse_def_file_and_start_or_target_pattern_file(
        &search_command_args
            .input_def_and_optional_scramble_file_args
            .def_file_wrapper_args
            .def_file,
        &search_command_args
            .input_def_and_optional_scramble_file_args
            .experimental_target_pattern,
    )?;
    let target_pattern = target_pattern.unwrap_or_else(|| kpuzzle.default_pattern());
    let scramble_pattern: KPattern = match (
        &search_command_args
            .input_def_and_optional_scramble_file_args
            .scramble_alg,
        &search_command_args
            .input_def_and_optional_scramble_file_args
            .scramble_file,
    ) {
        (None, None) => {
            println!("No scramble specified, exiting.");
            exit(0);
        }
        (None, Some(scramble_file)) => {
            let kpattern_data: KPatternData = match read_to_json(scramble_file) {
                Ok(kpattern_data) => kpattern_data,
                Err(e) => {
                    eprintln!("{:?}", e);
                    exit(1);
                }
            };
            match KPattern::try_from_data(&kpuzzle, &kpattern_data) {
                Ok(scramble_pattern) => scramble_pattern,
                Err(e) => {
                    return Err(CommandError::ArgumentError(ArgumentError {
                        description: e.to_string(),
                    }))
                }
            }
        }
        (Some(scramble_alg), None) => {
            let alg = match scramble_alg.parse::<Alg>() {
                Ok(alg) => alg,
                Err(e) => {
                    eprintln!("Could not parse alg: {:?}", e);
                    exit(1)
                }
            };
            // TODO: add a way for `KPuzzle` to construct a KTransformation from serialized data directly.
            let transformation = match kpuzzle.transformation_from_alg(&alg) {
                Ok(transformation) => transformation,
                Err(e) => {
                    eprintln!("Could apply scramble alg: {:?}", e);
                    exit(1);
                }
            };
            target_pattern.apply_transformation(&transformation)
        }
        (Some(_), Some(_)) => {
            eprintln!("Error: specified both a scramble alg and a scramble file, exiting.");
            exit(1);
        }
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
        &scramble_pattern,
        IndividualSearchOptions {
            min_num_solutions: search_command_args.min_num_solutions,
            min_depth: search_command_args.search_args.min_depth,
            max_depth: search_command_args.search_args.max_depth,
            ..Default::default()
        },
    );

    Ok(solutions)
}
