use cubing::kpuzzle::KPattern;

use crate::scramble::{
    parity::{basic_parity, BasicParity},
    puzzles::square1::wedges::{WedgeType, NUM_WEDGES, WEDGE_TYPE_LOOKUP},
};

pub(crate) fn bandaged_wedge_parity(pattern: &KPattern) -> BasicParity {
    let wedge_orbit_info = &pattern.kpuzzle().data.ordered_orbit_info[0];
    assert_eq!(wedge_orbit_info.name.0, "WEDGES");

    let mut bandaged_wedges = Vec::<u8>::default();
    for slot in 0..NUM_WEDGES {
        let value = unsafe {
            pattern
                .packed_orbit_data()
                .get_raw_piece_or_permutation_value(wedge_orbit_info, slot)
        };
        if WEDGE_TYPE_LOOKUP[value as usize] != WedgeType::CornerUpper {
            bandaged_wedges.push(value);
        }
    }
    basic_parity(&bandaged_wedges)
}
