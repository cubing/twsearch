use twsearch::{
    _internal::{cli::args::SearchCommandArgs, errors::CommandError},
    experimental_lib_api::{search, KPuzzleDefinitionSource, PatternSource},
};

pub fn cli_search(search_command_args: SearchCommandArgs) -> Result<(), CommandError> {
    let search_start_time: std::time::Instant = instant::Instant::now();
    let definition = KPuzzleDefinitionSource::from_clap_args(
        &search_command_args.def_and_optional_scramble_args.def_args,
    );
    let search_pattern = PatternSource::search_pattern_from_clap_args(
        &search_command_args.def_and_optional_scramble_args,
    )?;
    let solutions = search(definition, search_pattern, search_command_args)?;
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
    println!(
        "// Entire search duration: {:?}",
        instant::Instant::now() - search_start_time
    );
    Ok(())
}
