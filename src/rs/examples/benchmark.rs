use std::time::Instant;

use cubing::{parse_alg, parse_move, puzzles::cube3x3x3_kpuzzle};

use cubing::kpuzzle::{
    KPattern, KPatternData, KPatternOrbitData, KPuzzle, KPuzzleDefinition, KPuzzleOrbitDefinition,
    KPuzzleOrbitName, KTransformationData, KTransformationOrbitData,
};
use std::collections::HashMap;
use std::sync::Arc;

use twsearch::PackedKPuzzle;

const PRINT_FINAL_STATE: bool = false;

// Run using: cargo run --release --example benchmark
fn main() {
    let num_moves = 10_000_000;
    println!("Testing custom puzzle…\n--------");
    test_custom_puzzle();
    println!("Running timing tests…\n--------");
    test_packed(num_moves);
    test_unpacked(num_moves / 10);
}

fn test_packed(num_moves: usize) {
    let packed_kpuzzle = PackedKPuzzle::try_from(cube3x3x3_kpuzzle()).unwrap();

    let m = |s: &str| {
        packed_kpuzzle
            .transformation_from_move(&parse_move!(s).unwrap())
            .unwrap()
    };

    let move_transformations = vec![
        m("U"),
        m("U2"),
        m("U'"),
        m("L"),
        m("L2"),
        m("L'"),
        m("F"),
        m("F2"),
        m("F'"),
        m("R"),
        m("R2"),
        m("R'"),
        m("B"),
        m("B2"),
        m("B'"),
        m("D"),
        m("D2"),
        m("D'"),
    ];

    let mut current = packed_kpuzzle.start_state();
    let mut other = packed_kpuzzle.start_state();
    let start = Instant::now();
    for i in (0..num_moves).step_by(2) {
        current.apply_transformation_into(&move_transformations[i % 18], &mut other);
        other.apply_transformation_into(&move_transformations[(i + 1) % 18], &mut current);
    }
    if PRINT_FINAL_STATE {
        println!("{:?}", current.byte_slice());
        println!("Hash: 0x{:x}", current.hash());
    }
    let duration = start.elapsed();
    println!(
        "Time elapsed for {} moves (packed) without hashing (minimal allocation): {:?} ({:.2}M moves/s)\n--------",
        num_moves,
        duration,
        (std::convert::TryInto::<f64>::try_into(num_moves as u32).unwrap()
            / duration.as_secs_f64()
            / std::convert::TryInto::<f64>::try_into(1_000_000).unwrap())
    );

    let mut pattern = packed_kpuzzle.start_state();
    let start = Instant::now();
    for i in 0..num_moves {
        pattern = pattern.apply_transformation(&move_transformations[i % 18]);
    }
    if PRINT_FINAL_STATE {
        println!("{:?}", pattern.byte_slice());
        println!("Hash: 0x{:x}", pattern.hash());
    }
    let duration = start.elapsed();
    println!(
        "Time elapsed for {} moves (packed) without hashing: {:?} ({:.2}M moves/s)\n--------",
        num_moves,
        duration,
        (std::convert::TryInto::<f64>::try_into(num_moves as u32).unwrap()
            / duration.as_secs_f64()
            / std::convert::TryInto::<f64>::try_into(1_000_000).unwrap())
    );

    let mut current = packed_kpuzzle.start_state();
    let mut other = packed_kpuzzle.start_state();
    let start = Instant::now();
    for i in (0..num_moves).step_by(2) {
        current.apply_transformation_into(&move_transformations[i % 18], &mut other);
        _ = current.hash();
        other.apply_transformation_into(&move_transformations[(i + 1) % 18], &mut current);
        _ = other.hash();
    }
    if PRINT_FINAL_STATE {
        println!("{:?}", current.byte_slice());
        println!("Hash: 0x{:x}", current.hash());
    }
    let duration = start.elapsed();
    println!(
        "Time elapsed for {} moves (packed) with hashing (minimal allocation): {:?} ({:.2}M moves/s)\n--------",
        num_moves,
        duration,
        (std::convert::TryInto::<f64>::try_into(num_moves as u32).unwrap()
            / duration.as_secs_f64()
            / std::convert::TryInto::<f64>::try_into(1_000_000).unwrap())
    );

    let mut pattern = packed_kpuzzle.start_state();
    let start = Instant::now();
    for i in 0..num_moves {
        pattern = pattern.apply_transformation(&move_transformations[i % 18]);
        // _ = pattern.hash()
    }
    if PRINT_FINAL_STATE {
        println!("{:?}", pattern.byte_slice());
        println!("Hash: 0x{:x}", pattern.hash());
    }
    let duration = start.elapsed();
    println!(
        "Time elapsed for {} moves (packed) with hashing: {:?} ({:.2}M moves/s)\n--------",
        num_moves,
        duration,
        (std::convert::TryInto::<f64>::try_into(num_moves as u32).unwrap()
            / duration.as_secs_f64()
            / std::convert::TryInto::<f64>::try_into(1_000_000).unwrap())
    );
}

