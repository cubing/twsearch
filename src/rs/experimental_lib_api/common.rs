use derive_more::From;

use std::{path::PathBuf, process::exit};

use crate::_internal::{
    cli::{
        args::{DefOnlyArgs, ScrambleAndTargetPatternOptionalArgs},
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
pub enum KPuzzleSource {
    KPuzzle(KPuzzle),
    KPuzzleDefinition(KPuzzleDefinition),
    FilePath(PathBuf),
}

impl KPuzzleSource {
    pub fn kpuzzle(self) -> Result<KPuzzle, ArgumentError /* TODO */> {
        let def = match self {
            KPuzzleSource::KPuzzle(kpuzzle) => return Ok(kpuzzle),
            KPuzzleSource::KPuzzleDefinition(kpuzzle_definition) => kpuzzle_definition,
            KPuzzleSource::FilePath(path_buf) => read_to_json(&path_buf)?,
        };

        KPuzzle::try_from(def).map_err(|e| ArgumentError {
            description: format!("Invalid definition: {}", e),
        })
    }
}

impl KPuzzleSource {
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
    AlgAppliedToDefaultPattern(Alg),
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
            PatternSource::AlgAppliedToDefaultPattern(alg) => {
                match kpuzzle.default_pattern().apply_alg(&alg) {
                    Ok(pattern) => pattern,
                    Err(err) => {
                        return Err(CommandError::ArgumentError(ArgumentError {
                            description: err.to_string(), // TODO
                        }));
                    }
                }
            }
        })
    }
}

impl PatternSource {
    // TODO
    pub fn search_pattern_from_clap_args(
        scramble_and_target_pattern_optional_args: &ScrambleAndTargetPatternOptionalArgs,
    ) -> Result<Self, CommandError> {
        match (
            &scramble_and_target_pattern_optional_args.scramble_alg,
            &scramble_and_target_pattern_optional_args.scramble_file,
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
                Ok(Self::AlgAppliedToDefaultPattern(alg))
            }
            (Some(_), Some(_)) => {
                eprintln!("Error: specified both a scramble alg and a scramble file, exiting.");
                exit(1);
            }
        }
    }
}
