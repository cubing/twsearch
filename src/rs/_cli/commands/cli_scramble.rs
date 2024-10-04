use twsearch::{
    _internal::{options::ScrambleArgs, CommandError},
    scramble::{random_scramble_for_event, Event},
};

pub fn cli_scramble(args: &ScrambleArgs) -> Result<(), CommandError> {
    let l = args.event_id.as_str();
    let a = Event::try_from(l);
    let event = a?;

    let scramble = random_scramble_for_event(event)?;
    println!("{}", scramble);

    Ok(())
}
