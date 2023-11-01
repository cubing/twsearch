use instant::Instant;
use twsearch::scramble::{random_scramble_for_event, Event};

pub fn main() {
    for event in [Event::Cube4x4x4Speedsolving, Event::Cube4x4x4Speedsolving] {
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
