mod options;
mod serialize;
mod serve;

use std::{
    fs::read_to_string,
    io::Write,
    path::{Path, PathBuf},
};

use cubing::kpuzzle::KPuzzleDefinition;
use options::reset_args_from;
use serve::serve;
use tempfile::NamedTempFile;

use crate::{
    options::get_options,
    serialize::{serialize_kpuzzle_definition, KPuzzleSerializationOptions},
};

// TODO: Figure out how to move this out of the main entry file.
#[cxx::bridge]
pub mod rust_api {
    unsafe extern "C++" {
        include!("twsearch/src/cpp/ffi/rust_api.h");
        fn rust_api_set_arg(s: &str);
        fn rust_api_set_kpuzzle_definition(s: &str);
        // fn rust_api_solve_scramble(s: &str) -> String;
        fn rust_api_solve_position(s: &str) -> String;
        fn rust_api_reset();
        fn rust_api_main_search(def_file: &str, scramble_file: &str);
    }
}

fn main_search(def_file: &Path, scramble_file: &Option<PathBuf>) {
    let (def_file, _temp_file) = match def_file.extension().map(|ext| ext.to_str()) {
        Some(Some("tws")) => (
            def_file.to_str().expect("Invalid def file path").to_owned(),
            None::<NamedTempFile>,
        ),
        _ => {
            let def_json_str = read_to_string(def_file).expect("Could not read def file.");
            let def: KPuzzleDefinition =
                serde_json::from_str(&def_json_str).expect("Definition file is not valid JSON.");
            let mut temp_file = NamedTempFile::new().expect("Could not create a temp file.");
            let def_cpp_str = serialize_kpuzzle_definition(
                def,
                Some(&KPuzzleSerializationOptions {
                    move_subset: None,
                    // move_subset: move_subset.clone(), // TODO
                    custom_start_state: None,
                }),
            )
            .expect("Coudld not serialize the definition");
            temp_file
                .write_all(def_cpp_str.as_bytes())
                .expect("Could not write serialized definition.");

            let s = temp_file
                .path()
                .to_str()
                .expect("Invalid def file path")
                .to_owned();
            (s, Some(temp_file))
        }
    };

    let scramble_file = match scramble_file {
        Some(scramble_file) => scramble_file.to_str().expect("Invalid scramble file path"),
        None => "",
    };
    rust_api::rust_api_main_search(&def_file, scramble_file)
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
    }
}
