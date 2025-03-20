use std::sync::{LazyLock, Mutex, RwLock};

use cubing::alg::Alg;
use erased_set::ErasedSyncSet;

use crate::_internal::{
    errors::SearchError, puzzle_traits::puzzle_traits::SemiGroupActionPuzzle,
    search::filtering_decision::FilteringDecision,
};

#[derive(Default)]
pub struct NoScrambleAssociatedData {}

#[derive(Default)]
pub struct NoScrambleOptions {}

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
    ) -> Result<Alg, SearchError>;

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
            // Since we got the pattern from the trait implementation, it should be safe to `.unwrap()` — else, the trait implementation is broken.
            // TODO: are there any puzzles for which we may want to change this?
            let inverse_scramble = self
                .solve_pattern(&pattern, &scramble_associated_data, scramble_options)
                .unwrap();
            return self.collapse_inverted_alg(inverse_scramble.invert());
        }
    }
}

pub fn scramble_finder_cacher_map<
    ScrambleFinder: SolvingBasedScrambleFinder + 'static + Sync + Send,
    ReturnValue,
    F: Fn(&mut ScrambleFinder) -> ReturnValue,
>(
    f: F,
) -> ReturnValue {
    ScrambleFinderCacher::map::<ScrambleFinder, ReturnValue, F>(f)
}

pub fn generate_fair_scramble<
    ScrambleFinder: SolvingBasedScrambleFinder + 'static + Sync + Send,
>(
    scramble_options: &ScrambleFinder::ScrambleOptions,
) -> Alg {
    ScrambleFinderCacher::generate_fair_scramble::<ScrambleFinder>(scramble_options)
}

pub fn free_memory_for_all_scramble_finders() -> usize {
    ScrambleFinderCacher::free_memory_for_all_scramble_finders()
}

#[derive(Default)]
struct ScrambleFinderCacher {
    erased_sync_set: ErasedSyncSet,
}

static SCRAMBLE_FINDER_CACHER_SINGLETON: LazyLock<Mutex<ScrambleFinderCacher>> =
    LazyLock::<Mutex<ScrambleFinderCacher>>::new(Default::default);

impl ScrambleFinderCacher {
    pub fn map<
        ScrambleFinder: SolvingBasedScrambleFinder + 'static + Sync + Send,
        ReturnValue,
        F: Fn(&mut ScrambleFinder) -> ReturnValue,
    >(
        f: F,
    ) -> ReturnValue {
        // TODO: figure out how to share a concrete implementation instead of template code.
        // This is trickier than it sounds: https://stackoverflow.com/questions/40095383/how-to-return-a-reference-to-a-sub-value-of-a-value-that-is-under-a-mutex/40103840#40103840
        // There are some crates to do this, but not an obvious choice.
        let mut mutex_guard = SCRAMBLE_FINDER_CACHER_SINGLETON.lock().unwrap();

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

    pub fn generate_fair_scramble<
        ScrambleFinder: SolvingBasedScrambleFinder + 'static + Sync + Send,
    >(
        scramble_options: &ScrambleFinder::ScrambleOptions,
    ) -> Alg {
        ScrambleFinderCacher::map(|scramble_finder: &mut ScrambleFinder| {
            scramble_finder.generate_fair_scramble(scramble_options)
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
    pub fn free_memory_for_all_scramble_finders() -> usize {
        let mut mutex_guard = SCRAMBLE_FINDER_CACHER_SINGLETON.lock().unwrap();
        let num_freed = mutex_guard.erased_sync_set.len();
        mutex_guard.erased_sync_set.clear();
        // The number of freed scramble finders is not super useful in itself, but it can be a useful sense check that scramble finders were actually allocated and freed.
        num_freed
    }
}
