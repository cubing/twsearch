use cubing::kpuzzle::{KPattern, OrientationWithMod};
use rand::{seq::SliceRandom, thread_rng, Rng};

pub(crate) enum OrbitPermutationConstraint {
    EvenParity,
    OddParity,
    IdentityPermutation,
}

pub(crate) enum OrbitOrientationConstraint {
    IgnoreAllOrientations,
    SumToZero,
    // TODO: this is a hack for the baby FTO def.
    // - Split the pieces into two sets: those in the specified vec, and the other pieces.
    // - If a piece is in an index belonging to the same set, its orientation must be even.
    //   - Otherwise, its orientation must be odd.
    /// The given vec must be sorted.
    EvenOddHackSumToZero(Vec<u8>),
}

// Note: this refers to the piece that is at index 0 in the *solved* pattern (i.e. the piece with value `0` in the `permutation` array), which may not necessarily be at index 0 in the *randomized* pattern.
pub(crate) enum ConstraintForPiece0 {
    KeepSolved,
    IgnoredOrientation,
}

/// Example:
///
/// ```ignore
/// use crate::scramble::randomize::{ConstraintForFirstPiece, OrbitRandomizationConstraints};
///
/// OrbitRandomizationConstraints {
///     piece_0: Some(ConstraintForPiece0::KeepSolved),
///     ..Default::default()
/// }
/// ```
#[derive(Default)]
pub(crate) struct OrbitRandomizationConstraints {
    pub(crate) permutation: Option<OrbitPermutationConstraint>,
    pub(crate) orientation: Option<OrbitOrientationConstraint>,
    pub(crate) piece_0: Option<ConstraintForPiece0>,
    pub(crate) subset: Option<Vec<u8>>,
}

/// Selects a random permutation and applies a random orientation to each piece,
/// subject to the given constraints.
///
/// Returns the piece order of the (subset of) randomized pieces.
pub(crate) fn randomize_orbit(
    pattern: &mut KPattern,
    orbit_idx: usize,
    orbit_name: &str,
    constraints: OrbitRandomizationConstraints,
) -> Vec<u8> {
    // TODO: make it easier to reuse `OrbitInfo` references from a higher level.
    let orbit_info = &pattern.kpuzzle().clone().data.ordered_orbit_info[orbit_idx];
    assert_eq!(orbit_info.name.0, orbit_name);

    let mut rng = thread_rng();
    let piece_order_original = constraints
        .subset
        .unwrap_or_else(|| (0..orbit_info.num_pieces).collect());
    let mut piece_order_shuffled = piece_order_original.clone();
    let first_randomized_piece = piece_order_shuffled[0];
    let first_shuffled_piece_order_index =
        if matches!(constraints.piece_0, Some(ConstraintForPiece0::KeepSolved)) {
            1
        } else {
            0
        };
    let shuffling_slice = piece_order_shuffled
        .split_at_mut_checked(first_shuffled_piece_order_index)
        .unwrap()
        .1;
    match constraints.permutation {
        None => {
            shuffling_slice.shuffle(&mut rng);
        }
        Some(OrbitPermutationConstraint::EvenParity) => {
            shuffling_slice.shuffle(&mut rng);
            set_parity(shuffling_slice, BasicParity::Even);
        }
        Some(OrbitPermutationConstraint::OddParity) => {
            shuffling_slice.shuffle(&mut rng);
            set_parity(shuffling_slice, BasicParity::Odd);
        }
        Some(OrbitPermutationConstraint::IdentityPermutation) => {}
    }

    let mut total_orientation = 0;
    for (shuffled_i, p) in piece_order_shuffled.iter().enumerate() {
        let original_i = piece_order_original[shuffled_i];
        pattern.set_piece(orbit_info, original_i, *p);
        let is_last_shuffled_piece = shuffled_i == piece_order_original.len() - 1;
        let orientation_with_mod = match (
            &constraints.orientation,
            &constraints.piece_0,
            is_last_shuffled_piece,
            *p == first_randomized_piece,
        ) {
            (Some(OrbitOrientationConstraint::IgnoreAllOrientations), _, _, _) => {
                OrientationWithMod {
                    orientation: 0,
                    orientation_mod: 1,
                }
            }
            (Some(OrbitOrientationConstraint::SumToZero), _, true, _) => OrientationWithMod {
                orientation: subtract_u8_mod(0, total_orientation, orbit_info.num_orientations),
                orientation_mod: 0,
            },
            (_, Some(ConstraintForPiece0::KeepSolved), _, true) => OrientationWithMod {
                orientation: 0,
                orientation_mod: 0,
            },
            (_, Some(ConstraintForPiece0::IgnoredOrientation), _, true) => OrientationWithMod {
                orientation: 0,
                orientation_mod: 1,
            },
            // TODO: merge with next case?
            (
                Some(OrbitOrientationConstraint::EvenOddHackSumToZero(even_set)),
                _,
                is_last_shuffled_piece,
                _,
            ) => {
                let random_orientation = if is_last_shuffled_piece {
                    subtract_u8_mod(0, total_orientation, orbit_info.num_orientations)
                } else {
                    // TODO: what can we use instead of `.step_by(2)`?
                    let even_orientation_offset = if even_set.binary_search(&original_i).is_ok()
                        == even_set.binary_search(p).is_ok()
                    {
                        0
                    } else {
                        1
                    };
                    rng.gen_range(
                        0..(orbit_info.num_orientations + 1 - even_orientation_offset) / 2,
                    ) * 2
                        + even_orientation_offset
                };
                total_orientation = add_u8_mod(
                    total_orientation,
                    random_orientation,
                    orbit_info.num_orientations,
                );
                OrientationWithMod {
                    orientation: random_orientation,
                    orientation_mod: 0,
                }
            }
            (_, _, _, _) => {
                let random_orientation = rng.gen_range(0..orbit_info.num_orientations);
                total_orientation = add_u8_mod(
                    total_orientation,
                    random_orientation,
                    orbit_info.num_orientations,
                );
                OrientationWithMod {
                    orientation: random_orientation,
                    orientation_mod: 0,
                }
            }
        };

        pattern.set_orientation_with_mod(orbit_info, original_i, &orientation_with_mod);
    }
    piece_order_shuffled
}

// Adds without overflow.
fn add_u8_mod(v1: u8, v2: u8, modulus: u8) -> u8 {
    ((v1 as u32) + (v2 as u32)).rem_euclid(modulus as u32) as u8
}

fn subtract_u8_mod(v1: u8, v2: u8, modulus: u8) -> u8 {
    ((v1 as i32) - (v2 as i32)).rem_euclid(modulus as i32) as u8
}

fn set_parity(permutation: &mut [u8], target_parity: BasicParity) {
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
}

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
