use std::{
    fs::read_to_string,
    io::Write,
    path::{Path, PathBuf},
};

use cubing::kpuzzle::KPuzzleDefinition;
use serde::Deserialize;
use tempfile::NamedTempFile;

use crate::{
    options::{reset_args_from, SetCppArgs},
    rust_api,
    serialize::{
        serialize_kpuzzle_definition, serialize_scramble_list, KPuzzleSerializationOptions,
        ScrambleList,
    },
};

// Allow the C++ to take the inputs directly.
fn rewrite_input_file<T: for<'a> Deserialize<'a>>(
    input_file: &Path,
    rewrite_fn: fn(T) -> String,
) -> (String, Option<NamedTempFile>) {
    format!("Rewriting: {:?}", input_file);
    let input_str = read_to_string(input_file).expect("Could not read input file.");
    let input_parsed: T = serde_json::from_str(&input_str).expect("Input file is not valid JSON.");
    let output_str = rewrite_fn(input_parsed);
    let mut temp_file = NamedTempFile::new().expect("Could not create a temp file.");
    temp_file
        .write_all(output_str.as_bytes())
        .expect("Could not write a rewritten input file.");

    let s = temp_file
        .path()
        .to_str()
        .expect("Used an invalid temporary file path.")
        .to_owned();
    (s, Some(temp_file))
}

pub fn main_search(
    args_for_reset: &dyn SetCppArgs,
    def_file: &Path,
    scramble_file: &Option<PathBuf>,
) {
    reset_args_from(vec![args_for_reset]);

    let (def_file, _temp_file) = match def_file.extension().and_then(|ext| ext.to_str()) {
        Some("tws") => (
            def_file.to_str().expect("Invalid def file path").to_owned(),
            None::<NamedTempFile>,
        ),
        _ => {
            rewrite_input_file(def_file, |def: KPuzzleDefinition| {
                serialize_kpuzzle_definition(
                    def,
                    Some(&KPuzzleSerializationOptions {
                        move_subset: None,
                        // move_subset: move_subset.clone(), // TODO
                        custom_start_state: None,
                    }),
                )
                .expect("Could not serialize the definition")
            })
        }
    };

    let (scramble_file, _temp_file2) = match scramble_file {
        Some(scramble_file) => match scramble_file.extension().and_then(|ext| ext.to_str()) {
            Some("scr") => (
                scramble_file
                    .to_str()
                    .expect("Invalid scramble file path")
                    .to_owned(),
                None::<NamedTempFile>,
            ),
            _ => rewrite_input_file(scramble_file, |scramble_list: ScrambleList| {
                serialize_scramble_list(&scramble_list)
            }),
        },
        None => ("".to_owned(), None),
    };
    rust_api::rust_api_main_search(&def_file, &scramble_file)
}
