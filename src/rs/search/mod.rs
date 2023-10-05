#[allow(clippy::module_inception)]
mod search;
pub use search::IDFSearch;

mod prune_table;
pub use prune_table::*;

mod recursive_work_tracker;
pub(crate) use recursive_work_tracker::*;
