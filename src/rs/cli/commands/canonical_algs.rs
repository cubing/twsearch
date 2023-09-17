use std::collections::HashMap;

use cubing::{
    alg::{Move, QuantumMove},
    kpuzzle::{KPuzzle, KPuzzleDefinition},
};

use twsearch::{
    CommandError, PackedKPuzzle, PackedKTransformationBuffer, SearchError,
    _internal::cli::CanonicalAlgsArgs,
};

use crate::{
    commands::canonical_fsm::{self, AllMoveMultiples, CanonicalFSM, MoveInfo},
    io::read_to_json,
};

pub fn canonical_algs(args: &CanonicalAlgsArgs) -> Result<(), CommandError> {
    println!("{:?}", args);
    let def: KPuzzleDefinition = read_to_json(&args.input_args.def_file)?;
    let kpuzzle = KPuzzle::try_new(def).unwrap();
    let packed_kpuzzle = PackedKPuzzle::try_from(&kpuzzle).unwrap();

    let identity_transformation =
        packed_kpuzzle
            .identity_transformation()
            .map_err(|e| SearchError {
                description: e.to_string(), // TODO
            })?;

    // let input_moves = args
    //     .moves_args
    //     .moves_parsed()
    //     .unwrap_or_else(|| kpuzzle.definition().moves.keys().cloned().collect());

    // let mut move_gcds = HashMapWithKeyOrdering::<QuantumMove, Move>::default();
    // for input_move in input_moves {
    //     move_gcds.modify_or_set(
    //         &input_move.quantum,
    //         |existing_move: &Move| Move {
    //             quantum: existing_move.quantum.clone(),
    //             amount: abs_gcd(existing_move.amount, input_move.amount),
    //         },
    //         || input_move.clone(),
    //     )
    // }
    // dbg!(move_gcds.values());

    let mut seen_quantum_moves = HashMap::<QuantumMove, Move>::new();

    // TODO: actually calculate GCDs
    let mut all_multiples = AllMoveMultiples {
        multiples: Vec::new(),
    };
    let move_gcds = args
        .moves_args
        .moves_parsed()
        .unwrap_or_else(|| kpuzzle.definition().moves.keys().cloned().collect()); // TODO: `Iterator` instead of `Vec`.
    for r#move in move_gcds {
        if let Some(existing) = seen_quantum_moves.get(&r#move.quantum) {
            // TODO: deduplicate by quantum move.
            println!(
                "Warning: two moves with the same quantum move specified ({}, {}). This is usually redundant.",
                existing, r#move
            );
        } else {
            seen_quantum_moves.insert(r#move.quantum.as_ref().clone(), r#move.clone());
        }

        // let f = |move_lcm| -> Result<Vec<MoveInfo>, CommandError> {
        let mut multiples = vec![]; // TODO: use order to set capacity.
        let move_transformation =
            packed_kpuzzle
                .transformation_from_move(&r#move)
                .map_err(|e| SearchError {
                    description: e.to_string(), // TODO
                })?;
        let mut move_multiple_transformation =
            PackedKTransformationBuffer::from(move_transformation.clone());
        let mut amount: i32 = r#move.amount;
        while move_multiple_transformation.current != identity_transformation {
            let mut move_multiple = r#move.clone();
            move_multiple.amount = amount;
            multiples.push(MoveInfo {
                r#move: move_multiple,
                metric_turns: 1, // TODO
                transformation: move_multiple_transformation.current.clone(),
                inverse_transformation: move_multiple_transformation.current.invert(),
            });

            amount += r#move.amount;
            move_multiple_transformation.apply_transformation(&move_transformation);
        }
        all_multiples.multiples.push(multiples);
        // };
    }

    let canonical_fsm = CanonicalFSM::try_new(all_multiples).expect("Expected to work!");
    dbg!(canonical_fsm);

    // for a in all_multiples {
    //     for b in a {
    //         println!("{}", b.r#move);
    //     }
    // }
    // println!("{:?}", all_multiples);

    // let all_multiples = move_lcms.into_iter();
    // let sdfdsf: dyn Iterator<Item = Result<Vec<MoveInfo, CommandError>>> = all_multiples.map(f);
    // let sdkjfslkdf: Vec<Vec<MoveInfo>> = sdfdsf.collect()?;

    // println!("{:?}", packed_kpuzzle.default_pattern());
    // // println!("{:?}", packed_kpuzzle.default_pattern().unpack().kpattern_data);

    // let t1 = packed_kpuzzle
    //     .transformation_from_move(&"R".try_into().unwrap())
    //     .unwrap();
    // let t2 = packed_kpuzzle
    //     .transformation_from_move(&"L".try_into().unwrap())
    //     .unwrap();
    // let t3 = packed_kpuzzle
    //     .transformation_from_move(&"U".try_into().unwrap())
    //     .unwrap();

    // println!("{}", do_transformations_commute(&t1, &t2));
    // println!("{}", do_transformations_commute(&t2, &t3));
    // println!("{}", do_transformations_commute(&t1, &t3));

    Ok(())
}
