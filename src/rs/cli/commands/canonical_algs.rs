use std::{fs::read_to_string, path::Path};

use cubing::kpuzzle::{KPuzzle, KPuzzleDefinition, KTransformation};
use serde::Deserialize;
use twsearch::{Packed, PackedKPuzzle, _internal::cli::CanonicalAlgsArgs};

fn read_to_json<T: for<'a> Deserialize<'a>>(input_file: &Path) -> Result<T, String> {
    format!("Rewriting: {:?}", input_file);
    let input_str = read_to_string(input_file).or(Err("Could not read input file."))?;
    let input_parsed: T =
        serde_json::from_str(&input_str).or(Err("Input file is not valid JSON."))?;
    Ok(input_parsed)
}

fn do_transformations_commute(t1: &Packed<KTransformation>, t2: &Packed<KTransformation>) -> bool {
    t1.apply_transformation(t2) == t2.apply_transformation(t1)
}

pub fn canonical_algs(args: &CanonicalAlgsArgs) -> Result<(), String> {
    println!("{:?}", args);
    let def: KPuzzleDefinition = read_to_json(&args.input_args.def_file).unwrap();
    let kpuzzle = KPuzzle::try_new(def).unwrap();
    let packed_kpuzzle = PackedKPuzzle::try_from(kpuzzle).unwrap();

    println!("{:?}", packed_kpuzzle.start_state());
    // println!("{:?}", packed_kpuzzle.start_state().unpack().state_data);

    let t1 = packed_kpuzzle
        .transformation_from_move(&"R".try_into().unwrap())
        .unwrap();
    let t2 = packed_kpuzzle
        .transformation_from_move(&"L".try_into().unwrap())
        .unwrap();
    let t3 = packed_kpuzzle
        .transformation_from_move(&"U".try_into().unwrap())
        .unwrap();

    println!("{}", do_transformations_commute(&t1, &t2));
    println!("{}", do_transformations_commute(&t2, &t3));
    println!("{}", do_transformations_commute(&t1, &t3));

    Ok(())
}
