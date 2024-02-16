use cubing::kpuzzle::{KPattern, KPuzzle, OrientationWithMod};

const ZERO_ORIENTATION: OrientationWithMod = OrientationWithMod {
    orientation: 0,
    orientation_mod: 0,
};

/// Constructs a pattern with fully distinguishable pieces.
pub fn super_pattern(kpuzzle: &KPuzzle) -> KPattern {
    // TODO: is this more efficient than constructing directly?
    let mut pattern = kpuzzle.default_pattern();
    for orbit_info in kpuzzle.orbit_info_iter() {
        for idx in 0..orbit_info.num_pieces {
            pattern.set_piece(orbit_info, idx, idx);
            pattern.set_orientation_with_mod(orbit_info, idx, &ZERO_ORIENTATION);
        }
    }
    pattern
}
