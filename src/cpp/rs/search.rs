use std::path::PathBuf;

use twsearch::_internal::cli::options::InputDefFileOnlyArgs;

use crate::{
    rewrite::{rewrite_def_file, rewrite_scramble_file},
    rust_api,
    wrapper_options::{reset_args_from, SetCppArgs},
};

pub fn main_search(
    args_for_reset: &dyn SetCppArgs,
    input_args: &InputDefFileOnlyArgs,
    scramble_file: &Option<PathBuf>,
    target_pattern_file: &Option<PathBuf>,
) -> Result<(), String> {
    reset_args_from(vec![args_for_reset]);

    let (def_file, _temp1) = rewrite_def_file(input_args, target_pattern_file)?;
    let (scramble_file, _temp2) =
        rewrite_scramble_file(scramble_file, input_args.debug_print_serialized_json);

    rust_api::rust_api_main_search(&def_file, &scramble_file);

    Ok(())
}
