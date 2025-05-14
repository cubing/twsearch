use std::sync::Arc;

use crate::_internal::{
    cli::args::{SearchCommandOptionalArgs, VerbosityLevel},
    errors::{ArgumentError, CommandError},
    search::{
        iterative_deepening::{
            continuation_condition::ContinuationCondition,
            iterative_deepening_search::{
                IndividualSearchOptions, IterativeDeepeningSearch,
                IterativeDeepeningSearchConstructionOptions, OwnedIterativeSearchCursor,
            },
            solution_moves::alg_to_moves,
        },
        search_logger::SearchLogger,
    },
};
use cubing::{
    alg::{Alg, Move},
    kpuzzle::{KPattern, KPuzzle},
};

use super::common::PatternSource;

/// Note: the `search_command_optional_args` argument is not yet ergonomic, and will be refactored.
///
/// Usage example:
///
/// ```
/// use cubing::{alg::parse_alg, puzzles::cube3x3x3_kpuzzle};
/// use twsearch::experimental_lib_api::{search};
///
/// let kpuzzle = cube3x3x3_kpuzzle();
/// let search_pattern = kpuzzle
///     .default_pattern()
///     .apply_alg(parse_alg!("R U R'"))
///     .expect("Invalid alg for puzzle.");
/// let solutions =
///     search(kpuzzle, &search_pattern, Default::default()).expect("Search failed.");
/// for solution in solutions.take(5) {
///     println!("{}", solution);
/// }
/// ```
pub fn search(
    kpuzzle: &KPuzzle,
    search_pattern: &KPattern,
    search_command_optional_args: SearchCommandOptionalArgs,
) -> Result<OwnedIterativeSearchCursor, CommandError> {
    if search_command_optional_args.search_args.all_optimal {
        eprintln!("⚠️ --all-optimal was specified, but is not currently implemented. Ignoring.");
    }

    let target_pattern = match search_command_optional_args
        .scramble_and_target_pattern_optional_args
        .experimental_target_pattern
    {
        Some(path_buf) => PatternSource::FilePath(path_buf).pattern(kpuzzle)?,
        None => kpuzzle.default_pattern(),
    };

    let iterative_deepening_search =
        <IterativeDeepeningSearch<KPuzzle>>::try_new_kpuzzle_with_hash_prune_table_shim(
            kpuzzle.clone(),
            search_command_optional_args
                .generator_args
                .parse()
                .enumerate_moves_for_kpuzzle(kpuzzle),
            vec![target_pattern], // TODO: support multiple target patterns in API
            IterativeDeepeningSearchConstructionOptions {
                search_logger: Arc::new(SearchLogger {
                    verbosity: search_command_optional_args
                        .verbosity_args
                        .verbosity
                        .unwrap_or(VerbosityLevel::Error),
                }),
                metric: search_command_optional_args.metric_args.metric,
                random_start: search_command_optional_args.search_args.random_start,
                ..Default::default()
            },
            None,
        )?;

    let root_continuation_condition = {
        match (
            search_command_optional_args.search_args.continue_after,
            search_command_optional_args.search_args.continue_at,
        ) {
            (None, None) => ContinuationCondition::None,
            (Some(after), None) => {
                ContinuationCondition::After(parse_continuation_alg_arg(&after)?)
            }
            (None, Some(at)) => ContinuationCondition::At(parse_continuation_alg_arg(&at)?),
            (Some(_), Some(_)) => {
                // TODO: figure out how to make this unrepresentable using idiomatic `clap` config.
                panic!("Specifying `--continue-after` and `--continue-at` simultaneously is supposed to be impossible.");
            }
        }
    };
    let solutions = iterative_deepening_search
        .owned_search_with_default_individual_search_adaptations(
            search_pattern,
            IndividualSearchOptions {
                min_num_solutions: search_command_optional_args.min_num_solutions,
                min_depth_inclusive: search_command_optional_args.search_args.min_depth,
                max_depth_exclusive: search_command_optional_args.search_args.max_depth,
                root_continuation_condition,
                ..Default::default()
            },
        );

    Ok(solutions)
}

fn parse_continuation_alg_arg(s: &str) -> Result<Vec<Move>, CommandError> {
    // TODO: unify code between branches to save code size?
    let alg = s.parse::<Alg>().map_err(|e| -> _ {
        CommandError::ArgumentError(ArgumentError {
            description: e.description,
        })
    })?;
    let Some(moves) = alg_to_moves(&alg) else {
        return Err(CommandError::ArgumentError(ArgumentError {
            description: "Non-moves used in the continuation alg.".to_owned(),
        }));
    };
    Ok(moves)
}

#[cfg(test)]
mod tests {
    use cubing::{alg::parse_alg, puzzles::cube3x3x3_kpuzzle};

    use crate::{
        _internal::cli::args::{GeneratorArgs, SearchCommandOptionalArgs},
        experimental_lib_api::search,
    };

    #[test]
    fn search_api_test() {
        let kpuzzle = cube3x3x3_kpuzzle();
        let search_pattern = kpuzzle
            .default_pattern()
            .apply_alg(parse_alg!("R U R'"))
            .expect("Invalid alg for puzzle.");
        let mut solutions =
            search(kpuzzle, &search_pattern, Default::default()).expect("Search failed.");
        assert_eq!(solutions.next().expect("No solutions.").nodes.len(), 3);

        let mut solutions = search(
            kpuzzle,
            &search_pattern,
            SearchCommandOptionalArgs {
                generator_args: GeneratorArgs {
                    generator_moves_string: Some("R,U".to_owned()), // TODO: make this semantic
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(solutions.next().unwrap().nodes.len(), 3);
    }
}
