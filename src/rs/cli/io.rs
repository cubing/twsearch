use std::{fs::read_to_string, path::Path};

use serde::Deserialize;
use twsearch::ArgumentError;

pub fn read_to_json<T: for<'a> Deserialize<'a>>(input_file: &Path) -> Result<T, ArgumentError> {
    format!("Rewriting: {:?}", input_file);
    let input_str = read_to_string(input_file).or(Err("Could not read input file."))?;
    let input_parsed: T =
        serde_json::from_str(&input_str).or(Err("Input file is not valid JSON."))?;
    Ok(input_parsed)
}
