use cubing::alg::Alg;

use super::{
    puzzles::{
        baby_fto::scramble_baby_fto,
        big_cubes::{scramble_5x5x5, scramble_5x5x5_bld, scramble_6x6x6, scramble_7x7x7},
        clock::scramble_clock,
        cube2x2x2::Cube2x2x2ScrambleFinder,
        cube3x3x3::{
            PrefixOrSuffixConstraints, TwoPhase3x3x3Scramble, TwoPhase3x3x3ScrambleOptions,
        },
        cube4x4x4::cube4x4x4_solver::Cube4x4x4Solver,
        megaminx::scramble_megaminx,
        pyraminx::PyraminxScrambleFinder,
        skewb::SkewbScrambleFinder,
        square1::scramble::scramble_square1,
    },
    solving_based_scramble_finder::{generate_fair_scramble, NoScrambleOptions},
    Event, PuzzleError,
};

pub fn random_scramble_for_event(event: Event) -> Result<Alg, PuzzleError> {
    let err = Err(PuzzleError {
        description: format!("Scrambles are not implement for this event yet: {}", event),
    });
    match event {
        Event::Cube3x3x3Speedsolving => Ok(generate_fair_scramble::<TwoPhase3x3x3Scramble>(
            &TwoPhase3x3x3ScrambleOptions {
                prefix_or_suffix_constraints: PrefixOrSuffixConstraints::None,
            },
        )),
        Event::Cube2x2x2Speedsolving => Ok(generate_fair_scramble::<Cube2x2x2ScrambleFinder>(
            &NoScrambleOptions {},
        )),
        Event::Cube4x4x4Speedsolving => Ok(generate_fair_scramble::<Cube4x4x4Solver>(
            &NoScrambleOptions {},
        )),
        Event::Cube5x5x5Speedsolving => Ok(scramble_5x5x5()),
        Event::Cube6x6x6Speedsolving => Ok(scramble_6x6x6()),
        Event::Cube7x7x7Speedsolving => Ok(scramble_7x7x7()),
        Event::Cube3x3x3Blindfolded => Ok(generate_fair_scramble::<TwoPhase3x3x3Scramble>(
            &TwoPhase3x3x3ScrambleOptions {
                prefix_or_suffix_constraints: PrefixOrSuffixConstraints::ForBLD,
            },
        )),
        Event::Cube3x3x3FewestMoves => Ok(generate_fair_scramble::<TwoPhase3x3x3Scramble>(
            &TwoPhase3x3x3ScrambleOptions {
                prefix_or_suffix_constraints: PrefixOrSuffixConstraints::ForFMC,
            },
        )),
        Event::Cube3x3x3OneHanded => Ok(generate_fair_scramble::<TwoPhase3x3x3Scramble>(
            &TwoPhase3x3x3ScrambleOptions {
                prefix_or_suffix_constraints: PrefixOrSuffixConstraints::None,
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
        Event::Square1Speedsolving => Ok(scramble_square1()),
        Event::Cube4x4x4Blindfolded => err,
        Event::Cube5x5x5Blindfolded => Ok(scramble_5x5x5_bld()),
        Event::Cube3x3x3MultiBlind => Ok(generate_fair_scramble::<TwoPhase3x3x3Scramble>(
            &TwoPhase3x3x3ScrambleOptions {
                prefix_or_suffix_constraints: PrefixOrSuffixConstraints::ForBLD,
            },
        )), // TODO: represent multiple returned scrambles without affecting ergonomics for other events.
        Event::FTOSpeedsolving => err,
        Event::MasterTetraminxSpeedsolving => err,
        Event::KilominxSpeedsolving => err,
        Event::RediCubeSpeedsolving => err,
        Event::BabyFTOSpeedsolving => Ok(scramble_baby_fto()),
    }
}
