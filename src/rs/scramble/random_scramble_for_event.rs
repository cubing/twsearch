use cubing::{alg::Alg, kpuzzle::KPuzzle};

use crate::_internal::errors::{CommandError, SearchError};

use super::{
    get_kpuzzle::GetKPuzzle,
    puzzles::{
        baby_fto::scramble_baby_fto,
        big_cubes::{
            BigCubeScrambleFinderScrambleOptions, BigCubeScrambleFinderSuffixConstraints,
            Cube5x5x5ScrambleFinder, Cube6x6x6ScrambleFinder, Cube7x7x7ScrambleFinder,
        },
        clock_scramble_finder::ClockScrambleFinder,
        cube2x2x2_scramble_finder::Cube2x2x2ScrambleFinder,
        cube4x4x4::cube4x4x4_scramble_finder::Cube4x4x4ScrambleFinder,
        kilominx::kilominx_scramble_finder::KilominxScrambleFinder,
        megaminx::megaminx_scramble_finder::MegaminxScrambleFinder,
        pyraminx_scramble_finder::PyraminxScrambleFinder,
        skewb_scramble_finder::SkewbScrambleFinder,
        square1::square1_scramble_finder::Square1ScrambleFinder,
        two_phase_3x3x3_scramble_finder::{
            TwoPhase3x3x3PrefixOrSuffixConstraints, TwoPhase3x3x3ScrambleFinder,
            TwoPhase3x3x3ScrambleOptions,
        },
    },
    scramble_finder::{
        random_move_scramble_finder::{
            generate_filtered_random_move_scramble, random_move_scramble_finder_cacher_map,
            RandomMoveScrambleFinder,
        },
        solving_based_scramble_finder::{
            generate_fair_scramble, solving_based_scramble_finder_cacher_map, NoScrambleOptions,
            SolvingBasedScrambleFinder,
        },
    },
    Event, PuzzleError,
};

pub fn random_scramble_for_event(event: Event) -> Result<Alg, PuzzleError> {
    let err = Err(PuzzleError {
        description: format!("Scrambles are not implement for this event yet: {}", event),
    });
    match event {
        Event::Cube3x3x3Speedsolving => Ok(generate_fair_scramble::<TwoPhase3x3x3ScrambleFinder>(
            &TwoPhase3x3x3ScrambleOptions {
                prefix_or_suffix_constraints: TwoPhase3x3x3PrefixOrSuffixConstraints::None,
            },
        )),
        Event::Cube2x2x2Speedsolving => Ok(generate_fair_scramble::<Cube2x2x2ScrambleFinder>(
            &NoScrambleOptions {},
        )),
        Event::Cube4x4x4Speedsolving => Ok(generate_fair_scramble::<Cube4x4x4ScrambleFinder>(
            &NoScrambleOptions {},
        )),
        Event::Cube5x5x5Speedsolving => Ok(generate_filtered_random_move_scramble::<
            Cube5x5x5ScrambleFinder,
        >(&BigCubeScrambleFinderScrambleOptions {
            suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None,
        })),
        Event::Cube6x6x6Speedsolving => Ok(generate_filtered_random_move_scramble::<
            Cube6x6x6ScrambleFinder,
        >(&BigCubeScrambleFinderScrambleOptions {
            suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None,
        })),
        Event::Cube7x7x7Speedsolving => Ok(generate_filtered_random_move_scramble::<
            Cube7x7x7ScrambleFinder,
        >(&BigCubeScrambleFinderScrambleOptions {
            suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None,
        })),
        Event::Cube3x3x3Blindfolded => Ok(generate_fair_scramble::<TwoPhase3x3x3ScrambleFinder>(
            &TwoPhase3x3x3ScrambleOptions {
                prefix_or_suffix_constraints: TwoPhase3x3x3PrefixOrSuffixConstraints::ForBLD,
            },
        )),
        Event::Cube3x3x3FewestMoves => Ok(generate_fair_scramble::<TwoPhase3x3x3ScrambleFinder>(
            &TwoPhase3x3x3ScrambleOptions {
                prefix_or_suffix_constraints: TwoPhase3x3x3PrefixOrSuffixConstraints::ForFMC,
            },
        )),
        Event::Cube3x3x3OneHanded => Ok(generate_fair_scramble::<TwoPhase3x3x3ScrambleFinder>(
            &TwoPhase3x3x3ScrambleOptions {
                prefix_or_suffix_constraints: TwoPhase3x3x3PrefixOrSuffixConstraints::None,
            },
        )),
        Event::ClockSpeedsolving => Ok(
            generate_filtered_random_move_scramble::<ClockScrambleFinder>(&NoScrambleOptions {}),
        ),
        Event::MegaminxSpeedsolving => Ok(generate_filtered_random_move_scramble::<
            MegaminxScrambleFinder,
        >(&NoScrambleOptions {})),
        Event::PyraminxSpeedsolving => Ok(generate_fair_scramble::<PyraminxScrambleFinder>(
            &NoScrambleOptions {},
        )),
        Event::SkewbSpeedsolving => Ok(generate_fair_scramble::<SkewbScrambleFinder>(
            &Default::default(),
        )),
        Event::Square1Speedsolving => Ok(generate_fair_scramble::<Square1ScrambleFinder>(
            &NoScrambleOptions {},
        )),
        Event::Cube4x4x4Blindfolded => Ok(generate_fair_scramble::<Cube4x4x4ScrambleFinder>(
            &NoScrambleOptions {},
        )),
        Event::Cube5x5x5Blindfolded => Ok(generate_filtered_random_move_scramble::<
            Cube5x5x5ScrambleFinder,
        >(&BigCubeScrambleFinderScrambleOptions {
            suffix_constraints: BigCubeScrambleFinderSuffixConstraints::ForNoInspection,
        })),
        Event::Cube3x3x3MultiBlind => Ok(generate_fair_scramble::<TwoPhase3x3x3ScrambleFinder>(
            &TwoPhase3x3x3ScrambleOptions {
                prefix_or_suffix_constraints: TwoPhase3x3x3PrefixOrSuffixConstraints::ForBLD,
            },
        )), // TODO: represent multiple returned scrambles without affecting ergonomics for other events.
        Event::FTOSpeedsolving => err,
        Event::MasterTetraminxSpeedsolving => err,
        Event::KilominxSpeedsolving => Ok(generate_fair_scramble::<KilominxScrambleFinder>(
            &Default::default(),
        )),
        Event::RediCubeSpeedsolving => err,
        Event::BabyFTOSpeedsolving => Ok(scramble_baby_fto()),
    }
}

