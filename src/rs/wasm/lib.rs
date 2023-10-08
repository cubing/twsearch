mod utils;

use cubing::parse_alg;
use utils::set_panic_hook;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, cubing-rust-wasm!");
}

#[wasm_bindgen]
pub fn internal_init() {
    set_panic_hook()
}

#[wasm_bindgen]
pub fn invert_alg(alg_str: String) -> Result<String, String> {
    let parsed = parse_alg!(alg_str).map_err(|e| e.description)?;
    Ok(parsed.invert().to_string())
}
