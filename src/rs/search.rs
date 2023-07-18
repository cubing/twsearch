use std::{
    fs::read_to_string,
    io::Write,
    path::{Path, PathBuf},
};

use cubing::kpuzzle::{KPuzzleDefinition, KStateData};
use serde::Deserialize;
use tempfile::NamedTempFile;

use crate::{
    options::{reset_args_from, SetCppArgs},
    rust_api,
    serialize::{
        serialize_kpuzzle_definition, serialize_scramble_list, serialize_scramble_state_data,
        KPuzzleSerializationOptions, ScrambleList,
    },
};

// Allow the C++ to take the inputs directly.
fn rewrite_input_file<T: for<'a> Deserialize<'a>>(
    input_file: &Path,
    rewrite_fn: fn(T) -> Result<String, String>,
) -> Result<(String, Option<NamedTempFile>), String> {
    format!("Rewriting: {:?}", input_file);
    let input_str = read_to_string(input_file).or(Err("Could not read input file."))?;
    let input_parsed: T =
        serde_json::from_str(&input_str).or(Err("Input file is not valid JSON."))?;
    let output_str = rewrite_fn(input_parsed)?;
    let mut temp_file = NamedTempFile::new().or(Err("Could not create a temp file."))?;
    temp_file
        .write_all(output_str.as_bytes())
        .or(Err("Could not write a rewritten input file."))?;
    temp_file
        .flush()
        .or(Err("Could not flush rewritten file."))?;

    let s = match temp_file.path().to_str() {
        Some(s) => s,
        None => return Err("Used an invalid temporary file path.".to_owned()),
    };
    Ok((s.to_owned(), Some(temp_file)))
}

fn must_rewrite_input_file<T: for<'a> Deserialize<'a>>(
    input_file: &Path,
    rewrite_fn: fn(T) -> Result<String, String>,
) -> (String, Option<NamedTempFile>) {
    match rewrite_input_file(input_file, rewrite_fn) {
        Ok(v) => v,
        Err(e) => {
            panic!("{}", e)
        }
    }
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
            must_rewrite_input_file(def_file, |def: KPuzzleDefinition| {
                let def = serialize_kpuzzle_definition(
                    def,
                    Some(&KPuzzleSerializationOptions {
                        move_subset: None,
                        // move_subset: move_subset.clone(), // TODO
                        custom_start_state: None,
                    }),
                );
                def.map_err(|e| e.to_string())
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
            _ => {
                match rewrite_input_file(scramble_file, |scramble_list: ScrambleList| {
                    serialize_scramble_list(&scramble_list)
                }) {
                    Ok(v) => v,
                    Err(_) => must_rewrite_input_file(scramble_file, |kstate_data: KStateData| {
                        serialize_scramble_state_data("Anonymous_Scramble", &kstate_data)
                    }),
                }
            }
        },
        None => ("".to_owned(), None),
    };
    rust_api::rust_api_main_search(&def_file, &scramble_file)
}
