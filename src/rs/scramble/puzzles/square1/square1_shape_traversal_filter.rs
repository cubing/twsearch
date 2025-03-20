use crate::{
    _internal::search::filter::filtering_decision::FilteringDecision,
    scramble::puzzles::square1::wedges::{WedgeType, WEDGE_TYPE_LOOKUP},
};

const SLOTS_THAT_ARE_AFTER_SLICES: [u8; 4] = [0, 6, 12, 18];

pub fn shape_traversal_filter_pattern(pattern: &cubing::kpuzzle::KPattern) -> FilteringDecision {
    let orbit_info = &pattern.kpuzzle().data.ordered_orbit_info[0];
    assert_eq!(orbit_info.name.0, "WEDGES");

    for slot in SLOTS_THAT_ARE_AFTER_SLICES {
        let value = pattern.get_piece(orbit_info, slot);

        // Note: the `WEDGE_TYPE_LOOKUP` lookup is not necessary for phase 1, but it is needed for a single-phase search.
        if WEDGE_TYPE_LOOKUP[value as usize] == WedgeType::CornerUpper {
            return FilteringDecision::Reject;
        }
    }

    FilteringDecision::Accept
}
