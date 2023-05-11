mod options;
mod serialize;
mod serve;

use std::path::{Path, PathBuf};

use options::reset_args_from;
use serve::serve;

use crate::options::get_options;

// TODO: Figure out how to move this out of the main entry file.
#[cxx::bridge]
pub mod rust_api {
    unsafe extern "C++" {
        include!("twsearch/src/cpp/rustapi.h");
        fn rust_arg(s: &str);
        fn rust_setksolve(s: &str);
        // fn rust_solvescramble(s: &str) -> String;
        fn rust_solveposition(s: &str) -> String;
        fn rust_reset();
        fn rust_main_search(def_file: &str, scramble_file: &str);

        include!("twsearch/src/vendor/nissy-classic/src/nissy_rust_api.h");
        fn rust_nissy_solve_alg_twophase(scramble_alg: &str) -> String;
    }
}

fn main_search(def_file: &Path, scramble_file: &Option<PathBuf>) {
    let def_file = def_file.to_str().expect("Invalid def file path");
    let scramble_file = match scramble_file {
        Some(scramble_file) => scramble_file.to_str().expect("Invalid scramble file path"),
        None => "",
    };
    rust_api::rust_main_search(def_file, scramble_file)
}

fn main() {
    let args = get_options();

    match args.command {
        options::Command::Completions(_completions_args) => {
            panic!("Completions should have been printed during options parsing, followed by program exit.")
        }
        options::Command::Search(search_command_args) => {
            reset_args_from(vec![&search_command_args]);
            main_search(
                &search_command_args
                    .input_args
                    .def_file_wrapper_args
                    .def_file,
                &search_command_args.input_args.scramble_file,
            )
        }
        options::Command::Serve(serve_command_args) => serve(serve_command_args),
        // TODO: consolidate def-only arg implementations.
        options::Command::SchreierSims(schreier_sims_command_args) => {
            println!("Warning: `schreier-sims` does not support searching with identical pieces. If there are any identical pieces, they will be treated as distinguishable.");
            reset_args_from(vec![&schreier_sims_command_args]);
            main_search(&schreier_sims_command_args.input_args.def_file, &None)
        }
        options::Command::GodsAlgorithm(gods_algorithm_args) => {
            reset_args_from(vec![&gods_algorithm_args]);
            main_search(&gods_algorithm_args.input_args.def_file, &None)
        }
        options::Command::TimingTest(args) => {
            reset_args_from(vec![&args]);
            main_search(&args.input_args.def_file, &None)
        }
        options::Command::CanonicalAlgs(args) => {
            reset_args_from(vec![&args]);
            main_search(&args.input_args.def_file, &None)
        }
        options::Command::NissyTwophase(args) => {
            rust_api::rust_nissy_solve_alg_twophase(&args.scramble.to_string())
        },
    }
}
