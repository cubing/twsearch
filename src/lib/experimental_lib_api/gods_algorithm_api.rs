use cubing::kpuzzle::{KPattern, KPuzzle};

use crate::_internal::{
    canonical_fsm::search_generators::Generators,
    errors::TwipsError,
    gods_algorithm::gods_algorithm_table::{GodsAlgorithmSearch, GodsAlgorithmTable},
    notation::metric::TurnMetric,
};

#[derive(Default)]
pub struct GodsAlgorithmOptions {
    pub start_pattern: Option<KPattern>,
    pub generators: Generators,
    pub metric: Option<TurnMetric>,
}

/// Note: the `gods_algorithm_optional_args` argument is not yet ergonomic, and will be refactored.
///
/// Usage example:
///
/// ```
/// use cubing::{alg::{parse_alg, parse_move}, puzzles::cube2x2x2_kpuzzle};
/// use twips::{
///     _internal::canonical_fsm::search_generators::Generators,
///     experimental_lib_api::{gods_algorithm, GodsAlgorithmOptions},
/// };
///
/// let kpuzzle = cube2x2x2_kpuzzle();
/// let table = gods_algorithm(
///     kpuzzle,
///     GodsAlgorithmOptions {
///         generators: Generators::Custom {
///             moves: vec![parse_move!("U").clone(), parse_move!("R").clone()],
///             algs: vec![],
///         },
///         ..Default::default()
///     },
/// )
/// .unwrap();
/// // Looking up any pattern is now O(1).
/// let depth = table.pattern_to_depth.get(
///     &kpuzzle
///         .default_pattern()
///         .apply_alg(parse_alg!(
///             "F2 B2 D2 L' D L' D L2 F' U2 F' B2 R2 U2 F' D2 F"
///         ))
///         .unwrap(),
/// );
/// dbg!(depth);
/// ```
pub fn gods_algorithm(
    kpuzzle: &KPuzzle,
    options: GodsAlgorithmOptions,
) -> Result<GodsAlgorithmTable, TwipsError> {
    let start_pattern = options
        .start_pattern
        .unwrap_or_else(|| kpuzzle.default_pattern());
    let mut gods_algorithm_search = GodsAlgorithmSearch::try_new(
        kpuzzle.clone(),
        Some(start_pattern),
        &options.generators,
        options.metric.unwrap_or_default(),
    )?;
    gods_algorithm_search.fill();
    Ok(gods_algorithm_search.table)
}

#[cfg(test)]
mod tests {
    use cubing::{alg::parse_move, puzzles::cube3x3x3_kpuzzle};

    use crate::{
        _internal::canonical_fsm::search_generators::Generators,
        experimental_lib_api::{gods_algorithm, gods_algorithm_api::GodsAlgorithmOptions},
    };

    #[test]
    fn gods_algorithm_api_test() {
        let table = gods_algorithm(
            cube3x3x3_kpuzzle(),
            GodsAlgorithmOptions {
                generators: Generators::from(vec![
                    parse_move!("R2").clone(),
                    parse_move!("U2").clone(),
                ]),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(table.pattern_to_depth.len(), 12);
    }
}
