mod commands;
mod serve;

use std::{
    path::{Path, PathBuf},
    process::exit,
    sync::Arc,
};

use commands::{benchmark, canonical_algs, cli_scramble};
use cubing::{
    alg::Alg,
    kpuzzle::{KPattern, KPatternData, KPuzzle, KPuzzleDefinition},
};
use serve::serve;
use twsearch::_internal::{
    cli::options::{get_options, CliCommand, GodsAlgorithmArgs, SearchCommandArgs},
    options::VerbosityLevel,
    read_to_json, ArgumentError, CommandError, GodsAlgorithmSearch, IDFSearch,
    IndividualSearchOptions, SearchLogger,
};

fn main() -> Result<(), CommandError> {
    let args = get_options();

    match args.command {
        CliCommand::Completions(_completions_args) => {
            panic!("Completions should have been printed during options parsing, followed by program exit.");
        }
        CliCommand::Search(search_command_args) => search(search_command_args),
        CliCommand::Serve(serve_command_args) => serve(serve_command_args),
        // TODO: consolidate def-only arg implementations.
        CliCommand::SchreierSims(_schreier_sims_command_args) => todo!(),
        CliCommand::GodsAlgorithm(gods_algorithm_args) => gods_algorithm(gods_algorithm_args),
        CliCommand::TimingTest(_args) => todo!(),
        CliCommand::CanonicalAlgs(args) => canonical_algs(&args),
        CliCommand::Scramble(scramble_args) => cli_scramble(&scramble_args),
        CliCommand::Benchmark(benchmark_args) => benchmark(&benchmark_args),
    }
}

fn common(
    def_file: &Path,
    start_or_target_pattern_file: &Option<PathBuf>,
) -> Result<(KPuzzle, Option<KPattern>), CommandError> {
    let def: Result<KPuzzleDefinition, ArgumentError> = read_to_json(def_file);
    let def = def?;
    let kpuzzle = KPuzzle::try_from(def).map_err(|e| ArgumentError {
        description: format!("Invalid definition: {}", e),
    })?;

    let start_or_target_pattern: Option<KPattern> = match start_or_target_pattern_file {
        Some(start_pattern_file) => {
            let kpattern_data: KPatternData = read_to_json(start_pattern_file)?;
            Some(match KPattern::try_from_data(&kpuzzle, &kpattern_data) {
                Ok(start_or_target_pattern) => start_or_target_pattern,
                Err(e) => {
                    return Err(CommandError::ArgumentError(ArgumentError {
                        description: e.to_string(),
                    }))
                }
            })
        }
        None => None,
    };

    Ok((kpuzzle, start_or_target_pattern))
}

fn gods_algorithm(gods_algorithm_args: GodsAlgorithmArgs) -> Result<(), CommandError> {
    let (kpuzzle, start_pattern) = common(
        &gods_algorithm_args.input_args.def_file,
        &gods_algorithm_args.start_pattern_args.start_pattern,
    )?;
    let mut gods_algorithm_table = GodsAlgorithmSearch::try_new(
        kpuzzle,
        start_pattern,
        &gods_algorithm_args.generator_args.parse(),
        &gods_algorithm_args.metric_args.metric,
    )?;
    gods_algorithm_table.fill();
    Ok(())
}

fn search(search_command_args: SearchCommandArgs) -> Result<(), CommandError> {
    let (kpuzzle, target_pattern) = common(
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

    let mut idf_search = IDFSearch::try_new(
        kpuzzle,
        target_pattern,
        search_command_args.generator_args.parse(),
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

    let search_start_time = instant::Instant::now();
    let solutions = idf_search.search(
        &scramble_pattern,
        IndividualSearchOptions {
            min_num_solutions: search_command_args.min_num_solutions,
            min_depth: search_command_args.search_args.min_depth,
            max_depth: search_command_args.search_args.max_depth,
            disallowed_initial_quanta: None,
            disallowed_final_quanta: None,
        },
    );
    let mut solution_index = 0;
    for solution in solutions {
        solution_index += 1;
        println!(
            "{} // solution #{} ({} nodes)",
            solution,
            solution_index,
            solution.nodes.len()
        )
    }
    println!(
        "// Entire search duration: {:?}",
        instant::Instant::now() - search_start_time
    );

    Ok(())
}
