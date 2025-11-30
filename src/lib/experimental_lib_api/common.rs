use derive_more::From;

use std::path::PathBuf;

use crate::_internal::{errors::ArgumentError, read_to_json::read_to_json};
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

// TODO: can we afford to make these fields into references?
#[derive(Debug, From, Default)]
pub enum PatternSource {
    #[default]
    DefaultFromDefinition,
    FilePath(PathBuf),
    AlgAppliedToDefaultPattern(Alg),
}

impl PatternSource {
    pub fn kpattern(
        &self,
        kpuzzle: &KPuzzle,
    ) -> Result<Option<KPattern>, ArgumentError /* TODO */> {
        Ok(match self {
            PatternSource::DefaultFromDefinition => None,
            PatternSource::FilePath(path_buf) => {
                let kpattern_data: KPatternData = read_to_json(path_buf)?;
                match KPattern::try_from_data(kpuzzle, &kpattern_data) {
                    Ok(start_or_target_pattern) => Some(start_or_target_pattern),
                    Err(e) => {
                        return Err(ArgumentError {
                            description: e.to_string(),
                        })
                    }
                }
            }
            PatternSource::AlgAppliedToDefaultPattern(alg) => {
                match kpuzzle.default_pattern().apply_alg(alg) {
                    Ok(pattern) => Some(pattern),
                    Err(err) => {
                        return Err(ArgumentError {
                            description: err.to_string(), // TODO
                        });
                    }
                }
            }
        })
    }
}
