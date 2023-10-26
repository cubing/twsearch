mod commands;
mod serve;

use std::{
    path::{Path, PathBuf},
    process::exit,
    sync::Arc,
};

use commands::{benchmark, canonical_algs};
use cubing::{
    alg::Alg,
    kpuzzle::{KPattern, KPatternData, KPuzzle, KPuzzleDefinition},
};
use serve::serve;
use twsearch::_internal::{
    cli::options::{get_options, CliCommand, GodsAlgorithmArgs, SearchCommandArgs},
    options::VerbosityLevel,
    read_to_json, ArgumentError, CommandError, GodsAlgorithmSearch, IDFSearch,
    IndividualSearchOptions, PackedKPattern, PackedKPuzzle, SearchLogger,
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
        CliCommand::Benchmark(benchmark_args) => benchmark(&benchmark_args),
    }
}

fn common(
    def_file: &Path,
    start_or_target_pattern_file: &Option<PathBuf>,
) -> Result<(PackedKPuzzle, Option<PackedKPattern>), CommandError> {
    let def: Result<KPuzzleDefinition, ArgumentError> = read_to_json(def_file);
    let def = def?;
    let kpuzzle = KPuzzle::try_from(def).map_err(|e| ArgumentError {
        description: format!("Invalid definition: {}", e),
    })?;
    let packed_kpuzzle: PackedKPuzzle =
        PackedKPuzzle::try_from(kpuzzle.clone()).map_err(|e| ArgumentError {
            description: format!("Invalid definition: {}", e),
        })?;

    let start_or_target_pattern: Option<PackedKPattern> = match start_or_target_pattern_file {
        Some(start_pattern_file) => {
            let kpattern_data: KPatternData = read_to_json(start_pattern_file)?;
            let kpattern = KPattern {
                kpuzzle: kpuzzle.clone(),
                kpattern_data: Arc::new(kpattern_data),
            };
            Some(match packed_kpuzzle.try_pack_pattern(kpattern) {
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

    Ok((packed_kpuzzle, start_or_target_pattern))
}

fn gods_algorithm(gods_algorithm_args: GodsAlgorithmArgs) -> Result<(), CommandError> {
    let (packed_kpuzzle, start_pattern) = common(
        &gods_algorithm_args.input_args.def_file,
        &gods_algorithm_args.start_pattern_args.start_pattern,
    )?;
    let mut gods_algorithm_table = GodsAlgorithmSearch::try_new(
        packed_kpuzzle,
        start_pattern,
        &gods_algorithm_args.generator_args.parse(),
        &gods_algorithm_args.metric_args.metric,
    )?;
    gods_algorithm_table.fill();
    Ok(())
}

fn search(search_command_args: SearchCommandArgs) -> Result<(), CommandError> {
    let (packed_kpuzzle, target_pattern) = common(
        &search_command_args
            .input_def_and_optional_scramble_file_args
            .def_file_wrapper_args
            .def_file,
        &search_command_args
            .input_def_and_optional_scramble_file_args
            .experimental_target_pattern,
    )?;
    let target_pattern = target_pattern.unwrap_or_else(|| packed_kpuzzle.default_pattern());
    let scramble_pattern: PackedKPattern = match (
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
            // TODO: add a way for `PackedKPuzzle` to construct a PackedKPattern from serialized data directly.
            let unpacked_kpattern = KPattern {
                kpuzzle: packed_kpuzzle.data.kpuzzle.clone(),
                kpattern_data: Arc::new(kpattern_data),
            };
            match packed_kpuzzle.try_pack_pattern(unpacked_kpattern) {
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
            // TODO: add a way for `PackedKPuzzle` to construct a PackedKTransformation from serialized data directly.
            let unpacked_transformation =
                match packed_kpuzzle.data.kpuzzle.transformation_from_alg(&alg) {
                    Ok(unpacked_transformation) => unpacked_transformation,
                    Err(e) => {
                        eprintln!("Could apply scramble alg: {:?}", e);
                        exit(1);
                    }
                };
            let packed_transformation =
                match packed_kpuzzle.pack_transformation(&unpacked_transformation) {
                    Ok(packed_transformation) => packed_transformation,
                    Err(e) => {
                        eprintln!("Could not pack transformation: {:?}", e);
                        exit(1);
                    }
                };
            target_pattern.apply_transformation(&packed_transformation)
        }
        (Some(_), Some(_)) => {
            eprintln!("Error: specified both a scramble alg and a scramble file, exiting.");
            exit(1);
        }
    };

    let mut idf_search = IDFSearch::try_new(
        packed_kpuzzle,
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
