use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::_internal::{PackedKPattern, PackedKPuzzleOrbitInfo};

pub(crate) enum OrbitPermutationParityConstraints {
    IgnoreParity,
    #[allow(dead_code)] // TODO
    SingleOrbitEvenParity,
}
pub(crate) enum OrbitOrientationParityConstraints {
    #[allow(dead_code)] // TODO
    IgnoreParity,
    OrientationsMustSumToZero,
}

// Selects a random permutation (ignoring parity).
// Applies a random orientation to each piece (ensuring the total is 0).
pub(crate) fn randomize_orbit_naive(
    pattern: &mut PackedKPattern,
    orbit_info: &PackedKPuzzleOrbitInfo,
    permutation_constraints: OrbitPermutationParityConstraints,
    orientation_constraints: OrbitOrientationParityConstraints,
) {
    match permutation_constraints {
        OrbitPermutationParityConstraints::IgnoreParity => {}
        OrbitPermutationParityConstraints::SingleOrbitEvenParity => panic!("unimplemented"),
    }

    let mut rng = thread_rng();
    let mut corner_order: Vec<u8> = vec![0, 1, 2, 3, 4, 5, 6, 7];
    corner_order.shuffle(&mut rng);
    let mut total_orientation = 0;
    for (i, p) in corner_order.into_iter().enumerate() {
        pattern
            .packed_orbit_data
            .set_packed_piece_or_permutation(orbit_info, i, p);
        let orientation = match orientation_constraints {
            OrbitOrientationParityConstraints::IgnoreParity => {
                let random_orientation = rng.gen_range(0..orbit_info.num_orientations);
                total_orientation = add_u8_mod(
                    total_orientation,
                    random_orientation,
                    orbit_info.num_orientations,
                );
                random_orientation
            }
            OrbitOrientationParityConstraints::OrientationsMustSumToZero => {
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