fn solving_based_filter_and_search<
    ScrambleFinder: SolvingBasedScrambleFinder<TPuzzle = KPuzzle> + GetKPuzzle + Send + Sync + 'static,
>(
    options: &ExperimentalFilterAndOrSearchOptions,
    collapse_using_collapse_inverted_alg: bool,
    scramble_options: &ScrambleFinder::ScrambleOptions,
) -> Result<Option<Alg>, CommandError> {
    let alg = match solving_based_scramble_finder_cacher_map(
        |scramble_finder: &mut ScrambleFinder| -> Result<Option<Alg>, SearchError> {
            let pattern = scramble_finder
                .get_kpuzzle()
                .default_pattern()
                .apply_alg(&options.scramble_setup_alg)
                .expect("Invalid alg for puzzle.");

            if options.apply_filtering {
                if scramble_finder
                    .filter_pattern(&pattern, scramble_options)
                    .is_reject()
                {
                    return Err(SearchError {
                        description: "Rejected due to filtering".to_owned(),
                    });
                }
                eprintln!("Filtering decision: accepted")
            };

            Ok(if options.perform_search {
                let alg = scramble_finder.solve_pattern(&pattern, scramble_options)?;
                if collapse_using_collapse_inverted_alg {
                    Some(scramble_finder.collapse_inverted_alg(alg))
                } else {
                    Some(alg)
                }
            } else {
                None
            })
        },
    ) {
        Ok(alg) => alg,
        Err(err) => return Err(CommandError::SearchError(err)),
    };
    Ok(alg)
}

fn solving_based_filter_and_search_with_no_scramble_options<
    ScrambleFinder: SolvingBasedScrambleFinder<TPuzzle = KPuzzle, ScrambleOptions = NoScrambleOptions>
        + GetKPuzzle
        + Send
        + Sync
        + 'static,
>(
    options: &ExperimentalFilterAndOrSearchOptions,
    collapse_using_collapse_inverted_alg: bool,
) -> Result<Option<Alg>, CommandError> {
    solving_based_filter_and_search::<ScrambleFinder>(
        options,
        collapse_using_collapse_inverted_alg,
        &NoScrambleOptions {},
    )
}

fn random_move_filter<
    ScrambleFinder: RandomMoveScrambleFinder<TPuzzle = KPuzzle> + GetKPuzzle + Send + Sync + 'static,
>(
    options: &ExperimentalFilterAndOrSearchOptions,
    scramble_options: &ScrambleFinder::ScrambleOptions,
) -> Result<Option<Alg>, CommandError> {
    random_move_scramble_finder_cacher_map(
        |scramble_finder: &mut ScrambleFinder| -> Result<(), CommandError> {
            let pattern = scramble_finder
                .get_kpuzzle()
                .default_pattern()
                .apply_alg(&options.scramble_setup_alg)
                .expect("Invalid alg for puzzle.");

            if options.apply_filtering {
                if scramble_finder
                    .filter_pattern(&pattern, scramble_options)
                    .is_reject()
                {
                    return Err(SearchError {
                        description: "Rejected due to filtering".to_owned(),
                    }
                    .into());
                }
                eprint!("Filtering decision: accepted")
            };
            if options.perform_search {
                return Err(CommandError::ArgumentError(
                    "Tried to initiate a solve for a `RandomMoveScrambleFinder`".into(),
                ));
            };
            Ok(())
        },
    )?;
    Ok(None)
}

