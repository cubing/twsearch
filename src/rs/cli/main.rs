mod commands;

use std::process::exit;

use commands::canonical_algs::canonical_algs;
use twsearch::{
    GodsAlgorithmTable,
    _internal::cli::{get_options_cpp_wrapper, CliCommand, GodsAlgorithmArgs},
};

fn main() {
    let args = get_options_cpp_wrapper();

    let result: Result<(), String> = match args.command {
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
        CliCommand::GodsAlgorithm(gods_algorithm_args) => gods_algorithm(_gods_algorithm_args),
        CliCommand::TimingTest(_args) => todo!(),
        CliCommand::CanonicalAlgs(args) => canonical_algs(&args),
    };
    if let Err(err) = result {
        eprintln!("{}", err);
        exit(1);
    }
}

fn gods_algorithm(gods_algorithm_args: GodsAlgorithmArgs) -> Result<(), String> {
    let gods_algorithm_table = GodsAlgorithmTable::new();
    Ok(())
}
