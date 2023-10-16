use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::_internal::{PackedKPattern, PackedKPuzzleOrbitInfo};

pub(crate) enum OrbitPermutationConstraint {
    AnyPermutation,
    SingleOrbitEvenParity,
    IdentityPermutation,
}
pub(crate) enum OrbitOrientationConstraint {
    AnySum,
    OrientationsMustSumToZero,
}

// Selects a random permutation (ignoring parity).
// Applies a random orientation to each piece (ensuring the total is 0).
pub(crate) fn randomize_orbit_naive(
    pattern: &mut PackedKPattern,
    orbit_info: &PackedKPuzzleOrbitInfo,
    permutation_constraints: OrbitPermutationConstraint,
    orientation_constraints: OrbitOrientationConstraint,
) {
    let mut rng = thread_rng();
    let mut piece_order: Vec<u8> = (0..(orbit_info.num_pieces as u8)).collect();
    match permutation_constraints {
        OrbitPermutationConstraint::AnyPermutation => {
            piece_order.shuffle(&mut rng);
        }
        OrbitPermutationConstraint::SingleOrbitEvenParity => {
            piece_order.shuffle(&mut rng);
            make_parity_even(&mut piece_order);
        }
        OrbitPermutationConstraint::IdentityPermutation => {}
    }

    let mut total_orientation = 0;
    for (i, p) in piece_order.into_iter().enumerate() {
        pattern
            .packed_orbit_data
            .set_packed_piece_or_permutation(orbit_info, i, p);
        let orientation = match orientation_constraints {
            OrbitOrientationConstraint::AnySum => {
                let random_orientation = rng.gen_range(0..orbit_info.num_orientations);
                total_orientation = add_u8_mod(
                    total_orientation,
                    random_orientation,
                    orbit_info.num_orientations,
                );
                random_orientation
            }
            OrbitOrientationConstraint::OrientationsMustSumToZero => {
                subtract_u8_mod(0, total_orientation, orbit_info.num_orientations)
            }
        };

        pattern
            .packed_orbit_data
            .set_packed_orientation(orbit_info, i, orientation);
    }
}

// Adds without overflow.
fn add_u8_mod(v1: u8, v2: u8, modulus: u8) -> u8 {
    ((v1 as u32) + (v2 as u32)).rem_euclid(modulus as u32) as u8
}

fn subtract_u8_mod(v1: u8, v2: u8, modulus: u8) -> u8 {
    ((v1 as i32) - (v2 as i32)).rem_euclid(modulus as i32) as u8
}

fn make_parity_even(permutation: &mut [u8]) {
    let parity = basic_parity(permutation);
    if parity == BasicParity::Odd {
        // Since odd parity is only possible with more than 1 element in the permutation, we can safely swap the first two elements.
        permutation.swap(0, 1);
    };
}

#[derive(PartialEq)]
enum BasicParity {
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

fn basic_parity(permutation: &[u8]) -> BasicParity {
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
