# `twsearch`

A pure Rust implementation of [`twsearch`](https://github.com/cubing/twsearch). For now, most of the code is experimental and placed in `twsearch::_internal`.

## Example usage

```rust
use twsearch::scramble::{random_scramble_for_event, Event::Cube2x2x2};

pub fn main() {
    let scramble = random_scramble_for_event(Cube2x2x2).unwrap();
    println!("{}", scramble);
}
```
