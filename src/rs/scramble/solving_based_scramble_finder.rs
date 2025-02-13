use std::{cell::OnceCell, marker::PhantomData, sync::OnceLock};

use cubing::alg::Alg;

use crate::_internal::puzzle_traits::puzzle_traits::SemiGroupActionPuzzle;

pub enum FilteringDecision {
    Accept,
    Reject,
}

pub struct NoScrambleOptions {}

pub trait SolvingBasedScrambleFinder<
    TPuzzle: SemiGroupActionPuzzle,
    ScrambleOptions = NoScrambleOptions,
>
{
    fn new() -> Self;
    fn generate_fair_unfiltered_random_pattern(&mut self) -> TPuzzle::Pattern;
    fn filter_pattern(
        &mut self,
        pattern: &TPuzzle::Pattern,
        scramble_options: &ScrambleOptions,
    ) -> FilteringDecision;
    fn solve_pattern(
        &mut self,
        pattern: &TPuzzle::Pattern,
        scramble_options: &ScrambleOptions,
    ) -> Alg;
    fn invert_alg_to_use_as_scramble(
        &mut self,
        alg: Alg,
        scramble_options: &ScrambleOptions,
    ) -> Alg;

    fn generate_fair_scramble(&mut self, scramble_options: &ScrambleOptions) -> Alg {
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

struct CachedScrambleFinder<
    TPuzzle: SemiGroupActionPuzzle,
    ScrambleFinder: SolvingBasedScrambleFinder<TPuzzle, ScrambleOptions>,
    ScrambleOptions = NoScrambleOptions,
> {
    cached_finder: OnceCell<ScrambleFinder>,
    phantom_data: PhantomData<(TPuzzle, ScrambleOptions)>,
}

impl<
        TPuzzle: SemiGroupActionPuzzle,
        ScrambleFinder: SolvingBasedScrambleFinder<TPuzzle, ScrambleOptions>,
        ScrambleOptions,
    > CachedScrambleFinder<TPuzzle, ScrambleFinder, ScrambleOptions>
{
    pub fn get_cached_finder(&mut self) -> &mut ScrambleFinder {
        // TODO: use `.get_mut_or_init(…)` once that's stable.
        self.cached_finder.get_or_init(ScrambleFinder::new);
        self.cached_finder.get_mut().unwrap()
    }

    pub fn clear_cached_finder(&mut self) {
        self.cached_finder.take();
    }

    pub fn generate_fair_scramble(&mut self, scramble_options: &ScrambleOptions) -> Alg {
        self.get_cached_finder()
            .generate_fair_scramble(scramble_options)
    }
}
