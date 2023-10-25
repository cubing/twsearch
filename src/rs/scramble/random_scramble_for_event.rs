use lazy_static::lazy_static;
use std::sync::Mutex;

use cubing::alg::Alg;

use crate::_internal::PuzzleError;

use super::{
    cube2x2x2::scramble_2x2x2, cube3x3x3::Scramble3x3x3TwoPhase, megaminx::scramble_megaminx,
    pyraminx::scramble_pyraminx, Event,
};

pub fn random_scramble_for_event(event: Event) -> Result<Alg, PuzzleError> {
    let err = Err(PuzzleError {
        description: format!("Scrambles are not implement for this event yet: {}", event),
    });
    match event {
        Event::Cube3x3x3Speedsolving => {
            Ok(SCRAMBLE3X3X3_TWO_PHASE.lock().unwrap().scramble_3x3x3())
        }
        Event::Cube2x2x2Speedsolving => Ok(scramble_2x2x2()),
        Event::Cube4x4x4Speedsolving => err,
        Event::Cube5x5x5Speedsolving => err,
        Event::Cube6x6x6Speedsolving => err,
        Event::Cube7x7x7Speedsolving => err,
        Event::Cube3x3x3Blindfolded => err,
        Event::Cube3x3x3FewestMoves => err,
        Event::Cube3x3x3OneHanded => Ok(SCRAMBLE3X3X3_TWO_PHASE.lock().unwrap().scramble_3x3x3()),
        Event::ClockSpeedsolving => err,
        Event::MegaminxSpeedsolving => Ok(scramble_megaminx()),
        Event::PyraminxSpeedsolving => Ok(scramble_pyraminx()),
        Event::SkewbSpeedsolving => err,
        Event::Square1Speedsolving => err,
        Event::Cube4x4x4Blindfolded => err,
        Event::Cube5x5x5Blindfolded => err,
        Event::Cube3x3x3MultiBlind => err,
        Event::FTOSpeedsolving => err,
        Event::MasterTetraminxSpeedsolving => err,
        Event::KilominxSpeedsolving => err,
        Event::RediCubeSpeedsolving => err,
    }
}

// TODO: switch to `LazyLock` once that's stable: https://doc.rust-lang.org/nightly/std/cell/struct.LazyCell.html
lazy_static! {
    static ref SCRAMBLE3X3X3_TWO_PHASE: Mutex<Scramble3x3x3TwoPhase> =
        Mutex::new(Scramble3x3x3TwoPhase::default());
}