fn test_unpacked(num_moves: usize) {
    let kpuzzle = cube3x3x3_kpuzzle();
    let m = |s: &str| {
        kpuzzle
            .transformation_from_alg(&parse_alg!(s).unwrap())
            .unwrap()
    };
    let move_transformations = vec![
        m("U"),
        m("U2"),
        m("U'"),
        m("L"),
        m("L2"),
        m("L'"),
        m("F"),
        m("F2"),
        m("F'"),
        m("R"),
        m("R2"),
        m("R'"),
        m("B"),
        m("B2"),
        m("B'"),
        m("D"),
        m("D2"),
        m("D'"),
    ];

    let mut pattern = kpuzzle.default_pattern();
    let start = Instant::now();
    for i in 0..num_moves {
        pattern = pattern.apply_transformation(&move_transformations[i % 18]);
    }
    // println!("{:?}", pattern.kpattern_data);
    // Only works for a million
    // assert_eq!(
    //     pattern,
    //     kpuzzle
    //         .start_state()
    //         .apply_alg(&parse_alg!("U2 F2 L2 U2 D2 F2 R2 F2 R'").unwrap())
    //         .unwrap()
    // );
    let duration = start.elapsed();
    println!(
        "Time elapsed for {} moves (unpacked) without hashing: {:?} ({:.2}M moves/s)\n--------",
        num_moves,
        duration,
        (std::convert::TryInto::<f64>::try_into(num_moves as u32).unwrap()
            / duration.as_secs_f64()
            / std::convert::TryInto::<f64>::try_into(1_000_000).unwrap())
        .round()
            / std::convert::TryInto::<f64>::try_into(10).unwrap()
    );
}

fn test_custom_puzzle() {
    let def = KPuzzleDefinition {
        name: "custom".to_owned(),
        orbits: vec![
            (KPuzzleOrbitDefinition {
                orbit_name: "PIECES".into(),
                num_pieces: 2,
                num_orientations: 12,
            }),
        ],
        default_pattern: KPatternData::from([(
            KPuzzleOrbitName("PIECES".to_owned()),
            KPatternOrbitData {
                pieces: vec![0, 1],
                orientation: vec![0, 0],
                orientation_mod: Some(vec![3, 4]),
            },
        )])
        .into(),
        moves: HashMap::from([
            (
                "SPIN".try_into().unwrap(),
                Arc::new(KTransformationData::from([(
                    KPuzzleOrbitName("PIECES".to_owned()),
                    KTransformationOrbitData {
                        permutation: vec![0, 1], // TODO: is this actually L'?
                        orientation_delta: vec![2, 5],
                    },
                )])),
            ),
            (
                "SWAP".try_into().unwrap(),
                Arc::new(KTransformationData::from([(
                    KPuzzleOrbitName("PIECES".to_owned()),
                    KTransformationOrbitData {
                        permutation: vec![1, 0], // TODO: is this actually R'?
                        orientation_delta: vec![0, 0],
                    },
                )])),
            ),
        ]),
        experimental_derived_moves: None,
    };
    let kpuzzle = KPuzzle::try_new(def).unwrap();
    let packed_kpuzzle = PackedKPuzzle::try_from(kpuzzle.clone()).unwrap();

    let spin = packed_kpuzzle
        .transformation_from_move(&"SPIN".try_into().unwrap())
        .unwrap();
    let swap = packed_kpuzzle
        .transformation_from_move(&"SWAP".try_into().unwrap())
        .unwrap();

    let pattern = packed_kpuzzle.start_state();
    // println!("{:?}", pattern.unpack().kpattern_data);

    let pattern = pattern.apply_transformation(&spin);
    // println!("{:?}", pattern.unpack().kpattern_data);

    let pattern = pattern.apply_transformation(&swap);
    // println!("{:?}", pattern.unpack().kpattern_data);

    let pattern = pattern.apply_transformation(&spin);
    // println!("{:?}", pattern.unpack().kpattern_data);

    let expected = KPattern {
        kpuzzle,
        kpattern_data: KPatternData::from([(
            KPuzzleOrbitName("PIECES".to_owned()),
            KPatternOrbitData {
                pieces: vec![1, 0],
                orientation: vec![3, 1],
                orientation_mod: Some(vec![4, 3]),
            },
        )])
        .into(),
    };
    // println!("{:?}", expected.kpattern_data);
    assert_eq!(pattern.unpack(), expected);
    println!("Custom puzzle test passes!\n--------");
}
