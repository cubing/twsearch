use std::str::FromStr;

use twsearch::{
    _internal::{
        cli::args::DeriveArgs,
        errors::{ArgumentError, CommandError},
    },
    scramble::{derive_scramble_for_event_seeded, DerivationSalt, Event},
};

pub fn cli_derive(args: &DeriveArgs) -> Result<(), CommandError> {
    if args.root_derivation_seed.level() != 0 {
        return Err(Into::<ArgumentError>::into(
            "Root derivation seed must be at level 0 (second byte must be `0x00`).",
        )
        .into());
    }

    let derivation_seed = args
        .root_derivation_seed
        .derive_hierarchy(&args.derivation_salts)
        .derive(&DerivationSalt::from_str(args.event_id.as_str()).unwrap());

    println!(
        "{}",
        derive_scramble_for_event_seeded(
            &derivation_seed,
            Event::try_from(args.event_id.as_str()).unwrap()
        )
        .unwrap()
    );
    Ok(())
}
