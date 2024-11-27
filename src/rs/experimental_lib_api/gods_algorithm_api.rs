use crate::_internal::{
    cli::args::GodsAlgorithmArgs, errors::CommandError,
    gods_algorithm::gods_algorithm_table::GodsAlgorithmSearch,
};

use super::common::parse_def_file_and_start_or_target_pattern_file;

pub fn gods_algorithm(gods_algorithm_args: GodsAlgorithmArgs) -> Result<(), CommandError> {
    let (kpuzzle, start_pattern) = parse_def_file_and_start_or_target_pattern_file(
        &gods_algorithm_args.input_args.def_file,
        &gods_algorithm_args.start_pattern_args.start_pattern,
    )?;
    let mut gods_algorithm_table = GodsAlgorithmSearch::try_new(
        kpuzzle,
        start_pattern,
        &gods_algorithm_args.generator_args.parse(),
        &gods_algorithm_args.metric_args.metric,
    )?;
    gods_algorithm_table.fill();
    Ok(())
}
