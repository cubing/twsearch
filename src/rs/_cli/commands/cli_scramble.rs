use std::time::Instant;

use cubing::alg::{
    experimental_twizzle_link::{experimental_twizzle_link, ExperimentalTwizzleLinkParameters},
    Alg,
};
use twsearch::{
    _internal::{
        cli::args::{ScrambleArgs, ScrambleFinderArgs, ScrambleFinderCommand},
        errors::CommandError,
    },
    scramble::{random_scramble_for_event, scramble_finder_solve, Event},
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

pub fn cli_scramble_finder_solve(args: &ScrambleFinderArgs) -> Result<(), CommandError> {
    match &args.command {
        ScrambleFinderCommand::Solve(scramble_finder_solve_args) => {
            let event_id = scramble_finder_solve_args.event_id.as_str();
            let event = Event::try_from(event_id)?;

            let scramble_setup_alg = scramble_finder_solve_args
                .scramble_setup_alg
                .parse::<Alg>()
                .expect("Invalid alg");

            let current_scramble_start_time = Instant::now();
            let scramble = scramble_finder_solve(event, &scramble_setup_alg)?;
            eprintln!(
                "// Scramble found in: {:?}",
                Instant::now() - current_scramble_start_time
            );
            println!("{}", scramble);
            if scramble_finder_solve_args.print_link {
                let link = experimental_twizzle_link(ExperimentalTwizzleLinkParameters {
                    setup: Some(&scramble_setup_alg),
                    alg: Some(&scramble),
                    puzzle: Some(event.puzzle().id()),
                    ..Default::default()
                });
                println!("{}", link);
            }
        }
    };
    Ok(())
}
