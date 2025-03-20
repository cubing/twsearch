use cubing::{alg::Alg, puzzles::cube3x3x3_kpuzzle};

use super::{
    puzzles::{
        baby_fto::scramble_baby_fto,
        big_cubes::{scramble_5x5x5, scramble_5x5x5_bld, scramble_6x6x6, scramble_7x7x7},
        clock::scramble_clock,
        cube2x2x2_scramble_finder::Cube2x2x2ScrambleFinder,
        cube4x4x4::cube4x4x4_scramble_finder::Cube4x4x4ScrambleFinder,
        megaminx::scramble_megaminx,
        pyraminx_scramble_finder::PyraminxScrambleFinder,
        skewb_scramble_finder::SkewbScrambleFinder,
        square1::square1_scramble_finder::Square1ScrambleFinder,
        two_phase_3x3x3_scramble_finder::{
            TwoPhase3x3x3PrefixOrSuffixConstraints, TwoPhase3x3x3ScrambleAssociatedAffixes,
            TwoPhase3x3x3ScrambleAssociatedData, TwoPhase3x3x3ScrambleFinder,
            TwoPhase3x3x3ScrambleOptions,
        },
    },
    solving_based_scramble_finder::{
        generate_fair_scramble, scramble_finder_cacher_map, NoScrambleAssociatedData,
        NoScrambleOptions, SolvingBasedScrambleFinder,
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
        Event::Cube5x5x5Speedsolving => Ok(scramble_5x5x5()),
        Event::Cube6x6x6Speedsolving => Ok(scramble_6x6x6()),
        Event::Cube7x7x7Speedsolving => Ok(scramble_7x7x7()),
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
        Event::ClockSpeedsolving => Ok(scramble_clock()),
        Event::MegaminxSpeedsolving => Ok(scramble_megaminx()),
        Event::PyraminxSpeedsolving => Ok(generate_fair_scramble::<PyraminxScrambleFinder>(
            &NoScrambleOptions {},
        )),
        Event::SkewbSpeedsolving => Ok(generate_fair_scramble::<SkewbScrambleFinder>(
            &Default::default(),
        )),
        Event::Square1Speedsolving => Ok(generate_fair_scramble::<Square1ScrambleFinder>(
            &NoScrambleOptions {},
        )),
        Event::Cube4x4x4Blindfolded => err,
        Event::Cube5x5x5Blindfolded => Ok(scramble_5x5x5_bld()),
        Event::Cube3x3x3MultiBlind => Ok(generate_fair_scramble::<TwoPhase3x3x3ScrambleFinder>(
            &TwoPhase3x3x3ScrambleOptions {
                prefix_or_suffix_constraints: TwoPhase3x3x3PrefixOrSuffixConstraints::ForBLD,
            },
        )), // TODO: represent multiple returned scrambles without affecting ergonomics for other events.
        Event::FTOSpeedsolving => err,
        Event::MasterTetraminxSpeedsolving => err,
        Event::KilominxSpeedsolving => err,
        Event::RediCubeSpeedsolving => err,
        Event::BabyFTOSpeedsolving => Ok(scramble_baby_fto()),
    }
}

pub fn scramble_finder_solve(event: Event, scramble_setup_alg: &Alg) -> Result<Alg, PuzzleError> {
    let err = Err(PuzzleError {
        description: format!(
            "Random scramble testing is not implemented for this event yet: {}",
            event
        ),
    });
    match event {
        Event::Cube3x3x3Speedsolving => {
            let pattern = cube3x3x3_kpuzzle()
                .default_pattern()
                .apply_alg(scramble_setup_alg)
                .expect("Invalid alg for puzzle.");
            let test_scramble =     scramble_finder_cacher_map(|scramble_finder: &mut TwoPhase3x3x3ScrambleFinder| -> Result<Alg, crate::_internal::errors::SearchError> {scramble_finder.solve_pattern(&pattern, &TwoPhase3x3x3ScrambleAssociatedData{ affixes:TwoPhase3x3x3ScrambleAssociatedAffixes::None  }, &TwoPhase3x3x3ScrambleOptions{
                prefix_or_suffix_constraints: TwoPhase3x3x3PrefixOrSuffixConstraints::None,
            })}).expect("Could not test scramble.");
            Ok(test_scramble)
        }
        Event::Cube4x4x4Speedsolving => {
            let pattern = Cube4x4x4ScrambleFinder::get_kpuzzle()
                .default_pattern()
                .apply_alg(scramble_setup_alg)
                .expect("Invalid alg for puzzle.");
            let test_scramble =     scramble_finder_cacher_map(|scramble_finder: &mut Cube4x4x4ScrambleFinder| -> Result<Alg, crate::_internal::errors::SearchError> {scramble_finder.solve_pattern(&pattern, &NoScrambleAssociatedData{}, &NoScrambleOptions{})}).expect("Could not test scramble.");
            Ok(test_scramble)
        }
        Event::Square1Speedsolving => {
            let pattern = Square1ScrambleFinder::get_kpuzzle()
                .default_pattern()
                .apply_alg(scramble_setup_alg)
                .expect("Invalid alg for puzzle.");
            let test_scramble = scramble_finder_cacher_map(|scramble_finder: &mut Square1ScrambleFinder| -> Result<Alg, crate::_internal::errors::SearchError> {
                let alg = scramble_finder.solve_pattern(&pattern, &NoScrambleAssociatedData{}, &NoScrambleOptions{})?;
                Ok(scramble_finder.collapse_inverted_alg(  alg))
            
            }).expect("Could not test scramble.");
            Ok(test_scramble) 
        }
        _ => err,
    }
}
