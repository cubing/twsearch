mod options;
mod serialize;
mod serve;

use std::process::exit;

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
    }
}

fn main() {
    let args = get_options();

    match args.command {
        options::Command::Search { search_args } => {
            println!("{:?}", search_args);
            exit(1)
        }
        options::Command::Serve { search_args } => serve(search_args),
        options::Command::Completions {
            completions_args: _,
        } => {
            panic!("Completions should have been printed during options parsing, followed by program exit.")
        }
    }
}
