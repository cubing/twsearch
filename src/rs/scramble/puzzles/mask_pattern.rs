use cubing::kpuzzle::KPattern;

pub(crate) fn mask(source_pattern: &KPattern, mask_pattern: &KPattern) -> KPattern {
    let mut masked_pattern = mask_pattern.clone();
    for orbit_info in source_pattern.kpuzzle().orbit_info_iter() {
        for i in 0..orbit_info.num_pieces {
            let old_piece = source_pattern.get_piece(orbit_info, i);
            let old_piece_mapped = mask_pattern.get_piece(orbit_info, old_piece);
            masked_pattern.set_piece(orbit_info, i, old_piece_mapped);
            let orientation_with_mod = source_pattern.get_orientation_with_mod(orbit_info, i);
            masked_pattern.set_orientation_with_mod(orbit_info, i, orientation_with_mod);
        }
    }
    masked_pattern
}
