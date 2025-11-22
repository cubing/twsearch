# `twips`

Twizzle Pattern Searcher — a program to find algs and scrambles for twisty puzzles.

See <https://github.com/cubing/twips> for information.

This is the library crate. You are welcome to experiment, but keep in mind that the API is very unstable. (Please file issues if you have a use cases!)

## Example usage

```rust
use twips::scramble::{random_scramble_for_event, Event};

pub fn main() {
    let scramble = random_scramble_for_event(Event::Cube2x2x2Speedsolving).unwrap();
    println!("{}", scramble);
}
```
