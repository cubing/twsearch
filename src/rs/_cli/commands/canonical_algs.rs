use cubing::kpuzzle::{KPuzzle, KPuzzleDefinition};

use twsearch::_internal::{
    cli::options::CanonicalAlgsArgs, read_to_json, CanonicalFSM, CommandError, PackedKPuzzle,
    SearchGenerators,
};

pub fn canonical_algs(args: &CanonicalAlgsArgs) -> Result<(), CommandError> {
    let def: KPuzzleDefinition = read_to_json(&args.input_args.def_file)?;
    let kpuzzle = KPuzzle::try_new(def).unwrap();
    let packed_kpuzzle = PackedKPuzzle::try_from(&kpuzzle).unwrap();

    let search_generators = SearchGenerators::try_new(
        &packed_kpuzzle,
        &args.generator_args.parse(),
        &args.metric_args.metric,
        false,
    )?;

    let canonical_fsm = CanonicalFSM::try_new(search_generators).expect("Expected to work!");
    dbg!(canonical_fsm);

    Ok(())
}
