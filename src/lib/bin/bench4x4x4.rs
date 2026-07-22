use std::{str::FromStr, time::Instant};

use twips::scramble::{derive_scramble_for_event, DerivationSeed, Event};

// Generated randomly with `DerivationSeed::from_thread_rng()`
const SEEDS: &[&str] = &[
    "67ffb1b6264c1fb98a30d22b4024d18099cc1678aaa0d995ec5b94e4723582ae",
    "67ff335173b8de81e84d6f7cb0c73a468555c080727a41c315d12053fc72f072",
    "67ff421e3036174cd3d05f29ca5eb776da3b6e1f09042776d6a264ed051e9c86",
    "67ff6da8e32be39c519639824f7268097a2cf838cadc4e2bbd375f41bb331422",
    "67ff48d8d567c3ef7671e2d325ebf9d1ef88f1e0b5dccd2449ac8e97d6755383",
    "67ff4ac6407c3ee6343e4d346f7cf831087590bdce7ef4e5af7cbdbc5ecf1497",
];

fn main() {
    let now = Instant::now();

    for seed in SEEDS {
        let derivation_seed = DerivationSeed::from_str(seed).unwrap();

        let now = Instant::now();

        let scramble =
            derive_scramble_for_event(Event::Cube4x4x4Speedsolving, derivation_seed).unwrap();
        println!("({:?}) {scramble}", now.elapsed());
    }

    println!("Total time: {:?}", now.elapsed());
}
