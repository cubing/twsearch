use std::sync::{LazyLock, Mutex, RwLock};

use cubing::alg::Alg;
use erased_set::ErasedSyncSet;

use crate::{
    _internal::puzzle_traits::puzzle_traits::HasDefaultPattern,
    scramble::apply_flat_alg::apply_flat_alg,
};

use super::scramble_finder::ScrambleFinder;

pub trait RandomMoveScrambleFinder: ScrambleFinder {
    fn generate_unfiltered_random_move_scramble(
        &mut self,
        scramble_options: &Self::ScrambleOptions,
    ) -> Alg;

    fn puzzle(&self) -> &Self::TPuzzle;

    fn generate_filtered_random_move_scramble(
        &mut self,
        scramble_options: &Self::ScrambleOptions,
    ) -> Alg {
        loop {
            let scramble_alg = self.generate_unfiltered_random_move_scramble(scramble_options);
            let puzzle = self.puzzle();
            let pattern =
                apply_flat_alg(puzzle, &puzzle.puzzle_default_pattern(), &scramble_alg).unwrap();
            if self.filter_pattern(&pattern, scramble_options).is_reject() {
                continue;
            }
            return scramble_alg;
        }
    }
}

#[derive(Default)]
struct RandomMoveScrambleFinderCacher {
    erased_sync_set: ErasedSyncSet,
}

static RANDOM_MOVE_SCRAMBLE_FINDER_CACHER_SINGLETON: LazyLock<
    Mutex<RandomMoveScrambleFinderCacher>,
> = LazyLock::<Mutex<RandomMoveScrambleFinderCacher>>::new(Default::default);

pub fn generate_filtered_random_move_scramble<
    ScrambleFinder: RandomMoveScrambleFinder + 'static + Sync + Send,
>(
    scramble_options: &ScrambleFinder::ScrambleOptions,
) -> Alg {
    RandomMoveScrambleFinderCacher::generate_filtered_random_move_scramble::<ScrambleFinder>(
        scramble_options,
    )
}

pub(crate) fn free_memory_for_all_random_move_scramble_finders() -> usize {
    RandomMoveScrambleFinderCacher::free_memory_for_all_random_move_scramble_finders()
}

impl RandomMoveScrambleFinderCacher {
    // TODO: deduplicate with SolvingBasedScrambleFinderCacher?
    pub fn map<
        ScrambleFinder: RandomMoveScrambleFinder + 'static + Sync + Send,
        ReturnValue,
        F: Fn(&mut ScrambleFinder) -> ReturnValue,
    >(
        f: F,
    ) -> ReturnValue {
        // TODO: figure out how to share a concrete implementation instead of template code.
        // This is trickier than it sounds: https://stackoverflow.com/questions/40095383/how-to-return-a-reference-to-a-sub-value-of-a-value-that-is-under-a-mutex/40103840#40103840
        // There are some crates to do this, but not an obvious choice.
        let mut mutex_guard = RANDOM_MOVE_SCRAMBLE_FINDER_CACHER_SINGLETON.lock().unwrap();

        mutex_guard
            .erased_sync_set
            .get_or_insert_with(|| RwLock::new(ScrambleFinder::default()));

        let rw_lock = mutex_guard
            .erased_sync_set
            .get_mut::<RwLock<ScrambleFinder>>()
            .unwrap();

        let scramble_finder = rw_lock.get_mut().unwrap();
        /********/
        f(scramble_finder)
        // scramble_finder.generate_fair_scramble(scramble_options)
    }

    pub fn generate_filtered_random_move_scramble<
        ScrambleFinder: RandomMoveScrambleFinder + 'static + Sync + Send,
    >(
        scramble_options: &ScrambleFinder::ScrambleOptions,
    ) -> Alg {
        RandomMoveScrambleFinderCacher::map(|scramble_finder: &mut ScrambleFinder| {
            scramble_finder.generate_filtered_random_move_scramble(scramble_options)
        })
    }

    // pub fn free_memory_for_scramble_finder<
    //     ScrambleFinder: SolvingBasedScrambleFinder + 'static + Sync + Send,
    // >() {
    //     let mut mutex_guard = SINGLETON.lock().unwrap();
    //     mutex_guard.erased_sync_set.remove::<ScrambleFinder>();
    // }

    /// Returns the number of scramble finders freed.
    /// Note that some events share scramble finders, so this will not necessarily match the number of events scrambles have been generated for.
    pub fn free_memory_for_all_random_move_scramble_finders() -> usize {
        let mut mutex_guard = RANDOM_MOVE_SCRAMBLE_FINDER_CACHER_SINGLETON.lock().unwrap();
        let num_freed = mutex_guard.erased_sync_set.len();
        mutex_guard.erased_sync_set.clear();
        // The number of freed scramble finders is not super useful in itself, but it can be a useful sense check that scramble finders were actually allocated and freed.
        num_freed
    }
}
