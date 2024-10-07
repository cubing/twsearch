use cubing::kpuzzle::{KPattern, OrientationWithMod};
use rand::{seq::SliceRandom, thread_rng, Rng};

pub(crate) enum OrbitPermutationConstraint {
    AnyPermutation,
    SingleOrbitEvenParity,
    SingleOrbitOddParity,
    IdentityPermutation,
}

impl Default for OrbitPermutationConstraint {
    fn default() -> Self {
        Self::SingleOrbitOddParity
    }
}
pub(crate) enum OrbitOrientationConstraint {
    AnySum,
    OrientationsMustSumToZero,
}

// Note: this refers to the piece that is at index 0 in the *solved* pattern (i.e. the piece with value `0` in the `permutation` array), which may not necessarily be at index 0 in the *randomized* pattern.
pub(crate) enum PieceZeroConstraint {
    AnyPositionAndOrientation,
    KeepSolved,
    IgnoredOrientation,
}

// Selects a random permutation (ignoring parity).
// Applies a random orientation to each piece (ensuring the total is 0).
// Returns the piece order
pub(crate) fn randomize_orbit_naïve(
    pattern: &mut KPattern,
    orbit_idx: usize,
    orbit_name: &str,
    permutation_constraints: OrbitPermutationConstraint,
    orientation_constraints: OrbitOrientationConstraint,
    piece_zero_constraint: PieceZeroConstraint,
) -> Vec<u8> {
    // TODO: make it easier to reuse `OrbitInfo` references from a higher level.
    let orbit_info = &pattern.kpuzzle().clone().data.ordered_orbit_info[orbit_idx];
    assert_eq!(orbit_info.name.0, orbit_name);

    let mut rng = thread_rng();
    let first_randomized_piece = if matches!(piece_zero_constraint, PieceZeroConstraint::KeepSolved)
    {
        1
    } else {
        0
    };
    let mut piece_order: Vec<u8> = (first_randomized_piece..orbit_info.num_pieces).collect();
    match permutation_constraints {
        OrbitPermutationConstraint::AnyPermutation => {
            piece_order.shuffle(&mut rng);
        }
        OrbitPermutationConstraint::SingleOrbitEvenParity => {
            piece_order.shuffle(&mut rng);
            set_parity(&mut piece_order, BasicParity::Even);
        }
        OrbitPermutationConstraint::SingleOrbitOddParity => {
            piece_order.shuffle(&mut rng);
            set_parity(&mut piece_order, BasicParity::Odd);
        }
        OrbitPermutationConstraint::IdentityPermutation => {}
    }

    if matches!(piece_zero_constraint, PieceZeroConstraint::KeepSolved) {
        piece_order.insert(0, 0);
    }

    let mut total_orientation = 0;
    for (i, p) in piece_order.iter().enumerate() {
        let i = i as u8;
        pattern.set_piece(orbit_info, i, *p);
        let orientation_with_mod = match (
            &orientation_constraints,
            &piece_zero_constraint,
            i == orbit_info.num_pieces - 1,
            *p == 0,
        ) {
            (OrbitOrientationConstraint::OrientationsMustSumToZero, _, true, _) => {
                OrientationWithMod {
                    orientation: subtract_u8_mod(0, total_orientation, orbit_info.num_orientations),
                    orientation_mod: 0,
                }
            }
            (_, PieceZeroConstraint::KeepSolved, _, true) => OrientationWithMod {
                orientation: 0,
                orientation_mod: 0,
            },
            (_, PieceZeroConstraint::IgnoredOrientation, _, true) => OrientationWithMod {
                orientation: 0,
                orientation_mod: 1,
            },
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

        pattern.set_orientation_with_mod(orbit_info, i, &orientation_with_mod);
    }
    piece_order
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

#[derive(PartialEq, Eq, Debug)]
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
