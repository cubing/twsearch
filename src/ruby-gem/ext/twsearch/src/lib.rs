use magnus::{function, prelude::*, Error, Ruby};

use twsearch::scramble::{random_scramble_for_event, Event};

fn rb_random_scramble_for_event(ruby: &Ruby, event_str: String) -> Result<String, Error> {
    let event = Event::try_from(event_str.as_str())
        .map_err(|e| Error::new(ruby.exception_arg_error(), e.description))?;

    random_scramble_for_event(event)
        .map(|scramble| scramble.to_string())
        .map_err(|e| Error::new(ruby.exception_runtime_error(), e.description))
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Twsearch")?;

    module.define_singleton_method(
        "random_scramble_for_event",
        function!(rb_random_scramble_for_event, 1),
    )?;

    Ok(())
}
