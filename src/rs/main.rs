mod options;
mod serialize;
mod serve;

use std::process::exit;

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
    }
}

fn main() {
    let args = get_options();

    match args.command {
        options::Command::Completions(_completions_args) => {
            panic!("Completions should have been printed during options parsing, followed by program exit.")
        }
        options::Command::Search { search_args } => {
            println!("{:?}", search_args);
            exit(1)
        }
        options::Command::Serve { search_args } => serve(search_args),
        options::Command::SchreierSims {} => todo!(),
        options::Command::GodsAlgorithm(gods_algorithm_args) => {
            let def_file = gods_algorithm_args
                .input_args
                .def_file
                .to_str()
                .expect("Invalid def file path");
            let scramble_file = match &gods_algorithm_args.input_args.scramble_file {
                Some(scramble_file) => scramble_file.to_str().expect("Invalid scramble file path"),
                None => "",
            };
            reset_args_from(vec![&gods_algorithm_args]);
            rust_api::rust_main_search(def_file, scramble_file)
        }
    }
}
