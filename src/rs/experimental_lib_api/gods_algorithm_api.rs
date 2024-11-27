use crate::_internal::{
    cli::args::GodsAlgorithmArgs, errors::CommandError,
    gods_algorithm::gods_algorithm_table::GodsAlgorithmSearch,
};

use super::common::{KPuzzleDefinitionSource, PatternSource};

pub fn gods_algorithm(gods_algorithm_args: GodsAlgorithmArgs) -> Result<(), CommandError> {
    let kpuzzle =
        KPuzzleDefinitionSource::FilePath(gods_algorithm_args.def_args.def_file).kpuzzle()?;
    let start_pattern = match gods_algorithm_args
        .optional
        .start_pattern_args
        .start_pattern
    {
        Some(path_buf) => Some(PatternSource::FilePath(path_buf).pattern(&kpuzzle)?),
        None => None,
    };
    let mut gods_algorithm_table = GodsAlgorithmSearch::try_new(
        kpuzzle,
        start_pattern,
        &gods_algorithm_args.optional.generator_args.parse(),
        &gods_algorithm_args.optional.metric_args.metric,
    )?;
    gods_algorithm_table.fill();
    Ok(())
}
