// We explictly use `pub use` instead of `pub mod` here so that it's possible to
// tell from this single file exactly what is exported (and impossible to
// accidentally export more).

mod common;
pub use common::KPuzzleSource; // TODO
pub use common::PatternSource; // TODO

mod search_api;
pub use search_api::search;

mod gods_algorithm_api;
pub use gods_algorithm_api::gods_algorithm;

mod multi_phase_search;
pub use multi_phase_search::{
    KPuzzleSimpleMaskPhase, KPuzzleSimpleMaskPhaseConstructionOptions, MultiPhaseSearch,
    SearchPhase,
};
