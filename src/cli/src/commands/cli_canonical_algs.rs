use cubing::kpuzzle::{KPuzzle, KPuzzleDefinition};
use twips::_internal::{
    canonical_fsm::{
        canonical_fsm::CanonicalFSM,
        search_generators::{SearchGenerators, SearchGeneratorsConstructorOptions},
    },
    errors::TwipsError,
    read_to_json::read_to_json,
};

use crate::args::CanonicalAlgsArgs;

pub fn canonical_algs(args: &CanonicalAlgsArgs) -> Result<(), TwipsError> {
    let def: KPuzzleDefinition = read_to_json(&args.def_args.def_file)?;
    let kpuzzle = KPuzzle::try_new(def).unwrap();

    let search_generators = SearchGenerators::try_new(
        &kpuzzle,
        args.generator_args
            .generators()
            .enumerate_moves_for_kpuzzle(&kpuzzle),
        SearchGeneratorsConstructorOptions {
            metric: args.metric_args.metric,
            random_start: Some(false),
        },
    )?;

    let canonical_fsm = CanonicalFSM::try_new(kpuzzle, search_generators, Default::default())
        .expect("Could not construct FSM");
    dbg!(canonical_fsm);

    Ok(())
}
