use cubing::kpuzzle::{KPuzzle, KPuzzleDefinition};
use twsearch::{
    CommandError, PackedKPuzzle, PackedKTransformation, _internal::cli::CanonicalAlgsArgs,
};

use crate::io::read_to_json;

fn do_transformations_commute(t1: &PackedKTransformation, t2: &PackedKTransformation) -> bool {
    t1.apply_transformation(t2) == t2.apply_transformation(t1)
}

pub fn canonical_algs(args: &CanonicalAlgsArgs) -> Result<(), CommandError> {
    println!("{:?}", args);
    let def: KPuzzleDefinition = read_to_json(&args.input_args.def_file)?;
    let kpuzzle = KPuzzle::try_new(def).unwrap();
    let packed_kpuzzle = PackedKPuzzle::try_from(kpuzzle).unwrap();

    println!("{:?}", packed_kpuzzle.default_pattern());
    // println!("{:?}", packed_kpuzzle.default_pattern().unpack().kpattern_data);

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
