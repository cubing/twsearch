use cubing::kpuzzle::KPattern;

pub(crate) fn orbit_pieces_byte_slice<'a>(
    pattern: &'a KPattern,
    orbit_index: usize,
    orbit_name: &str,
) -> &'a [u8] {
    let orbit = &pattern.kpuzzle().data.ordered_orbit_info[orbit_index];
    debug_assert_eq!(orbit.name.0, orbit_name);

    let from = orbit.pieces_or_permutations_offset;
    let to = from + (orbit.num_pieces as usize);

    let full_byte_slice = unsafe { pattern.byte_slice() };
    &full_byte_slice[from..to]
}
