mod commands;
mod io;

use std::{
    path::{Path, PathBuf},
    process::exit,
    sync::Arc,
    time::Instant,
};

use commands::canonical_algs::canonical_algs;
use cubing::{
    alg::{Alg, Move},
    kpuzzle::{KPattern, KPatternData, KPuzzle, KPuzzleDefinition},
};
use io::read_to_json;
use twsearch::{
    ArgumentError, CommandError, GodsAlgorithmSearch, IDFSearch, PackedKPattern, PackedKPuzzle,
    SearchLogger,
    _internal::cli::{
        get_options_cpp_wrapper, CliCommand, GodsAlgorithmArgs, MovesArgs, SearchCommandArgs,
    },
};

fn main() -> Result<(), CommandError> {
    let args = get_options_cpp_wrapper();

    match args.command {
        CliCommand::Completions(_completions_args) => {
            panic!("Completions should have been printed during options parsing, followed by program exit.");
        }
        CliCommand::Search(search_command_args) => search(search_command_args),
        CliCommand::Serve(_serve_command_args) => {
            eprintln!("Skipping `serve` command.");
            Ok(())
        }
        // TODO: consolidate def-only arg implementations.
        CliCommand::SchreierSims(_schreier_sims_command_args) => todo!(),
        CliCommand::GodsAlgorithm(gods_algorithm_args) => gods_algorithm(gods_algorithm_args),
        CliCommand::TimingTest(_args) => todo!(),
        CliCommand::CanonicalAlgs(args) => canonical_algs(&args),
    }
}

fn common(
    def_file: &Path,
    start_or_target_pattern_file: &Option<PathBuf>,
    moves_args: &MovesArgs,
) -> Result<(PackedKPuzzle, Option<PackedKPattern>, Vec<Move>), CommandError> {
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
            Some(packed_kpuzzle.pack_pattern(kpattern))
        }
        None => None,
    };

    // TODO: automatic multiples.
    let move_list = moves_args
        .moves_parsed()
        .unwrap_or_else(|| kpuzzle.definition().moves.keys().cloned().collect());
    Ok((packed_kpuzzle, start_or_target_pattern, move_list))
}

fn gods_algorithm(gods_algorithm_args: GodsAlgorithmArgs) -> Result<(), CommandError> {
    let (packed_kpuzzle, start_pattern, move_list) = common(
        &gods_algorithm_args.input_args.def_file,
        &gods_algorithm_args.start_pattern_args.start_pattern,
        &gods_algorithm_args.moves_args,
    )?;
    let mut gods_algorithm_table =
        GodsAlgorithmSearch::try_new(packed_kpuzzle, start_pattern, move_list)?;
    gods_algorithm_table.fill();
    Ok(())
}

fn search(search_command_args: SearchCommandArgs) -> Result<(), CommandError> {
    let (packed_kpuzzle, target_pattern, move_list) = common(
        &search_command_args
            .input_def_and_optional_scramble_file_args
            .def_file_wrapper_args
            .def_file,
        &search_command_args
            .input_def_and_optional_scramble_file_args
            .experimental_target_pattern,
        &search_command_args.moves_args,
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
            packed_kpuzzle.pack_pattern(unpacked_kpattern)
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

    let idf_search = IDFSearch::try_new(
        packed_kpuzzle,
        target_pattern,
        move_list,
        Arc::new(SearchLogger {
            verbosity: search_command_args
                .verbosity_args
                .verbosity
                .unwrap_or(twsearch::_internal::cli::VerbosityLevel::Error),
        }),
    )?;

    let search_start_time = Instant::now();
    let solutions = idf_search.search(
        &scramble_pattern,
        search_command_args.min_num_solutions.unwrap_or(1),
    );
    let mut solution_index = 0;
    for solution in solutions {
        solution_index += 1;
        println!(
            "{} // solution #{} ({} moves)",
            solution,
            solution_index,
            solution.nodes.len()
        )
    }
    println!(
        "// Entire search duration: {:?}",
        Instant::now() - search_start_time
    );

    Ok(())
}
