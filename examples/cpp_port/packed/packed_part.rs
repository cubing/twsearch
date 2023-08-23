pub enum PackedPart {
    Piece,
    Permutation,
    Orientation,
}

impl PackedPart {
    pub fn offset_multiplier(&self) -> usize {
        match self {
            PackedPart::Piece => 0,
            PackedPart::Permutation => 0,
            PackedPart::Orientation => 1,
        }
    }
}
