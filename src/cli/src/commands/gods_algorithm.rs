use twips::{
    _internal::errors::TwipsError,
    experimental_lib_api::{gods_algorithm, GodsAlgorithmOptions, KPuzzleSource},
};

use crate::args::GodsAlgorithmArgs;

pub fn cli_gods_algorithm(gods_algorithm_args: GodsAlgorithmArgs) -> Result<(), TwipsError> {
    let kpuzzle_source: KPuzzleSource = gods_algorithm_args.def_args.into();
    let kpuzzle = kpuzzle_source.kpuzzle()?;
    let options = GodsAlgorithmOptions {
        start_pattern: gods_algorithm_args
            .optional
            .start_pattern_args
            .start_pattern_source()
            .kpattern(&kpuzzle)?,
        generators: gods_algorithm_args.optional.generator_args.generators(),
        metric: gods_algorithm_args.optional.metric_args.metric,
    };
    // gods_algorithm_args.into::KPuzzleS
    gods_algorithm(&kpuzzle, options)?;
    Ok(())
}
