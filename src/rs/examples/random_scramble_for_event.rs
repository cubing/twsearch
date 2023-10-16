use instant::Instant;
use twsearch::scramble::random_scramble_for_event;

pub fn main() {
    let start_time = Instant::now();
    let scramble = random_scramble_for_event(twsearch::scramble::Event::Cube2x2x2).unwrap();
    println!("{} // Found in {:?}", scramble, Instant::now() - start_time);
}
