use crate::_internal::{PackedKPuzzle, PackedKPuzzleOrbitInfo};

pub(crate) fn orbit_info<'a>(
    packed_kpuzzle: &'a PackedKPuzzle,
    orbit_index: usize,
    expected_orbit_name: &'a str,
) -> &'a PackedKPuzzleOrbitInfo {
    let orbit_info = &packed_kpuzzle.data.orbit_iteration_info[orbit_index];
    assert_eq!(orbit_info.name.0, expected_orbit_name);
    orbit_info
}
