use cubing::alg::Alg;
use twsearch::{
    _internal::{
        cli::args::{ScrambleArgs, ScrambleFinderArgs, ScrambleFinderCommand},
        errors::CommandError,
    },
    scramble::{random_scramble_for_event, scramble_finder_solve, Event},
};

pub fn cli_scramble(args: &ScrambleArgs) -> Result<(), CommandError> {
    let l = args.event_id.as_str();
    let a = Event::try_from(l);
    let event = a?;

    for _ in 0..args.amount {
        let scramble = random_scramble_for_event(event)?;
        println!("{}", scramble);
    }

    Ok(())
}

pub fn cli_scramble_finder_solve(args: &ScrambleFinderArgs) -> Result<(), CommandError> {
    match &args.command {
        ScrambleFinderCommand::Solve(scramble_finder_solve_args) => {
            let l = scramble_finder_solve_args.event_id.as_str();
            let a = Event::try_from(l);
            let event = a?;

            let test_scramble_alg = scramble_finder_solve_args
                .scramble_setup_alg
                .parse::<Alg>()
                .expect("Invalid alg");

            let scramble = scramble_finder_solve(event, &test_scramble_alg)?;
            println!("{}", scramble);
        }
    };
    Ok(())
}
