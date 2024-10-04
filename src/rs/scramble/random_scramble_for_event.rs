use cubing::alg::Alg;

use crate::_internal::PuzzleError;

use super::{
    puzzles::{
        big_cubes::{scramble_5x5x5, scramble_5x5x5_bld, scramble_6x6x6, scramble_7x7x7},
        clock::scramble_clock,
        cube2x2x2::scramble_2x2x2,
        cube3x3x3::{scramble_3x3x3, scramble_3x3x3_bld, scramble_3x3x3_fmc},
        megaminx::scramble_megaminx,
        pyraminx::scramble_pyraminx,
        skewb::scramble_skewb,
        square1::scramble_square1,
    },
    Event,
};

pub fn random_scramble_for_event(event: Event) -> Result<Alg, PuzzleError> {
    let err = Err(PuzzleError {
        description: format!("Scrambles are not implement for this event yet: {}", event),
    });
    match event {
        Event::Cube3x3x3Speedsolving => Ok(scramble_3x3x3()),
        Event::Cube2x2x2Speedsolving => Ok(scramble_2x2x2()),
        Event::Cube4x4x4Speedsolving => err,
        Event::Cube5x5x5Speedsolving => Ok(scramble_5x5x5()),
        Event::Cube6x6x6Speedsolving => Ok(scramble_6x6x6()),
        Event::Cube7x7x7Speedsolving => Ok(scramble_7x7x7()),
        Event::Cube3x3x3Blindfolded => Ok(scramble_3x3x3_bld()),
        Event::Cube3x3x3FewestMoves => Ok(scramble_3x3x3_fmc()),
        Event::Cube3x3x3OneHanded => Ok(scramble_3x3x3()),
        Event::ClockSpeedsolving => Ok(scramble_clock()),
        Event::MegaminxSpeedsolving => Ok(scramble_megaminx()),
        Event::PyraminxSpeedsolving => Ok(scramble_pyraminx()),
        Event::SkewbSpeedsolving => Ok(scramble_skewb()),
        Event::Square1Speedsolving => Ok(scramble_square1()),
        Event::Cube4x4x4Blindfolded => err,
        Event::Cube5x5x5Blindfolded => Ok(scramble_5x5x5_bld()),
        Event::Cube3x3x3MultiBlind => Ok(scramble_3x3x3_bld()), // TODO: represent multiple returned scrambles without affecting ergonomics for other events.
        Event::FTOSpeedsolving => err,
        Event::MasterTetraminxSpeedsolving => err,
        Event::KilominxSpeedsolving => err,
        Event::RediCubeSpeedsolving => err,
    }
}
