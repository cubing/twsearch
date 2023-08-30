mod commands;
mod io;

use std::sync::Arc;

use commands::canonical_algs::canonical_algs;
use cubing::kpuzzle::{KPattern, KPatternData, KPuzzle, KPuzzleDefinition};
use io::read_to_json;
use twsearch::{
    ArgumentError, CommandError, GodsAlgorithmSearch, PackedKPattern, PackedKPuzzle,
    _internal::cli::{get_options_cpp_wrapper, CliCommand, GodsAlgorithmArgs},
};

fn main() -> Result<(), CommandError> {
    let args = get_options_cpp_wrapper();

    match args.command {
        CliCommand::Completions(_completions_args) => {
            panic!("Completions should have been printed during options parsing, followed by program exit.");
        }
        CliCommand::Search(_search_command_args) => todo!(),
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

fn gods_algorithm(gods_algorithm_args: GodsAlgorithmArgs) -> Result<(), CommandError> {
    let def: Result<KPuzzleDefinition, ArgumentError> =
        read_to_json(&gods_algorithm_args.input_args.def_file);
    let def = def?;
    let kpuzzle = KPuzzle::try_from(def).map_err(|e| ArgumentError {
        description: format!("Invalid definition: {}", e),
    })?;
    let packed_kpuzzle: PackedKPuzzle =
        PackedKPuzzle::try_from(kpuzzle.clone()).map_err(|e| ArgumentError {
            description: format!("Invalid definition: {}", e),
        })?;

    let start_pattern: Option<PackedKPattern> =
        match gods_algorithm_args.start_pattern_args.start_pattern {
            Some(start_pattern) => {
                let kpattern_data: KPatternData = read_to_json(&start_pattern)?;
                let kpattern = KPattern {
                    kpuzzle: kpuzzle.clone(),
                    kpattern_data: Arc::new(kpattern_data),
                };
                Some(packed_kpuzzle.pack_pattern(kpattern))
            }
            None => None,
        };

    // TODO: automatic multiples.
    let move_list = gods_algorithm_args
        .moves_args
        .moves_parsed()
        .unwrap_or_else(|| kpuzzle.definition().moves.keys().cloned().collect());

    let mut gods_algorithm_table =
        GodsAlgorithmSearch::try_new(packed_kpuzzle, start_pattern, move_list)?;
    gods_algorithm_table.fill();
    Ok(())
}
