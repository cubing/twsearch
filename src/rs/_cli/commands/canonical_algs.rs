use cubing::kpuzzle::{KPuzzle, KPuzzleDefinition};

use twsearch::_internal::{
    cli::options::CanonicalAlgsArgs, read_to_json, CanonicalFSM, CommandError, SearchGenerators,
};

pub fn canonical_algs(args: &CanonicalAlgsArgs) -> Result<(), CommandError> {
    let def: KPuzzleDefinition = read_to_json(&args.input_args.def_file)?;
    let kpuzzle = KPuzzle::try_new(def).unwrap();

    let search_generators = SearchGenerators::try_new(
        &kpuzzle,
        args.generator_args
            .parse()
            .enumerate_moves_for_kpuzzle(&kpuzzle),
        &args.metric_args.metric,
        false,
    )?;

    let canonical_fsm =
        CanonicalFSM::try_new(kpuzzle, search_generators).expect("Expected to work!");
    dbg!(canonical_fsm);

    Ok(())
}
