use std::str::FromStr;

use magnus::{function, prelude::*, Error, Ruby};

use twsearch::scramble::{random_scramble_for_event, derive_scramble_for_event_seeded, Event, DerivationSeed, DerivationSalt};

fn rb_random_scramble_for_event(ruby: &Ruby, event_str: String) -> Result<String, Error> {
    let event = Event::try_from(event_str.as_str())
        .map_err(|e| Error::new(ruby.exception_arg_error(), e.description))?;

    random_scramble_for_event(event)
        .map(|scramble| scramble.to_string())
        .map_err(|e| Error::new(ruby.exception_runtime_error(), e.description))
}

fn rb_derive_scramble_for_event(ruby: &Ruby, event_str: String, hex_derivation_seed_str: String, derivation_salt_hierarchy: Vec<String>) -> Result<String, Error> {
    let event = Event::try_from(event_str.as_str())
        .map_err(|e| Error::new(ruby.exception_arg_error(), e.description))?;

    let root_derivation_seed = DerivationSeed::from_str(&hex_derivation_seed_str)
        .map_err(|e| Error::new(ruby.exception_arg_error(), e))?;

    let derivation_seed = if derivation_salt_hierarchy.is_empty() {
        root_derivation_seed
    } else {
        let hierarchy: Vec<DerivationSalt> = derivation_salt_hierarchy.iter()
            .map(|s| s.as_str())
            .map(DerivationSalt::from_str)
            .collect::<Result<Vec<DerivationSalt>, String>>()
            .map_err(|e| Error::new(ruby.exception_arg_error(), e))?;

        root_derivation_seed.derive_hierarchy(&hierarchy)
    };

    derive_scramble_for_event_seeded(&derivation_seed, event)
        .map(|scramble| scramble.to_string())
        .map_err(|e| Error::new(ruby.exception_runtime_error(), e))
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Twsearch")?;

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
