use cubing::kpuzzle::KPuzzle;

use crate::_internal::{
    cli::args::GodsAlgorithmOptionalArgs,
    errors::CommandError,
    gods_algorithm::gods_algorithm_table::{GodsAlgorithmSearch, GodsAlgorithmTable},
};

use super::common::PatternSource;

/// Note: the `gods_algorithm_optional_args` argument is not yet ergonomic, and will be refactored.
///
/// Usage example:
///
/// ```
/// use cubing::{alg::parse_alg, puzzles::cube2x2x2_kpuzzle};
/// use twsearch::{
///     _internal::cli::args::{GeneratorArgs, GodsAlgorithmOptionalArgs}, // TODO
///     experimental_lib_api::gods_algorithm,
/// };
///
/// let kpuzzle = cube2x2x2_kpuzzle();
/// let table = gods_algorithm(
///     kpuzzle,
///     GodsAlgorithmOptionalArgs {
///         generator_args: GeneratorArgs {
///             generator_moves_string: Some("U,F2,R".to_owned()), // TODO: make this semantic
///             ..Default::default()
///         },
///         ..Default::default()
///     },
/// )
/// .unwrap();
///
/// // Looking up any pattern is now O(1).
/// let depth = table.pattern_to_depth.get(
///     &kpuzzle
///         .default_pattern()
///         .apply_alg(&parse_alg!("U' F R' F' U R' U' R F2 U'"))
///         .unwrap(),
/// );
/// dbg!(depth);
/// ```
pub fn gods_algorithm(
    kpuzzle: &KPuzzle,
    gods_algorithm_optional_args: GodsAlgorithmOptionalArgs,
) -> Result<GodsAlgorithmTable, CommandError> {
    let start_pattern = match gods_algorithm_optional_args
        .start_pattern_args
        .start_pattern
    {
        Some(path_buf) => Some(PatternSource::FilePath(path_buf).pattern(kpuzzle)?),
        None => None,
    };
    let mut gods_algorithm_search = GodsAlgorithmSearch::try_new(
        kpuzzle.clone(),
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
        experimental_lib_api::gods_algorithm,
    };

    #[test]
    fn gods_algorithm_api_test() {
        let table = gods_algorithm(
            cube3x3x3_kpuzzle(),
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
