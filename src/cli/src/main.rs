mod commands;
mod serve;

use commands::{
    benchmark::benchmark,
    canonical_algs::canonical_algs,
    cli_scramble::{cli_scramble, cli_scramble_finder, cli_solve_known_puzzle},
    cli_search::cli_search,
    gods_algorithm::cli_gods_algorithm,
};
use twips::_internal::{
    cli::args::{get_options, CliCommand},
    errors::CommandError,
};

use crate::commands::cli_derive::cli_derive;

#[tokio::main]
async fn main() -> Result<(), CommandError> {
    let args = get_options();

    match args.command {
        CliCommand::Completions(_completions_args) => {
            panic!("Completions should have been printed during options parsing, followed by program exit.");
        }
        CliCommand::Search(search_command_args) => cli_search(search_command_args),
        CliCommand::SolveKnownPuzzle(search_command_args) => {
            cli_solve_known_puzzle(search_command_args)
        }
        CliCommand::Serve(serve_command_args) => serve::serve::serve(serve_command_args).await,
        // TODO: consolidate def-only arg implementations.
        CliCommand::SchreierSims(_schreier_sims_command_args) => todo!(),
        CliCommand::GodsAlgorithm(gods_algorithm_args) => cli_gods_algorithm(gods_algorithm_args),
        CliCommand::TimingTest(_args) => todo!(),
        CliCommand::CanonicalAlgs(args) => canonical_algs(&args),
        CliCommand::Scramble(scramble_args) => cli_scramble(&scramble_args),
        CliCommand::ScrambleFinder(scramble_finder_solve_args) => {
            cli_scramble_finder(&scramble_finder_solve_args)
        }
        CliCommand::Derive(derive_args) => cli_derive(&derive_args),
        CliCommand::Benchmark(benchmark_args) => benchmark(&benchmark_args),
    }
}
