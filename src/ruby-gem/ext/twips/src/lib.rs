use std::str::FromStr;

use magnus::{function, prelude::*, Error, Ruby};

use twips::scramble::{random_scramble_for_event, derive_scramble_for_event_seeded, Event, DerivationSeed, DerivationSalt};

fn rb_random_scramble_for_event(ruby: &Ruby, event_str: String) -> Result<String, Error> {
    let event = Event::try_from(event_str.as_str())
        .map_err(|e| Error::new(ruby.exception_arg_error(), e.description))?;

    random_scramble_for_event(event)
        .map(|scramble| scramble.to_string())
        .map_err(|e| Error::new(ruby.exception_runtime_error(), e.description))
}

fn rb_derive_scramble_for_event(ruby: &Ruby, hex_derivation_seed_str: String, derivation_salt_hierarchy_str: String, subevent_str: String) -> Result<String, Error> {
    let subevent = Event::try_from(subevent_str.as_str())
        .map_err(|e| Error::new(ruby.exception_arg_error(), e.description))?;

    let derivation_seed = DerivationSeed::from_str(&hex_derivation_seed_str)
        .map_err(|e| Error::new(ruby.exception_arg_error(), e))?;

    let hierarchy = if derivation_salt_hierarchy_str.is_empty() {
        vec![]
    } else {
        derivation_salt_hierarchy_str
            .split("/")
            .map(DerivationSalt::from_str)
            .collect::<Result<Vec<DerivationSalt>, String>>()
            .map_err(|e| Error::new(ruby.exception_arg_error(), e))?
    };

    derive_scramble_for_event_seeded(&derivation_seed, &hierarchy, subevent)
        .map(|scramble| scramble.to_string())
        .map_err(|e| Error::new(ruby.exception_runtime_error(), e))
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Twips")?;

    module.define_singleton_method(
        "random_scramble_for_event",
        function!(rb_random_scramble_for_event, 1),
    )?;

    module.define_singleton_method(
        "derive_scramble_for_event",
        function!(rb_derive_scramble_for_event, 3),
    )?;

    Ok(())
}
