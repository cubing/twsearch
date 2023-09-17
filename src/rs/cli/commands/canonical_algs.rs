use cubing::kpuzzle::{KPuzzle, KPuzzleDefinition};

use twsearch::{CommandError, PackedKPuzzle, _internal::cli::CanonicalAlgsArgs};

use crate::{
    commands::{canonical_fsm::CanonicalFSM, move_multiples::moves_into_multiples_group},
    io::read_to_json,
};

pub fn canonical_algs(args: &CanonicalAlgsArgs) -> Result<(), CommandError> {
    let def: KPuzzleDefinition = read_to_json(&args.input_args.def_file)?;
    let kpuzzle = KPuzzle::try_new(def).unwrap();
    let packed_kpuzzle = PackedKPuzzle::try_from(&kpuzzle).unwrap();

    let move_gcds = args
        .moves_args
        .moves_parsed()
        .unwrap_or_else(|| kpuzzle.definition().moves.keys().cloned().collect()); // TODO: `Iterator` instead of `Vec`.

    let move_multiples_group = moves_into_multiples_group(&packed_kpuzzle, &move_gcds)?;

    let canonical_fsm = CanonicalFSM::try_new(move_multiples_group).expect("Expected to work!");
    dbg!(canonical_fsm);

    Ok(())
}
