use instant::Instant;
use twsearch::scramble::{random_scramble_for_event, Event};

pub fn main() {
    for event in [
        Event::SkewbSpeedsolving,
        //<<< Event::Cube3x3x3Speedsolving,
        //<<< Event::Cube3x3x3Speedsolving,
        //<<< Event::Cube3x3x3Speedsolving,
        //<<< Event::Cube3x3x3Speedsolving,
        //<<< Event::Cube3x3x3Speedsolving,
        //<<< Event::Cube2x2x2Speedsolving,
        //<<< Event::Cube5x5x5Speedsolving,
        //<<< Event::Cube6x6x6Speedsolving,
        //<<< Event::Cube7x7x7Speedsolving,
        //<<< Event::Cube3x3x3Blindfolded,
        //<<< Event::Cube3x3x3FewestMoves,
        //<<< Event::Cube3x3x3OneHanded,
        //<<< Event::MegaminxSpeedsolving,
        //<<< Event::PyraminxSpeedsolving,
        //<<< Event::Cube5x5x5Blindfolded,
        //<<< Event::Cube3x3x3MultiBlind,
        //<<< Event::ClockSpeedsolving,
        //<<< Event::Cube3x3x3FewestMoves,
        //<<< Event::Cube3x3x3FewestMoves,
        //<<< Event::Cube3x3x3FewestMoves,
        //<<< Event::Cube3x3x3FewestMoves,
        //<<< Event::Cube3x3x3FewestMoves,
        //<<< Event::Cube3x3x3FewestMoves,
        //<<< Event::Cube3x3x3FewestMoves,
        //<<< Event::Cube3x3x3FewestMoves,
        //<<< Event::Cube3x3x3FewestMoves,
        //<<< Event::Cube3x3x3FewestMoves,
        //<<< Event::Cube3x3x3FewestMoves,
        //<<< Event::Cube3x3x3FewestMoves,
        //<<< Event::Cube3x3x3FewestMoves,
        //<<< Event::Cube3x3x3FewestMoves,
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
