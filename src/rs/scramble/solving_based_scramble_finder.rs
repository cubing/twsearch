use std::sync::{LazyLock, Mutex, RwLock};

use cubing::alg::Alg;
use erased_set::ErasedSyncSet;

use crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle;

pub enum FilteringDecision {
    Accept,
    Reject,
}

pub struct NoScrambleOptions {}

pub trait SolvingBasedScrambleFinder {
    type TPuzzle: SemiGroupActionPuzzle;
    type ScrambleOptions;

    fn new() -> Self;
    fn generate_fair_unfiltered_random_pattern(
        &mut self,
    ) -> <<Self as SolvingBasedScrambleFinder>::TPuzzle as SemiGroupActionPuzzle>::Pattern;
    fn filter_pattern(
        &mut self,
        pattern: &<<Self as SolvingBasedScrambleFinder>::TPuzzle as SemiGroupActionPuzzle>::Pattern,
        scramble_options: &Self::ScrambleOptions,
    ) -> FilteringDecision;
    fn solve_pattern(
        &mut self,
        pattern: &<<Self as SolvingBasedScrambleFinder>::TPuzzle as SemiGroupActionPuzzle>::Pattern,
        scramble_options: &Self::ScrambleOptions,
    ) -> Alg;
    fn invert_alg_to_use_as_scramble(
        &mut self,
        alg: Alg,
        scramble_options: &Self::ScrambleOptions,
    ) -> Alg;

    fn generate_fair_scramble(&mut self, scramble_options: &Self::ScrambleOptions) -> Alg {
        loop {
            let pattern = self.generate_fair_unfiltered_random_pattern();
            if matches!(
                self.filter_pattern(&pattern, scramble_options),
                FilteringDecision::Reject
            ) {
                continue;
            }
            let inverse_scramble = self.solve_pattern(&pattern, scramble_options);
            return self.invert_alg_to_use_as_scramble(inverse_scramble, scramble_options);
        }
    }
}

pub fn generate_fair_scramble<
    ScrambleFinder: SolvingBasedScrambleFinder + 'static + Sync + Send,
>(
    scramble_options: &ScrambleFinder::ScrambleOptions,
) -> Alg {
    ScrambleFinderCacher::generate_fair_scramble::<ScrambleFinder>(scramble_options)
}

#[derive(Default)]
struct ScrambleFinderCacher {
    erased_sync_set: ErasedSyncSet,
}

static SINGLETON: LazyLock<Mutex<ScrambleFinderCacher>> =
    LazyLock::<Mutex<ScrambleFinderCacher>>::new(Default::default);

impl ScrambleFinderCacher {
    pub fn generate_fair_scramble<
        ScrambleFinder: SolvingBasedScrambleFinder + 'static + Sync + Send,
    >(
        scramble_options: &ScrambleFinder::ScrambleOptions,
    ) -> Alg {
        let mut mutex_guard = SINGLETON.lock().unwrap();
        mutex_guard
            .erased_sync_set
            .get_or_insert_with(|| RwLock::new(ScrambleFinder::new()));

        let rw_lock = mutex_guard
            .erased_sync_set
            .get_mut::<RwLock<ScrambleFinder>>()
            .unwrap();

        rw_lock
            .get_mut()
            .unwrap()
            .generate_fair_scramble(scramble_options)
    }

    pub fn free_memory_for_scramble_finder<
        ScrambleFinder: SolvingBasedScrambleFinder + 'static + Sync + Send,
    >() {
        let mut mutex_guard = SINGLETON.lock().unwrap();
        mutex_guard.erased_sync_set.remove::<ScrambleFinder>();
    }

    pub fn clear<ScrambleFinder: SolvingBasedScrambleFinder + 'static + Sync + Send>() {
        let mut mutex_guard = SINGLETON.lock().unwrap();
        mutex_guard.erased_sync_set.clear();
    }
}