fn random_move_filter_with_no_scramble_options<
    ScrambleFinder: RandomMoveScrambleFinder<TPuzzle = KPuzzle, ScrambleOptions = NoScrambleOptions>
        + GetKPuzzle
        + Send
        + Sync
        + 'static,
>(
    options: &ExperimentalFilterAndOrSearchOptions,
) -> Result<Option<Alg>, CommandError> {
    random_move_filter::<ScrambleFinder>(options, &NoScrambleOptions {})
}

// TODO: this is kind of gnarly, but it avoids some severe limitations with dynamic dispatch in Rust due to the associated type for `ScrambleFinder`.
pub struct ExperimentalFilterAndOrSearchOptions {
    // pattern: &<ScrambleFinder::TPuzzle as SemiGroupActionPuzzle>::Pattern,
    pub scramble_setup_alg: Alg,
    pub apply_filtering: bool,
    pub perform_search: bool,
}

pub fn experimental_scramble_finder_filter_and_or_search(
    event: Event,
    options: &ExperimentalFilterAndOrSearchOptions,
) -> Result<Option<Alg>, CommandError> {
    let err = Err(PuzzleError {
        description: format!(
            "Scramble finder testing is not implemented for this event yet: {}",
            event
        ),
    }
    .into());
    match event {
        Event::Cube3x3x3Speedsolving => {
            solving_based_filter_and_search::<TwoPhase3x3x3ScrambleFinder>(
                options,
                false,
                &TwoPhase3x3x3ScrambleOptions {
                    prefix_or_suffix_constraints: TwoPhase3x3x3PrefixOrSuffixConstraints::None,
                },
            )
        }
        Event::Cube3x3x3Blindfolded => {
            solving_based_filter_and_search::<TwoPhase3x3x3ScrambleFinder>(
                options,
                false,
                &TwoPhase3x3x3ScrambleOptions {
                    prefix_or_suffix_constraints: TwoPhase3x3x3PrefixOrSuffixConstraints::ForBLD,
                },
            )
        }
        Event::Cube2x2x2Speedsolving => solving_based_filter_and_search_with_no_scramble_options::<
            Cube2x2x2ScrambleFinder,
        >(options, false),
        Event::Cube4x4x4Speedsolving => solving_based_filter_and_search_with_no_scramble_options::<
            Cube4x4x4ScrambleFinder,
        >(options, false),
        Event::Cube5x5x5Speedsolving => random_move_filter::<Cube5x5x5ScrambleFinder>(
            options,
            &BigCubeScrambleFinderScrambleOptions {
                suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None,
            },
        ),
        Event::Cube6x6x6Speedsolving => random_move_filter::<Cube6x6x6ScrambleFinder>(
            options,
            &BigCubeScrambleFinderScrambleOptions {
                suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None,
            },
        ),
        Event::Cube7x7x7Speedsolving => random_move_filter::<Cube7x7x7ScrambleFinder>(
            options,
            &BigCubeScrambleFinderScrambleOptions {
                suffix_constraints: BigCubeScrambleFinderSuffixConstraints::None,
            },
        ),
        Event::ClockSpeedsolving => {
            random_move_filter_with_no_scramble_options::<ClockScrambleFinder>(options)
        }
        Event::MegaminxSpeedsolving => {
            random_move_filter_with_no_scramble_options::<MegaminxScrambleFinder>(options)
        }
        Event::PyraminxSpeedsolving => solving_based_filter_and_search_with_no_scramble_options::<
            PyraminxScrambleFinder,
        >(options, false),
        Event::Square1Speedsolving => solving_based_filter_and_search_with_no_scramble_options::<
            Square1ScrambleFinder,
        >(options, true),
        Event::Cube4x4x4Blindfolded => solving_based_filter_and_search_with_no_scramble_options::<
            Cube4x4x4ScrambleFinder,
        >(options, false),
        Event::Cube5x5x5Blindfolded => random_move_filter::<Cube5x5x5ScrambleFinder>(
            options,
            &BigCubeScrambleFinderScrambleOptions {
                suffix_constraints: BigCubeScrambleFinderSuffixConstraints::ForNoInspection,
            },
        ),
        Event::KilominxSpeedsolving => solving_based_filter_and_search_with_no_scramble_options::<
            KilominxScrambleFinder,
        >(options, false),
        _ => err,
    }
}
