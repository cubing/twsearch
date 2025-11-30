use twips::{
    _internal::{
        errors::TwipsError, search::iterative_deepening::individual_search::IndividualSearchOptions,
    },
    experimental_lib_api::{search, KPuzzleSource, PatternSource, SearchOptions},
};

use crate::args::SearchCommandArgs;

pub fn cli_search(search_command_args: SearchCommandArgs) -> Result<(), TwipsError> {
    let search_start_time: std::time::Instant = instant::Instant::now();
    let kpuzzle: KPuzzleSource = search_command_args.def_args.def_args.into();
    let kpuzzle = kpuzzle.kpuzzle()?;

    let search_pattern: PatternSource = search_command_args
        .optional
        .scramble_and_target_pattern_optional_args
        .search_pattern();
    let solutions = search(
        &kpuzzle,
        &search_pattern
            .kpattern(&kpuzzle)?
            .unwrap_or_else(|| kpuzzle.default_pattern()),
        SearchOptions {
            // TODO: allow mapping to `None`
            target_pattern: search_command_args
                .optional
                .scramble_and_target_pattern_optional_args
                .target_pattern()
                .kpattern(&kpuzzle)?,
            generators: search_command_args.optional.generator_args.generators(),
            metric: search_command_args.optional.metric_args.metric,
            random_start: Some(search_command_args.optional.search_args.random_start),
            verbosity: search_command_args.optional.verbosity_args.verbosity,
            individual_search_options: IndividualSearchOptions {
                min_num_solutions: search_command_args.optional.min_num_solutions,
                min_depth_inclusive: search_command_args.optional.search_args.min_depth,
                max_depth_exclusive: search_command_args.optional.search_args.max_depth,
                canonical_fsm_pre_moves: None,
                canonical_fsm_post_moves: None,
                root_continuation_condition: search_command_args
                    .optional
                    .search_args
                    .continuation_condition()?,
            },
        },
    )?;
    let mut solution_index = 0;
    for solution in solutions {
        solution_index += 1;
        println!(
            "{} // solution #{} ({} nodes)",
            solution,
            solution_index,
            solution.nodes.len()
        )
    }
    eprintln!(
        "// Entire search duration: {:?}",
        instant::Instant::now() - search_start_time
    );
    Ok(())
}
