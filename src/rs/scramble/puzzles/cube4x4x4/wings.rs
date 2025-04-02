pub(crate) const NUM_WINGS: u8 = 24;

// A primary position is any wing position that can be reached from piece 0 (UBl) using <U, L, R, D>.
// A primary piece is any piece that starts in a primary position, and a high position/piece is any that is not a primary position/piece.
// This lookup does the following:
//
// - Given a primary piece, it returns that same piece.
// - Given a high piece, it returns the primary piece that is part of the same dedge (double edge) pair in the solved state.
pub(crate) const WING_TO_PRIMARY_WING_IN_DEDGE: [u8; NUM_WINGS as usize] = [
    0, 1, 2, 3, 3, 11, 23, 17, 2, 9, 20, 11, 1, 19, 21, 9, 0, 17, 22, 19, 20, 21, 22, 23,
];

pub(crate) const WING_TO_PARTNER_WING: [u8; NUM_WINGS as usize] = [
    16, 12, 8, 4, 3, 11, 23, 17, 2, 15, 20, 5, 1, 19, 21, 9, 0, 7, 22, 13, 10, 14, 18, 6,
];

pub(crate) const POSITION_IS_PRIMARY: [bool; NUM_WINGS as usize] = [
    true, true, true, true, false, false, false, false, false, true, false, true, false, false,
    false, false, false, true, false, true, true, true, true, true,
];
