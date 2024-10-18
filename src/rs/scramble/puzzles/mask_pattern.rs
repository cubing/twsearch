use std::cmp::min;

use cubing::kpuzzle::{KPattern, OrientationWithMod};

use crate::scramble::PuzzleError;

pub(crate) fn mask(
    source_pattern: &KPattern,
    mask_pattern: &KPattern,
) -> Result<KPattern, PuzzleError> {
    let mut masked_pattern = mask_pattern.clone();
    for orbit_info in source_pattern.kpuzzle().orbit_info_iter() {
        for i in 0..orbit_info.num_pieces {
            let old_piece = source_pattern.get_piece(orbit_info, i);
            let old_piece_mapped = mask_pattern.get_piece(orbit_info, old_piece);
            masked_pattern.set_piece(orbit_info, i, old_piece_mapped);

            let source_orientation_with_mod =
                source_pattern.get_orientation_with_mod(orbit_info, i);
            let mask_orientation_with_mod = mask_pattern.get_orientation_with_mod(orbit_info, i);

            if mask_orientation_with_mod.orientation != 0 {
                return Err(PuzzleError {
                    description: "Masks cannot currently have piece orientation".to_owned(),
                });
            };

            let source_mod = source_orientation_with_mod.orientation_mod;
            let source_mod = if source_mod == 0 {
                orbit_info.num_orientations
            } else {
                source_mod
            };

            let mask_mod = mask_orientation_with_mod.orientation_mod;
            let mask_mod = if mask_mod == 0 {
                orbit_info.num_orientations
            } else {
                mask_mod
            };

            if source_mod % mask_mod != 0 && mask_mod % source_mod != 0 {
                return Err(PuzzleError {
                    description: "Incompatible orientation mod in mask".to_owned(),
                });
            };

            let masked_mod = min(source_mod, mask_mod);
            let orientation_with_mod = OrientationWithMod {
                orientation: source_orientation_with_mod.orientation % masked_mod,
                orientation_mod: if masked_mod == orbit_info.num_orientations {
                    0
                } else {
                    masked_mod
                },
            };

            masked_pattern.set_orientation_with_mod(orbit_info, i, &orientation_with_mod);
        }
    }
    Ok(masked_pattern)
}
