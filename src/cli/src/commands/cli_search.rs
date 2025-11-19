use twips::{
    _internal::{cli::args::SearchCommandArgs, errors::CommandError},
    experimental_lib_api::{search, KPuzzleSource, PatternSource},
};

pub fn cli_search(search_command_args: SearchCommandArgs) -> Result<(), CommandError> {
    let search_start_time: std::time::Instant = instant::Instant::now();
    let kpuzzle =
        KPuzzleSource::from_clap_args(&search_command_args.def_args.def_args).kpuzzle()?;
    let search_pattern = PatternSource::search_pattern_from_clap_args(
        &search_command_args
            .optional
            .scramble_and_target_pattern_optional_args,
    )?
    .pattern(&kpuzzle)?;
    let solutions = search(&kpuzzle, &search_pattern, search_command_args.optional)?;
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
