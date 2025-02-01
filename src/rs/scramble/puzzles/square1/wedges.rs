use cubing::kpuzzle::KPattern;

#[derive(PartialEq, Eq)]
pub(crate) enum WedgeType {
    CornerLower,
    CornerUpper,
    Edge,
}

pub(crate) const NUM_WEDGES: u8 = 24;

pub(crate) const WEDGE_TYPE_LOOKUP: [WedgeType; NUM_WEDGES as usize] = [
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
];

const FIRST_WEDGE_INDEX_U: u8 = 0;
const FIRST_WEDGE_INDEX_D: u8 = 12;

// The move amount (for `U_SQ_` or `D_SQ_`) that needs to be applied to
// edges/corners to match the lookup table reference shape (which is `(1, 0)`
// away from solved).
#[derive(Debug, Clone)]
#[allow(non_snake_case)]
pub(crate) struct Square1Phase2Offsets {
    pub(crate) edges_amount_U: i32,
    pub(crate) edges_amount_D: i32,
    pub(crate) corners_amount_U: i32,
    pub(crate) corners_amount_D: i32,
}

fn get_phase2_shape_offset_single_side(pattern: &KPattern, wedge_index: u8) -> (i32, i32) {
    let orbit_info = &pattern.kpuzzle().data.ordered_orbit_info[0];
    assert_eq!(orbit_info.name.0, "WEDGES");

    match WEDGE_TYPE_LOOKUP[pattern.get_piece(orbit_info, wedge_index) as usize] {
        WedgeType::CornerLower => (-2, 1),
        WedgeType::CornerUpper => (-1, -1),
        WedgeType::Edge => (0, 0),
    }
}

pub(crate) fn get_phase2_shape_offsets(pattern: &KPattern) -> Square1Phase2Offsets {
    // Note that the tuples look like Square-1 move tuples, but are in fact not (they both correspond to U moves).
    #[allow(non_snake_case)]
    let (edges_amount_U, corners_amount_U) =
        get_phase2_shape_offset_single_side(pattern, FIRST_WEDGE_INDEX_U);
    #[allow(non_snake_case)]
    let (edges_amount_D, corners_amount_D) =
        get_phase2_shape_offset_single_side(pattern, FIRST_WEDGE_INDEX_D);
    Square1Phase2Offsets {
        edges_amount_U,
        edges_amount_D,
        corners_amount_U,
        corners_amount_D,
    }
}
