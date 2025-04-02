use std::ops::Add;

/// Swaps the first two entries (if needed) to make the permutation match `target_parity`.
pub(crate) fn set_parity(permutation: &mut [u8], target_parity: BasicParity) {
    let parity = basic_parity(permutation);
    if parity != target_parity {
        // Since odd parity is only possible with more than 1 element in the permutation, we can safely swap the first two elements.
        permutation.swap(0, 1);
    };
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub(crate) enum BasicParity {
    Even,
    Odd,
}

impl BasicParity {
    pub fn flip(&mut self) {
        let new_value = match self {
            BasicParity::Even => BasicParity::Odd,
            BasicParity::Odd => BasicParity::Even,
        };
        *self = new_value
    }

    /// [BasicParity::Even] → 0
    /// [BasicParity::Odd] → 1
    // TODO: Implement `From<BasicParity> for u8`? and `(Try)From<u8> for BasicParity`?
    pub fn to_0_or_1(&self) -> u8 {
        match self {
            BasicParity::Even => 0,
            BasicParity::Odd => 1,
        }
    }
}

/// This supports any input with non-repeating values, not necessarily
/// contiguous. (This allows calculating the parity of a subset of another
/// permutation without renumbering.)
pub(crate) fn basic_parity(permutation: &[u8]) -> BasicParity {
    let mut parity = BasicParity::Even;
    // TODO: we can save a tiny bit of speed by avoid iterating over the last element for `p1`.
    for (i, p2) in permutation.iter().enumerate().skip(1) {
        for p1 in &permutation[0..i] {
            if p1 > p2 {
                parity.flip();
            }
        }
    }
    parity
}

impl Add for BasicParity {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        if self == rhs {
            BasicParity::Even
        } else {
            BasicParity::Odd
        }
    }
}
