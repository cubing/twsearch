use instant::Instant;
use twsearch::scramble::{
    random_scramble_for_event,
    Event::{Cube2x2x2, Pyraminx},
};

pub fn main() {
    let start_time = Instant::now();
    let scramble = random_scramble_for_event(Cube2x2x2).unwrap();
    println!(
        "{} // 2x2x2 scramble found in {:?}",
        scramble,
        Instant::now() - start_time
    );

    let start_time = Instant::now();
    let scramble = random_scramble_for_event(Pyraminx).unwrap();
    println!(
        "{} // Pyraminx scramble found in {:?}",
        scramble,
        Instant::now() - start_time
    );
}
