use cubing::kpuzzle::{KPattern, KTransformation, OrientationWithMod};

// TODO: validation
pub(crate) fn kpattern_to_transformation(pattern: &KPattern) -> Option<KTransformation> {
    let mut transformation = pattern.kpuzzle().identity_transformation();
    for orbit_info in pattern.kpuzzle().orbit_info_iter() {
        for i in 0..orbit_info.num_pieces {
            transformation.set_permutation_idx(orbit_info, i, pattern.get_piece(orbit_info, i));
            let orientation_with_mod = pattern.get_orientation_with_mod(orbit_info, i);
            // TODO
            if orientation_with_mod.orientation_mod != 0
                && orientation_with_mod.orientation_mod != 1
            {
                return None;
            }
            transformation.set_orientation_delta(orbit_info, i, orientation_with_mod.orientation);
        }
    }
    Some(transformation)
}

// TODO: validation
/// Uses an `orientationMod` of `0` for every piece.
fn ktransformation_to_kpattern(transformation: &KTransformation) -> Option<KPattern> {
    let mut kpattern = transformation.kpuzzle().default_pattern();
    for orbit_info in transformation.kpuzzle().orbit_info_iter() {
        for i in 0..orbit_info.num_pieces {
            kpattern.set_piece(
                orbit_info,
                i,
                transformation.get_permutation_idx(orbit_info, i),
            );
            let orientation_delta = transformation.get_orientation_delta(orbit_info, i);
            kpattern.set_orientation_with_mod(
                orbit_info,
                i,
                &OrientationWithMod {
                    orientation: orientation_delta,
                    orientation_mod: 0,
                },
            );
        }
    }
    Some(kpattern)
}

pub(crate) fn invert_kpattern_as_transformation(pattern: &KPattern) -> Option<KPattern> {
    let transformation = kpattern_to_transformation(pattern)?;
    ktransformation_to_kpattern(&transformation.invert())
}
