use std::sync::Arc;

use cubing::{
    alg::{Alg, Move},
    parse_alg,
    puzzles::cube3x3x3_kpuzzle,
};
use wasm_bindgen::prelude::*;

use crate::{
    utils::set_panic_hook, IDFSearch, PackedKPuzzle, SearchLogger, _internal::cli::VerbosityLevel,
};

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
pub fn search_test() -> String {
    let kpuzzle = cube3x3x3_kpuzzle();
    let packed_kpuzzle =
        PackedKPuzzle::try_from(kpuzzle).expect("Could not create `packed_kpuzzle");

    let move_list = vec![
        "U".parse::<Move>().expect("Could not parse move"),
        "F".parse::<Move>().expect("Could not parse move"),
        "R".parse::<Move>().expect("Could not parse move"),
    ];

    let target_pattern = packed_kpuzzle.default_pattern();
    let search_pattern = target_pattern.apply_transformation(
        &packed_kpuzzle
            .transformation_from_alg(&"L' U' L F U2 R".parse::<Alg>().unwrap())
            .expect("Could not create search pattern."),
    );

    let idf_search = IDFSearch::try_new(
        packed_kpuzzle,
        target_pattern.clone(),
        move_list,
        Arc::new(SearchLogger {
            verbosity: VerbosityLevel::Info,
        }),
    )
    .expect("Could not construct search.");

    match idf_search
        .search(&search_pattern, crate::IndividualSearchOptions::default())
        .next()
    {
        Some(alg) => alg.to_string(),
        None => "// no solution?".to_owned(),
    }
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
