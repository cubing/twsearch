use crate::_internal::{
    cli::args::GodsAlgorithmOptionalArgs,
    errors::CommandError,
    gods_algorithm::gods_algorithm_table::{GodsAlgorithmSearch, GodsAlgorithmTable},
};

use super::common::{KPuzzleSource, PatternSource};

/// Note: the `search_command_optional_args` argument is not yet ergonomic, and will be refactored.
///
/// Usage example:
///
/// ```
/// use cubing::puzzles::cube3x3x3_kpuzzle;
///
/// use twsearch::{
///     _internal::cli::args::{GeneratorArgs, GodsAlgorithmOptionalArgs}, // TODO
///     experimental_lib_api::{gods_algorithm, KPuzzleSource},
/// };
///
/// let definition = KPuzzleSource::KPuzzle(cube3x3x3_kpuzzle().clone());
/// let table = gods_algorithm(
///     definition,
///     GodsAlgorithmOptionalArgs {
///         generator_args: GeneratorArgs {
///             generator_moves_string: Some("R2,U2".to_owned()), // TODO: make this semantic
///             ..Default::default()
///         },
///         ..Default::default()
///     },
/// )
/// .unwrap();
/// dbg!(&table.pattern_to_depth);
/// ```
pub fn gods_algorithm(
    definition: impl Into<KPuzzleSource>,
    gods_algorithm_optional_args: GodsAlgorithmOptionalArgs,
) -> Result<GodsAlgorithmTable, CommandError> {
    let kpuzzle = definition.into().kpuzzle()?;
    let start_pattern = match gods_algorithm_optional_args
        .start_pattern_args
        .start_pattern
    {
        Some(path_buf) => Some(PatternSource::FilePath(path_buf).pattern(&kpuzzle)?),
        None => None,
    };
    let mut gods_algorithm_search = GodsAlgorithmSearch::try_new(
        kpuzzle,
        start_pattern,
        &gods_algorithm_optional_args.generator_args.parse(),
        &gods_algorithm_optional_args.metric_args.metric,
    )?;
    gods_algorithm_search.fill();
    Ok(gods_algorithm_search.table)
}

#[cfg(test)]
mod tests {
    use cubing::puzzles::cube3x3x3_kpuzzle;

    use crate::{
        _internal::cli::args::{GeneratorArgs, GodsAlgorithmOptionalArgs},
        experimental_lib_api::{gods_algorithm, KPuzzleSource},
    };

    #[test]
    fn gods_algorithm_api_test() {
        let definition = KPuzzleSource::KPuzzle(cube3x3x3_kpuzzle().clone());
        let table = gods_algorithm(
            definition,
            GodsAlgorithmOptionalArgs {
                generator_args: GeneratorArgs {
                    generator_moves_string: Some("R2,U2".to_owned()), // TODO: make this semantic
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(table.pattern_to_depth.len(), 12);
    }
}
