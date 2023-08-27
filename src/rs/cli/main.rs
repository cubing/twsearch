mod commands;
mod io;

use std::process::exit;

use commands::canonical_algs::canonical_algs;
use cubing::{
    kpuzzle::{KPuzzle, KPuzzleDefinition},
    parse_move,
};
use io::read_to_json;
use twsearch::{
    GodsAlgorithmSearch, PackedKPuzzle,
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
        CliCommand::GodsAlgorithm(gods_algorithm_args) => gods_algorithm(gods_algorithm_args),
        CliCommand::TimingTest(_args) => todo!(),
        CliCommand::CanonicalAlgs(args) => canonical_algs(&args),
    };
    if let Err(err) = result {
        eprintln!("{}", err);
        exit(1);
    }
}

fn gods_algorithm(gods_algorithm_args: GodsAlgorithmArgs) -> Result<(), String> {
    let def: KPuzzleDefinition = read_to_json(&gods_algorithm_args.input_args.def_file)?;
    let kpuzzle = KPuzzle::try_from(def).map_err(|e| e.description)?;
    let packed_kpuzzle: PackedKPuzzle =
        PackedKPuzzle::try_from(kpuzzle).map_err(|e| e.description)?;
    let move_list = vec![parse_move!("R2").unwrap(), parse_move!("U2").unwrap()];

    let mut gods_algorithm_table = GodsAlgorithmSearch::try_new(packed_kpuzzle, move_list)?;
    gods_algorithm_table.fill();
    Ok(())
}
