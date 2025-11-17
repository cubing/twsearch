use cubing::kpuzzle::{KPuzzle, KPuzzleDefinition};
use twips::_internal::{
    canonical_fsm::{canonical_fsm::CanonicalFSM, search_generators::SearchGenerators},
    cli::{args::CanonicalAlgsArgs, io::read_to_json},
    errors::CommandError,
};

pub fn canonical_algs(args: &CanonicalAlgsArgs) -> Result<(), CommandError> {
    let def: KPuzzleDefinition = read_to_json(&args.def_args.def_file)?;
    let kpuzzle = KPuzzle::try_new(def).unwrap();

    let search_generators = SearchGenerators::try_new(
        &kpuzzle,
        args.generator_args
            .parse()
            .enumerate_moves_for_kpuzzle(&kpuzzle),
        &args.metric_args.metric,
        false,
    )?;

    let canonical_fsm = CanonicalFSM::try_new(kpuzzle, search_generators, Default::default())
        .expect("Could not construct FSM");
    dbg!(canonical_fsm);

    Ok(())
}
