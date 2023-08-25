use std::{
    alloc::{alloc, dealloc},
    fmt::Debug,
};

use super::{byte_conversions::u8_to_usize, packed_kpuzzle::PackedKPuzzleOrbitInfo, PackedKPuzzle};
pub struct PackedKTransformation {
    pub packed_kpuzzle: PackedKPuzzle,
    pub bytes: *mut u8,
}
use cubing::kpuzzle::KPuzzle;

#[cfg(not(feature = "no_orientation_mod"))]
use cubing::kpuzzle::{KTransformation, KTransformationOrbitData};

impl Drop for PackedKTransformation {
    fn drop(&mut self) {
        unsafe { dealloc(self.bytes, self.packed_kpuzzle.data.layout) }
    }
}

impl PackedKTransformation {
    pub fn new(packed_kpuzzle: PackedKPuzzle) -> Self {
        let bytes = unsafe { alloc(packed_kpuzzle.data.layout) };
        Self {
            packed_kpuzzle,
            bytes,
        }
    }
    // TODO: dedup with PackedKTransformation, or at least implement as a trait?
    pub fn get_piece_or_permutation(&self, orbit_info: &PackedKPuzzleOrbitInfo, i: usize) -> u8 {
        unsafe {
            self.bytes
                .add(orbit_info.pieces_or_pemutations_offset + i)
                .read()
        }
    }

    // TODO: dedup with PackedKTransformation, or at least implement as a trait?
    pub fn get_orientation(&self, orbit_info: &PackedKPuzzleOrbitInfo, i: usize) -> u8 {
        unsafe { self.bytes.add(orbit_info.orientations_offset + i).read() }
    }

    // TODO: dedup with PackedKTransformation, or at least implement as a trait?
    pub fn set_piece_or_permutation(
        &self,
        orbit_info: &PackedKPuzzleOrbitInfo,
        i: usize,
        value: u8,
    ) {
        unsafe {
            self.bytes
                .add(orbit_info.pieces_or_pemutations_offset + i)
                .write(value)
        }
    }

    // TODO: dedup with PackedKTransformation, or at least implement as a trait?
    pub fn set_orientation(&self, orbit_info: &PackedKPuzzleOrbitInfo, i: usize, value: u8) {
        unsafe {
            self.bytes
                .add(orbit_info.orientations_offset + i)
                .write(value)
        }
    }

    // Adapted from https://github.com/cubing/cubing.rs/blob/b737c6a36528e9984b45b29f9449a9a330c272fb/src/kpuzzle/transformation.rs#L32-L61
    // TODO: dedup the implementation (but avoid runtime overhead for the shared abstraction).
    pub fn apply_transformation(
        &self,
        transformation: &PackedKTransformation,
    ) -> PackedKTransformation {
        let mut new_state = PackedKTransformation::new(self.packed_kpuzzle.clone());
        self.apply_transformation_into(transformation, &mut new_state);
        new_state
    }

    // Adapted from https://github.com/cubing/cubing.rs/blob/b737c6a36528e9984b45b29f9449a9a330c272fb/src/kpuzzle/transformation.rs#L32-L61
    // TODO: dedup the implementation (but avoid runtime overhead for the shared abstraction).
    // TODO: assign to self from another value, not into another
    pub fn apply_transformation_into(
        &self,
        transformation: &PackedKTransformation,
        into_state: &mut PackedKTransformation,
    ) {
        for orbit_info in &self.packed_kpuzzle.data.orbit_iteration_info {
            // TODO: optimization when either value is the identity.
            for i in 0..orbit_info.num_pieces {
                let transformation_idx = transformation.get_piece_or_permutation(orbit_info, i);

                let new_piece_permutation =
                    self.get_piece_or_permutation(orbit_info, u8_to_usize(transformation_idx));
                into_state.set_piece_or_permutation(orbit_info, i, new_piece_permutation);

                let previous_packed_orientation =
                    self.get_orientation(orbit_info, u8_to_usize(transformation_idx));

                // TODO: lookup table?
                let new_orientation = (previous_packed_orientation
                    + transformation.get_orientation(orbit_info, i))
                    % orbit_info.num_orientations;
                into_state.set_orientation(orbit_info, i, new_orientation);
            }
        }
    }

