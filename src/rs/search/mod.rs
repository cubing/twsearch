#[allow(clippy::module_inception)]
mod search;
pub use search::IDFSearch;

mod prune_table;
pub use prune_table::*;
