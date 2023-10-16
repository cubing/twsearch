use instant::Instant;
use twsearch::scramble::{random_scramble_for_event, Event::Cube2x2x2};

pub fn main() {
    let start_time = Instant::now();
    let scramble = random_scramble_for_event(Cube2x2x2).unwrap();
    println!("{} // Found in {:?}", scramble, Instant::now() - start_time);
}
