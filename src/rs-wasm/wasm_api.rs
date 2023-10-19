use wasm_bindgen::prelude::*;

use twsearch::scramble::{random_scramble_for_event, Event};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

pub fn internal_init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn wasmRandomScrambleForEvent(event_str: String) -> Result<String, String> {
    internal_init();

    let event = Event::try_from(event_str.as_str()).map_err(|e| e.description)?;
    match random_scramble_for_event(event) {
        Ok(scramble) => Ok(scramble.to_string()),
        Err(e) => Err(e.description),
    }
}
