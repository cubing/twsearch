use std::str::FromStr;

use magnus::{function, prelude::*, Error, Ruby};

use twips::scramble::{
    derive_scramble_for_event_seeded, random_scramble_for_event,
    scramble_finder::free_memory_for_all_scramble_finders, DerivationSalt, DerivationSeed, Event,
};

fn rb_random_scramble_for_event(ruby: &Ruby, event_str: String) -> Result<String, Error> {
    let event = Event::try_from(event_str.as_str())
        .map_err(|e| Error::new(ruby.exception_arg_error(), e.description))?;

    random_scramble_for_event(event)
        .map(|scramble| scramble.to_string())
        .map_err(|e| Error::new(ruby.exception_runtime_error(), e.description))
}

fn rb_derive_scramble_for_event(
    ruby: &Ruby,
    hex_derivation_seed_str: String,
    derivation_salt_hierarchy_str: Vec<String>,
    subevent_str: String,
) -> Result<String, Error> {
    let subevent = Event::try_from(subevent_str.as_str())
        .map_err(|e| Error::new(ruby.exception_arg_error(), e.description))?;

    let derivation_seed = DerivationSeed::from_str(&hex_derivation_seed_str)
        .map_err(|e| Error::new(ruby.exception_arg_error(), e))?;

    let hierarchy = derivation_salt_hierarchy_str
        .iter()
        .map(|s| s.as_str())
        .map(DerivationSalt::from_str)
        .collect::<Result<Vec<DerivationSalt>, String>>()
        .map_err(|e| Error::new(ruby.exception_arg_error(), e))?;

    derive_scramble_for_event_seeded(&derivation_seed, &hierarchy, subevent)
        .map(|scramble| scramble.to_string())
        .map_err(|e| Error::new(ruby.exception_runtime_error(), e))
}

fn rb_free_memory_for_all_scramble_finders() -> u32 {
    // We cast to `u32` for the public API so that it's more stable across environments (including WASM).
    // If we've allocated more than `u32::MAX` scramble finders, I'd be *very* impressed.
    free_memory_for_all_scramble_finders() as u32
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

    module.define_singleton_method(
        "free_memory_for_all_scramble_finders",
        function!(rb_free_memory_for_all_scramble_finders, 0),
    )?;

    Ok(())
}
