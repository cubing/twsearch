use twsearch::scramble::{random_scramble_for_event, Event};

pub fn main() {
    let scramble = random_scramble_for_event(Event::Cube2x2x2Speedsolving).unwrap();
    println!("{}", scramble);
}
