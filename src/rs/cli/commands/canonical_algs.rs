use cubing::kpuzzle::{KPuzzle, KPuzzleDefinition};

use twsearch::{
    CanonicalFSM, CommandError, PackedKPuzzle, SearchMoveCache, _internal::cli::CanonicalAlgsArgs,
};

use crate::io::read_to_json;

pub fn canonical_algs(args: &CanonicalAlgsArgs) -> Result<(), CommandError> {
    let def: KPuzzleDefinition = read_to_json(&args.input_args.def_file)?;
    let kpuzzle = KPuzzle::try_new(def).unwrap();
    let packed_kpuzzle = PackedKPuzzle::try_from(&kpuzzle).unwrap();

    let move_seeds = args
        .moves_args
        .moves_parsed()
        .unwrap_or_else(|| kpuzzle.definition().moves.keys().cloned().collect()); // TODO: `Iterator` instead of `Vec`.

    let search_move_cache =
        SearchMoveCache::try_new(&packed_kpuzzle, &move_seeds, &args.metric_args.metric)?;

    let canonical_fsm = CanonicalFSM::try_new(search_move_cache).expect("Expected to work!");
    dbg!(canonical_fsm);

    Ok(())
}
