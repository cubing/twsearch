use twips::{
    _internal::{
        cli::args::DeriveArgs,
        errors::{ArgumentError, CommandError},
    },
    scramble::{derive_scramble_for_event_seeded, Event},
};

pub fn cli_derive(args: &DeriveArgs) -> Result<(), CommandError> {
    if args.root_derivation_seed.level() != 0 {
        return Err(Into::<ArgumentError>::into(
            "Root derivation seed must be at level 0 (second byte must be `0x00`).",
        )
        .into());
    }

    // Validate the event arg.
    let _ = Event::try_from(args.derivation_salts[2].unhashed_salt().as_str())?;

    let subevent = Event::try_from(args.derivation_salts[6].unhashed_salt().as_str())?;
    println!(
        "{}",
        derive_scramble_for_event_seeded(
            &args.root_derivation_seed,
            &args.derivation_salts,
            subevent
        )
        .unwrap()
    );
    Ok(())
}
