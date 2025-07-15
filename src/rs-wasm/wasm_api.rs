use cubing::alg::Move;
use cubing::kpuzzle::{KPattern, KPatternData, KPuzzle};
use serde::{Deserialize, Serialize};
use twsearch::_internal::cli::args::{CustomGenerators, Generators};
use twsearch::_internal::search::iterative_deepening::individual_search::IndividualSearchOptions;
use twsearch::_internal::search::iterative_deepening::iterative_deepening_search::{
    ImmutableSearchData, IterativeDeepeningSearch,
};
use twsearch::scramble::scramble_finder::free_memory_for_all_scramble_finders;
use wasm_bindgen::prelude::*;

use twsearch::scramble::{random_scramble_for_event, Event};

pub fn internal_init() {
    console_error_panic_hook::set_once();
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WasmTwsearchOptions {
    target_pattern: Option<KPatternData>,
    generator_moves: Option<Vec<Move>>,

    #[serde(flatten)]
    inidividual_search_options: IndividualSearchOptions,
}

#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn wasmTwsearch(
    kpuzzle_json: String,
    search_pattern_json: String,
    options_json: String, // TODO
) -> Result<String, String> {
    internal_init();

    let kpuzzle = KPuzzle::try_from_json(kpuzzle_json.as_bytes());
    let kpuzzle = kpuzzle.map_err(|e| e.to_string())?;

    let search_pattern = KPattern::try_from_json(&kpuzzle, search_pattern_json.as_bytes());
    let search_pattern = search_pattern.map_err(|e| e.to_string())?;

    let options: WasmTwsearchOptions = match serde_json::from_slice(options_json.as_bytes()) {
        Ok(options) => options,
        Err(e) => return Err(e.to_string()),
    };
    if options
        .inidividual_search_options
        .min_num_solutions
        .is_some()
    {
        return Err("`minNumSolutions` is not implemented yet".to_owned());
    }

    let target_pattern = match options.target_pattern {
        Some(target_pattern_data) => {
            let target_pattern = KPattern::try_from_data(&kpuzzle, &target_pattern_data);
            target_pattern.map_err(|e| e.to_string())?
        }
        None => kpuzzle.default_pattern(),
    };
    let generators = match options.generator_moves {
        Some(generator_moves) => Generators::Custom(CustomGenerators {
            moves: generator_moves,
            algs: vec![],
        }),
        None => Generators::Default,
    };

    let mut iterative_deepening_search =
        <IterativeDeepeningSearch<KPuzzle>>::new_with_hash_prune_table(
            ImmutableSearchData::try_from_common_options_with_auto_search_generators(
                kpuzzle.clone(),
                generators.enumerate_moves_for_kpuzzle(&kpuzzle),
                vec![target_pattern], // TODO: support multiple target patterns.
                Default::default(),
            )
            .map_err(|e| e.description)?,
            Default::default(), // StoredSearchAdaptations::default(),
            Default::default(), // HashPruneTableSizeBounds::default(),
        );

    match iterative_deepening_search
        .search(
            &search_pattern,
            options.inidividual_search_options,
            Default::default(),
        )
        .next()
    {
        Some(alg) => Ok(alg.to_string().to_owned()),
        None => Err("No solution found!".to_owned()),
    }
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

#[wasm_bindgen]
#[allow(non_snake_case)]
pub extern "C" fn wasmFreeMemoryForAllScrambleFinders() -> u32 {
    // We cast to `u32` for the public API so that it's more stable across environments (including WASM).
    // If we've allocated more than `u32::MAX` scramble finders, I'd be *very* impressed.
    free_memory_for_all_scramble_finders() as u32
}
