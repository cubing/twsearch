use twsearch::{
    _internal::{cli::args::ScrambleArgs, errors::CommandError},
    scramble::{random_scramble_for_event, Event},
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
