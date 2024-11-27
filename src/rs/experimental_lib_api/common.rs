use derive_more::From;

use std::{path::PathBuf, process::exit};

use crate::_internal::{
    cli::{
        args::{DefAndOptionalScrambleArgs, DefOnlyArgs},
        io::read_to_json,
    },
    errors::{ArgumentError, CommandError},
};
use cubing::{
    alg::Alg,
    kpuzzle::{KPattern, KPatternData, KPuzzle, KPuzzleDefinition},
};

// TODO: can we afford to make these fields into references?
#[derive(Debug, From)]
pub enum KPuzzleDefinitionSource {
    FilePath(PathBuf),
    KPuzzleDefinition(KPuzzleDefinition),
}

impl KPuzzleDefinitionSource {
    pub fn kpuzzle(self) -> Result<KPuzzle, ArgumentError /* TODO */> {
        let def = match self {
            KPuzzleDefinitionSource::FilePath(path_buf) => read_to_json(&path_buf)?,
            KPuzzleDefinitionSource::KPuzzleDefinition(kpuzzle_definition) => kpuzzle_definition,
        };

        KPuzzle::try_from(def).map_err(|e| ArgumentError {
            description: format!("Invalid definition: {}", e),
        })
    }
}

impl KPuzzleDefinitionSource {
    // TODO
    pub fn from_clap_args(def_args: &DefOnlyArgs) -> Self {
        Self::FilePath(def_args.def_file.clone())
    }
}

// TODO: can we afford to make these fields into references?
#[derive(Debug, From)]
pub enum PatternSource {
    DefaultFromDefinition,
    FilePath(PathBuf),
    Alg(Alg),
}

impl PatternSource {
    pub fn pattern(self, kpuzzle: &KPuzzle) -> Result<KPattern, CommandError /* TODO */> {
        Ok(match self {
            PatternSource::DefaultFromDefinition => kpuzzle.default_pattern(),
            PatternSource::FilePath(path_buf) => {
                let kpattern_data: KPatternData = read_to_json(&path_buf)?;
                match KPattern::try_from_data(kpuzzle, &kpattern_data) {
                    Ok(start_or_target_pattern) => start_or_target_pattern,
                    Err(e) => {
                        return Err(CommandError::ArgumentError(ArgumentError {
                            description: e.to_string(),
                        }))
                    }
                }
            }
            PatternSource::Alg(alg) => match kpuzzle.default_pattern().apply_alg(&alg) {
                Ok(pattern) => pattern,
                Err(err) => {
                    return Err(CommandError::ArgumentError(ArgumentError {
                        description: err.to_string(), // TODO
                    }));
                }
            },
        })
    }
}

impl PatternSource {
    // TODO
    pub fn search_pattern_from_clap_args(
        def_and_optional_scramble_args: &DefAndOptionalScrambleArgs,
    ) -> Result<Self, CommandError> {
        match (
            &def_and_optional_scramble_args.scramble_alg,
            &def_and_optional_scramble_args.scramble_file,
        ) {
            (None, None) => {
                println!("No scramble specified, exiting.");
                exit(0);
            }
            (None, Some(scramble_file)) => Ok(Self::FilePath(scramble_file.clone())),
            (Some(scramble_alg), None) => {
                let alg = match scramble_alg.parse::<Alg>() {
                    Ok(alg) => alg,
                    Err(e) => {
                        eprintln!("Could not parse alg: {:?}", e);
                        exit(1)
                    }
                };
                Ok(Self::Alg(alg))
            }
            (Some(_), Some(_)) => {
                eprintln!("Error: specified both a scramble alg and a scramble file, exiting.");
                exit(1);
            }
        }
    }
}
