use cubing::kpuzzle::{KPattern, KTransformation};

// TODO: validation
pub(crate) fn kpattern_to_transformation(kpattern: &KPattern) -> Option<KTransformation> {
    let mut transformation = kpattern.kpuzzle().identity_transformation();
    for orbit_info in kpattern.kpuzzle().orbit_info_iter() {
        for i in 0..orbit_info.num_pieces {
            transformation.set_permutation_idx(orbit_info, i, kpattern.get_piece(orbit_info, i));
            let orientation_with_mod = kpattern.get_orientation_with_mod(orbit_info, i);
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
