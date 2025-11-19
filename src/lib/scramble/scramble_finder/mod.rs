#[allow(clippy::module_inception)]
pub mod scramble_finder;
pub mod solving_based_scramble_finder;

pub mod random_move_scramble_finder;
// pub use solving_based_scramble_finder::free_memory_for_all_scramble_finders;

use random_move_scramble_finder::free_memory_for_all_random_move_scramble_finders;
use solving_based_scramble_finder::free_memory_for_all_solving_based_scramble_finders;

pub fn free_memory_for_all_scramble_finders() -> usize {
    free_memory_for_all_solving_based_scramble_finders()
        + free_memory_for_all_random_move_scramble_finders()
}
