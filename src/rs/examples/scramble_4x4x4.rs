use instant::Instant;
use twsearch::scramble::{random_scramble_for_event, Event};

pub fn main() {
    let start_time = Instant::now();
    let scramble = random_scramble_for_event(Event::Cube4x4x4Speedsolving).unwrap();
    println!(
        "{} // scramble found in {:?} ({} nodes)",
        scramble,
        Instant::now() - start_time,
        scramble.nodes.len()
    );
}