    pub fn byte_slice(&self) -> &[u8] {
        // yiss ☺️
        // https://stackoverflow.com/a/27150865
        unsafe { std::slice::from_raw_parts(self.bytes, self.packed_kpuzzle.data.num_bytes) }
    }

    pub fn hash(&self) -> u64 {
        cityhash::city_hash_64(self.byte_slice())
    }

    #[cfg(not(feature = "no_orientation_mod"))]
    pub fn unpack(&self) -> KTransformation {
        use std::sync::Arc;

        use cubing::kpuzzle::KTransformationData;

        let mut state_data = KTransformationData::new();
        for orbit_info in &self.packed_kpuzzle.data.orbit_iteration_info {
            let mut permutation = Vec::<usize>::new();
            let mut orientation = Vec::<usize>::new();
            for i in 0..orbit_info.num_pieces {
                permutation.push(u8_to_usize(self.get_piece_or_permutation(orbit_info, i)));
                orientation.push(u8_to_usize(self.get_orientation(orbit_info, i)));
            }
            let orbit_data = KTransformationOrbitData {
                permutation,
                orientation,
            };
            state_data.insert(orbit_info.name.clone(), orbit_data);
        }
        KTransformation {
            kpuzzle: self.packed_kpuzzle.data.kpuzzle.clone(),
            transformation_data: Arc::new(state_data),
        }
    }
}

struct KPuzzleDebug {
    kpuzzle: KPuzzle,
}

impl Debug for KPuzzleDebug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ … name: \"{}\" … }}", &self.kpuzzle.definition().name)
    }
}

impl Debug for PackedKTransformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackedKTransformation")
            .field(
                "packed_kpuzzle",
                &KPuzzleDebug {
                    kpuzzle: self.packed_kpuzzle.data.kpuzzle.clone(),
                },
            )
            .field("bytes", &self.byte_slice())
            .finish()
    }
}

impl PartialEq<PackedKTransformation> for PackedKTransformation {
    fn eq(&self, other: &Self) -> bool {
        self.byte_slice() == other.byte_slice()
    }
}

#[cfg(test)]
mod tests {
    use cubing::alg::AlgParseError;
    use cubing::parse_move;
    use cubing::puzzles::cube3x3x3_kpuzzle;

    use crate::packed::packed_kpuzzle::ConversionError;
    use crate::{PackedKPuzzle, PackedKTransformation};

    #[test]
    fn test_orientation_mod() -> Result<(), String> {
        let kpuzzle = cube3x3x3_kpuzzle();
        let packed_kpuzzle = PackedKPuzzle::try_from(kpuzzle).map_err(|e| e.description)?;

        let from_move = |move_str: &str| -> Result<PackedKTransformation, String> {
            let r#move = parse_move!(move_str).map_err(|e: AlgParseError| e.description)?;
            packed_kpuzzle
                .transformation_from_move(&r#move)
                .map_err(|e: ConversionError| e.to_string())
        };

        let id = packed_kpuzzle
            .identity_transformation()
            .map_err(|e| e.to_string())?;
        let t1 = from_move("R")?;
        let t2 = from_move("R2")?;
        let t2prime = from_move("R2'")?;
        let t4 = from_move("R4")?;
        let t5 = from_move("R5")?;

        assert_eq!(id, t4);
        assert_eq!(t1, t5);
        assert_eq!(t2, t2prime);

        assert_ne!(id, t1);
        assert_ne!(id, t2);
        assert_ne!(t1, t2);

        assert_eq!(id.apply_transformation(&t1), t1);
        assert_eq!(t1.apply_transformation(&t1), t2);
        assert_eq!(t2.apply_transformation(&t1).apply_transformation(&t2), t1);

        Ok(())
    }
}
