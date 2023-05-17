use std::{
    fs::read_to_string,
    io::Write,
    path::{Path, PathBuf},
};

use cubing::kpuzzle::KPuzzleDefinition;
use tempfile::NamedTempFile;

use crate::{
    options::{reset_args_from, SetCppArgs},
    rust_api,
    serialize::{serialize_kpuzzle_definition, KPuzzleSerializationOptions},
};

pub fn main_search(
    args_for_reset: &dyn SetCppArgs,
    def_file: &Path,
    scramble_file: &Option<PathBuf>,
) {
    reset_args_from(vec![args_for_reset]);

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
