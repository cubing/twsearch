use cubing::alg::Move;

use crate::{CanonicalFSMState, PackedKPattern, PackedKPuzzle};

pub struct IDFSearch {
    pub packed_kpuzzle: PackedKPuzzle,
    pub target_pattern: PackedKPattern,
    pub move_list: Vec<Move>,
    pub scramble_pattern: PackedKPattern,
}

impl IDFSearch {
    pub fn print_fields(&self) {
        dbg!(&self.packed_kpuzzle);
        dbg!(&self.target_pattern);
        dbg!(&self.move_list);
        dbg!(&self.scramble_pattern);
    }

    // fn recurse(current_pattern: PackedKPattern, current_state: CanonicalFSMState) {}
}
