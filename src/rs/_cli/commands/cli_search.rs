use twsearch::{
    _internal::{cli::args::SearchCommandArgs, errors::CommandError},
    experimental_lib_api::search,
};

pub fn cli_search(search_command_args: SearchCommandArgs) -> Result<(), CommandError> {
    let search_start_time = instant::Instant::now();
    let solutions = search(search_command_args)?;
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
