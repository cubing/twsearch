use std::path::{Path, PathBuf};

use crate::_internal::{
    cli::io::read_to_json,
    errors::{ArgumentError, CommandError},
};
use cubing::kpuzzle::{KPattern, KPatternData, KPuzzle, KPuzzleDefinition};

// TODO: move the core of this to `cubing.rs`.
pub(crate) fn parse_def_file_and_start_or_target_pattern_file(
    def_file: &Path,
    start_or_target_pattern_file: &Option<PathBuf>,
) -> Result<(KPuzzle, Option<KPattern>), CommandError> {
    let def: Result<KPuzzleDefinition, ArgumentError> = read_to_json(def_file);
    let def = def?;
    let kpuzzle = KPuzzle::try_from(def).map_err(|e| ArgumentError {
        description: format!("Invalid definition: {}", e),
    })?;

    let start_or_target_pattern: Option<KPattern> = match start_or_target_pattern_file {
        Some(start_pattern_file) => {
            let kpattern_data: KPatternData = read_to_json(start_pattern_file)?;
            Some(match KPattern::try_from_data(&kpuzzle, &kpattern_data) {
                Ok(start_or_target_pattern) => start_or_target_pattern,
                Err(e) => {
                    return Err(CommandError::ArgumentError(ArgumentError {
                        description: e.to_string(),
                    }))
                }
            })
        }
        None => None,
    };

    Ok((kpuzzle, start_or_target_pattern))
}
