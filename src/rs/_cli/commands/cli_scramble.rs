use cubing::alg::Alg;
use twsearch::{
    _internal::{
        cli::args::{ScrambleArgs, TestScrambleArgs},
        errors::CommandError,
    },
    scramble::{random_scramble_for_event, test_random_scramble, Event},
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

pub fn cli_test_scramble(args: &TestScrambleArgs) -> Result<(), CommandError> {
    let l = args.event_id.as_str();
    let a = Event::try_from(l);
    let event = a?;

    let test_scramble_alg = args.scramble_setup_alg.parse::<Alg>().expect("Invalid alg");

    let scramble = test_random_scramble(event, &test_scramble_alg)?;
    println!("{}", scramble);

    Ok(())
}
