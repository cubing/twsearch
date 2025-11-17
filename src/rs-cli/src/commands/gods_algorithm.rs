use twips::{
    _internal::{cli::args::GodsAlgorithmArgs, errors::CommandError},
    experimental_lib_api::{gods_algorithm, KPuzzleSource},
};

pub fn cli_gods_algorithm(gods_algorithm_args: GodsAlgorithmArgs) -> Result<(), CommandError> {
    gods_algorithm(
        &KPuzzleSource::from_clap_args(&gods_algorithm_args.def_args).kpuzzle()?,
        gods_algorithm_args.optional,
    )?;
    Ok(())
}
