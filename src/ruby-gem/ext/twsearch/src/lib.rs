use magnus::{define_module, function, prelude::*, Error};

use twsearch::scramble::{random_scramble_for_event, Event};

#[no_mangle]
fn rb_random_scramble_for_event(event_str: String) -> Result<String, Error> {
    let event = Event::try_from(event_str.as_str())
        .map_err(|e| Error::new(magnus::exception::arg_error(), e.description))?;

    random_scramble_for_event(event)
        .map(|scramble| scramble.to_string())
        .map_err(|e| Error::new(magnus::exception::runtime_error(), e.description))
}

#[magnus::init]
fn init() -> Result<(), Error> {
    let module = define_module("Twsearch")?;

    module.define_singleton_method(
        "random_scramble_for_event",
        function!(rb_random_scramble_for_event, 1),
    )?;

    Ok(())
}
