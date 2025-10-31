use std::time::Instant;

use cubing::alg::experimental_twizzle_link::{
    experimental_twizzle_link, ExperimentalTwizzleLinkParameters,
};
use twsearch::{
    _internal::{
        cli::args::{
            ScrambleArgs, ScrambleFinderArgs, ScrambleFinderCommand, SolveKnownPuzzleCommandArgs,
        },
        errors::CommandError,
    },
    scramble::{
        experimental_scramble_finder_filter_and_or_search, random_scramble_for_event,
        solve_known_puzzle, Event, ExperimentalFilterAndOrSearchOptions,
    },
};

pub fn cli_scramble(args: &ScrambleArgs) -> Result<(), CommandError> {
    let event_id = args.event_id.as_str();
    let event = Event::try_from(event_id)?;

    let total_start_time = Instant::now();
    for i in 1..=args.amount {
        let current_scramble_start_time = Instant::now();
        let scramble = random_scramble_for_event(event)?;
        eprintln!(
            "// Scramble #{} found in: {:?}",
            i,
            Instant::now() - current_scramble_start_time
        );
        println!("{}", scramble);
        let elapsed_duration = Instant::now() - total_start_time;
        eprintln!(
            "Found {} scramble{} in {:?} so far (average: {:?} per scramble)",
            i,
            if i == 1 { "" } else { "s" },
            elapsed_duration,
            elapsed_duration / (i as u32)
        );
    }

    Ok(())
}

// TODO: refactor this once `experimental_scramble_finder_filter_and_or_search` can support a less gnarly API.
pub fn cli_scramble_finder(args: &ScrambleFinderArgs) -> Result<(), CommandError> {
    let (filter_args, apply_filtering, perform_search) = match &args.command {
        ScrambleFinderCommand::Search(scramble_finder_solve_args) => (
            &scramble_finder_solve_args.filter_args,
            scramble_finder_solve_args.apply_filtering,
            true,
        ),
        ScrambleFinderCommand::Filter(scramble_finder_filter_args) => {
            (scramble_finder_filter_args, true, false)
        }
    };

    let event_id = filter_args.event_id.as_str();
    let event = Event::try_from(event_id)?;

    let scramble_setup_alg = &filter_args.scramble_setup_alg.clone();

    let current_scramble_start_time = Instant::now();
    let scramble = experimental_scramble_finder_filter_and_or_search(
        event,
        &ExperimentalFilterAndOrSearchOptions {
            scramble_setup_alg: &filter_args.scramble_setup_alg,
            apply_filtering,
            perform_search,
        },
    )?;

    match &args.command {
        ScrambleFinderCommand::Search(scramble_finder_solve_args) => {
            eprintln!(
                "// Scramble found in: {:?}",
                Instant::now() - current_scramble_start_time
            );
            let scramble = scramble.unwrap();
            println!("{}", scramble);
            if matches!(scramble_finder_solve_args.print_link, Some(true)) {
                let link = experimental_twizzle_link(ExperimentalTwizzleLinkParameters {
                    setup: Some(scramble_setup_alg),
                    alg: Some(&scramble),
                    puzzle: Some(event.puzzle().id()),
                    ..Default::default()
                });
                println!("{}", link);
            }
        }
        ScrambleFinderCommand::Filter(_scramble_finder_filter_args) => {}
    };

    Ok(())
}

pub fn cli_solve_known_puzzle(
    search_command_args: SolveKnownPuzzleCommandArgs,
) -> Result<(), CommandError> {
    let solution = solve_known_puzzle(
        search_command_args.puzzle,
        &search_command_args.scramble_setup_alg,
    )
    .unwrap()
    .unwrap();

    println!("{}", solution);
    if matches!(search_command_args.print_link, Some(true)) {
        eprintln!(
            "{}",
            experimental_twizzle_link(ExperimentalTwizzleLinkParameters {
                setup: Some(&search_command_args.scramble_setup_alg),
                alg: Some(&solution),
                puzzle: Some(search_command_args.puzzle.id()),
                ..Default::default()
            })
        );
    }

    Ok(())
}
