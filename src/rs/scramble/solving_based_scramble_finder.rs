use std::sync::{LazyLock, Mutex, RwLock};

use cubing::alg::Alg;
use erased_set::ErasedSyncSet;

use crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle;

pub enum FilteringDecision {
    Accept,
    Reject,
}

// pub struct NoScrambleOptions {}

pub trait SolvingBasedScrambleFinder: Default {
    type TPuzzle: SemiGroupActionPuzzle;
    type ScrambleAssociatedData;
    type ScrambleOptions;

    fn generate_fair_unfiltered_random_pattern(
        &mut self,
        scramble_options: &Self::ScrambleOptions,
    ) -> (
        <<Self as SolvingBasedScrambleFinder>::TPuzzle as SemiGroupActionPuzzle>::Pattern,
        Self::ScrambleAssociatedData,
    );

    fn filter_pattern(
        &mut self,
        pattern: &<<Self as SolvingBasedScrambleFinder>::TPuzzle as SemiGroupActionPuzzle>::Pattern,
        scramble_associated_data: &Self::ScrambleAssociatedData,
        scramble_options: &Self::ScrambleOptions,
    ) -> FilteringDecision;

    fn solve_pattern(
        &mut self,
        pattern: &<<Self as SolvingBasedScrambleFinder>::TPuzzle as SemiGroupActionPuzzle>::Pattern,
        scramble_associated_data: &Self::ScrambleAssociatedData,
        scramble_options: &Self::ScrambleOptions,
    ) -> Alg;

    fn collapse_inverted_alg(&mut self, alg: Alg) -> Alg;

    fn generate_fair_scramble(&mut self, scramble_options: &Self::ScrambleOptions) -> Alg {
        loop {
            let (pattern, scramble_associated_data) =
                self.generate_fair_unfiltered_random_pattern(scramble_options);
            if matches!(
                self.filter_pattern(&pattern, &scramble_associated_data, scramble_options),
                FilteringDecision::Reject
            ) {
                continue;
            }
            let inverse_scramble =
                self.solve_pattern(&pattern, &scramble_associated_data, scramble_options);
            return self.collapse_inverted_alg(inverse_scramble.invert());
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

pub fn free_memory_for_all_scramble_finders() {
    ScrambleFinderCacher::free_memory_for_all_scramble_finders();
}

#[derive(Default)]
struct ScrambleFinderCacher {
    erased_sync_set: ErasedSyncSet,
}

static SCRAMBLE_FINDER_CACHER_SINGLETON: LazyLock<Mutex<ScrambleFinderCacher>> =
    LazyLock::<Mutex<ScrambleFinderCacher>>::new(Default::default);

impl ScrambleFinderCacher {
    pub fn generate_fair_scramble<
        ScrambleFinder: SolvingBasedScrambleFinder + 'static + Sync + Send,
    >(
        scramble_options: &ScrambleFinder::ScrambleOptions,
    ) -> Alg {
        let mut mutex_guard = SCRAMBLE_FINDER_CACHER_SINGLETON.lock().unwrap();
        mutex_guard
            .erased_sync_set
            .get_or_insert_with(|| RwLock::new(ScrambleFinder::default()));

        let rw_lock = mutex_guard
            .erased_sync_set
            .get_mut::<RwLock<ScrambleFinder>>()
            .unwrap();

        rw_lock
            .get_mut()
            .unwrap()
            .generate_fair_scramble(scramble_options)
    }

    // pub fn free_memory_for_scramble_finder<
    //     ScrambleFinder: SolvingBasedScrambleFinder + 'static + Sync + Send,
    // >() {
    //     let mut mutex_guard = SINGLETON.lock().unwrap();
    //     mutex_guard.erased_sync_set.remove::<ScrambleFinder>();
    // }

    pub fn free_memory_for_all_scramble_finders() {
        let mut mutex_guard = SCRAMBLE_FINDER_CACHER_SINGLETON.lock().unwrap();
        mutex_guard.erased_sync_set.clear();
    }
}
