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

fn read_to_json<T: for<'a> Deserialize<'a>>(input_file: &Path) -> Result<T, String> {
    format!("Rewriting: {:?}", input_file);
    let input_str = read_to_string(input_file).or(Err("Could not read input file."))?;
    let input_parsed: T =
        serde_json::from_str(&input_str).or(Err("Input file is not valid JSON."))?;
    Ok(input_parsed)
}

// Allow the C++ to take the inputs directly.
fn write_rewritten_input_file(
    output_str: String,
    debug_print_serialized_json: bool,
) -> Result<(String, Option<NamedTempFile>), String> {
    if debug_print_serialized_json {
        println!("\n\n--------\n{}\n--------\n\n", output_str);
    }

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

fn rewrite_input_file<T: for<'a> Deserialize<'a>>(
    input_file: &Path,
    rewrite_fn: fn(T) -> Result<String, String>,
    debug_print_serialized_json: bool,
) -> Result<(String, Option<NamedTempFile>), String> {
    let json = rewrite_fn(read_to_json(input_file)?)?;
    write_rewritten_input_file(json, debug_print_serialized_json)
}

fn must_rewrite_input_file<T: for<'a> Deserialize<'a>>(
    input_file: &Path,
    rewrite_fn: fn(T) -> Result<String, String>,
    debug_print_serialized_json: bool,
) -> (String, Option<NamedTempFile>) {
    let json = rewrite_fn(read_to_json(input_file).unwrap()).unwrap();
    match write_rewritten_input_file(json, debug_print_serialized_json) {
        Ok(v) => v,
        Err(e) => {
            panic!("{}", e)
        }
    }
}

fn must_rewrite_input_file_with_optional_second_file<
    T: for<'a> Deserialize<'a>,
    U: for<'a> Deserialize<'a>,
>(
    input_file_1: &Path,
    input_file_2: &Option<PathBuf>,
    rewrite_fn: fn(T, Option<U>) -> Result<String, String>,
    debug_print_serialized_json: bool,
) -> (String, Option<NamedTempFile>) {
    let input_file_1_json = read_to_json(input_file_1).unwrap();
    let input_file_2_json = match input_file_2 {
        Some(input_file_2) => read_to_json(input_file_2).unwrap(),
        None => None,
    };
    let json = rewrite_fn(input_file_1_json, input_file_2_json).unwrap();
    match write_rewritten_input_file(json, debug_print_serialized_json) {
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
    debug_print_serialized_json: bool,
    target_pattern_file: &Option<PathBuf>,
) -> Result<(), String> {
    reset_args_from(vec![args_for_reset]);

    let (def_file, _temp_file) = match def_file.extension().and_then(|ext| ext.to_str()) {
        Some("tws") => {
            if target_pattern_file.is_some() {
                return Err(
                    "Target pattern is currently not supported for `.tws` input files. Please use JSON.".to_owned(),
                );
            };
            (
                def_file.to_str().expect("Invalid def file path").to_owned(),
                None::<NamedTempFile>,
            )
        }
        _ => {
            must_rewrite_input_file_with_optional_second_file(
                def_file,
                target_pattern_file,
                |def: KPuzzleDefinition, custom_start_state: Option<KStateData>| {
                    let def = serialize_kpuzzle_definition(
                        def,
                        Some(&KPuzzleSerializationOptions {
                            move_subset: None,
                            // move_subset: move_subset.clone(), // TODO
                            custom_start_state,
                        }),
                    );
                    def.map_err(|e| e.to_string())
                },
                debug_print_serialized_json,
            )
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
                match rewrite_input_file(
                    scramble_file,
                    |scramble_list: ScrambleList| serialize_scramble_list(&scramble_list),
                    debug_print_serialized_json,
                ) {
                    Ok(v) => v,
                    Err(_) => must_rewrite_input_file(
                        scramble_file,
                        |kstate_data: KStateData| {
                            serialize_scramble_state_data("Anonymous_Scramble", &kstate_data)
                        },
                        debug_print_serialized_json,
                    ),
                }
            }
        },
        None => ("".to_owned(), None),
    };
    rust_api::rust_api_main_search(&def_file, &scramble_file);

    Ok(())
}
