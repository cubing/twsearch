// Run using: cargo run --package twips --release --example scramble_all_events

use instant::Instant;
use twips::scramble::{random_scramble_for_event, Event};

pub fn main() {
    for event in [
        Event::Cube3x3x3Speedsolving,
        Event::Cube2x2x2Speedsolving,
        // Event::Cube4x4x4Speedsolving,
        Event::Cube5x5x5Speedsolving,
        Event::Cube6x6x6Speedsolving,
        Event::Cube7x7x7Speedsolving,
        Event::Cube3x3x3Blindfolded,
        Event::Cube3x3x3FewestMoves,
        Event::Cube3x3x3OneHanded,
        Event::ClockSpeedsolving,
        Event::MegaminxSpeedsolving,
        Event::PyraminxSpeedsolving,
        Event::SkewbSpeedsolving,
        Event::Square1Speedsolving,
        // Event::Cube4x4x4Blindfolded,
        Event::Cube5x5x5Blindfolded,
        Event::Cube3x3x3MultiBlind,
        //Event::FTOSpeedsolving
        //Event::MasterTetraminxSpeedsolving
        Event::KilominxSpeedsolving,
        //Event::RediCubeSpeedsolving
        Event::BabyFTOSpeedsolving,
    ] {
        let start_time = Instant::now();
        let scramble = random_scramble_for_event(event).unwrap();
        println!(
            "{} // {} scramble found in {:?} ({} nodes)",
            scramble,
            event,
            Instant::now() - start_time,
            scramble.nodes.len()
        );
    }
}
