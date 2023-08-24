mod search;
mod serialize;
mod serve;
mod wrapper_options;

use std::process::exit;

use search::main_search;
use serve::serve;
use twsearch::_internal::cli::{get_options_cpp_wrapper, CliCommand};

// TODO: Figure out how to move this out of the main entry file.
#[cxx::bridge]
pub mod rust_api {
    unsafe extern "C++" {
        include!("twsearch-cpp-wrapper/ffi/rust_api.h");
        fn rust_api_set_arg(s: &str);
        fn rust_api_set_kpuzzle_definition(s: &str);
        // fn rust_api_solve_scramble(s: &str) -> String;
        fn rust_api_solve_position(s: &str) -> String;
        fn rust_api_reset();
        // TODO: We can't use `optional` because https://github.com/dtolnay/cxx/issues/87 is unresolved.
        // Use the empty string to indicate an empty value for `scramble_file`.
        fn rust_api_main_search(def_file: &str, scramble_file: &str);
    }
}

fn main() {
    let args = get_options_cpp_wrapper();

    let result = match args.command {
        CliCommand::Completions(_completions_args) => {
            panic!("Completions should have been printed during options parsing, followed by program exit.");
        }
        CliCommand::Search(search_command_args) => main_search(
            &search_command_args,
            &search_command_args
                .input_args
                .def_file_wrapper_args
                .def_file,
            &search_command_args.input_args.scramble_file,
            search_command_args
                .input_args
                .def_file_wrapper_args
                .debug_print_serialized_json,
            &search_command_args.input_args.experimental_target_pattern,
        ),
        CliCommand::Serve(serve_command_args) => serve(serve_command_args, true),
        // TODO: consolidate def-only arg implementations.
        CliCommand::SchreierSims(schreier_sims_command_args) => {
            println!("Warning: `schreier-sims` does not support searching with identical pieces. If there are any identical pieces, they will be treated as distinguishable.");

            main_search(
                &schreier_sims_command_args,
                &schreier_sims_command_args.input_args.def_file,
                &None,
                schreier_sims_command_args
                    .input_args
                    .debug_print_serialized_json,
                &None, // TODO: allow custom target pattern?
            )
        }
        CliCommand::GodsAlgorithm(gods_algorithm_args) => main_search(
            &gods_algorithm_args,
            &gods_algorithm_args.input_args.def_file,
            &None,
            gods_algorithm_args.input_args.debug_print_serialized_json,
            &None, // TODO: allow custom target pattern?
        ),
        CliCommand::TimingTest(args) => main_search(
            &args,
            &args.input_args.def_file,
            &None,
            args.input_args.debug_print_serialized_json,
            &None, // TODO: allow custom target pattern?
        ),
        CliCommand::CanonicalAlgs(args) => main_search(
            &args,
            &args.input_args.def_file,
            &None,
            args.input_args.debug_print_serialized_json,
            &None, // TODO: allow custom target pattern?
        ),
    };
    if let Err(err) = result {
        eprintln!("{}", err);
        exit(1);
    }
}
