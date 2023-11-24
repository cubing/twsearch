use cubing::kpuzzle::{KPuzzle, KPuzzleOrbitInfo};

pub(crate) fn orbit_info<'a>(
    packed_kpuzzle: &'a KPuzzle,
    orbit_index: usize,
    expected_orbit_name: &'a str,
) -> &'a KPuzzleOrbitInfo {
    let orbit_info = &packed_kpuzzle.data.ordered_orbit_info[orbit_index];
    assert_eq!(orbit_info.name.0, expected_orbit_name);
    orbit_info
}
