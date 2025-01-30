#[derive(PartialEq, Eq)]
pub(crate) enum WedgeType {
    CornerLower,
    CornerUpper,
    Edge,
}

pub(crate) const NUM_WEDGES: u8 = 24;

pub(crate) const FIRST_WEDGE_INDEX_U: u8 = 0;
pub(crate) const FIRST_WEDGE_INDEX_D: u8 = 12;

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
