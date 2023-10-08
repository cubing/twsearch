use std::{fmt::Debug, hash::BuildHasher, mem::swap};

use super::{
    byte_conversions::{u8_to_usize, usize_to_u8},
    packed_kpuzzle::PackedKPuzzleOrbitInfo,
    packed_orbit_data::PackedOrbitData,
    PackedKPuzzle,
};

#[derive(Clone, Eq)]
pub struct PackedKTransformation {
    pub packed_orbit_data: PackedOrbitData,
}
use cubing::kpuzzle::{KPuzzle, KTransformation, KTransformationOrbitData};

impl PackedKTransformation {
    pub fn new(packed_kpuzzle: PackedKPuzzle) -> Self {
        Self {
            packed_orbit_data: PackedOrbitData::new_with_uninitialized_bytes(packed_kpuzzle),
        }
    }
    // TODO: dedup with PackedKTransformation, or at least implement as a trait?
    pub fn get_piece_or_permutation(&self, orbit_info: &PackedKPuzzleOrbitInfo, i: usize) -> u8 {
        unsafe {
            self.packed_orbit_data
                .bytes
                .add(orbit_info.pieces_or_pemutations_offset + i)
                .read()
        }
    }

    // TODO: dedup with PackedKTransformation, or at least implement as a trait?
    pub fn get_orientation(&self, orbit_info: &PackedKPuzzleOrbitInfo, i: usize) -> u8 {
        unsafe {
            self.packed_orbit_data
                .bytes
                .add(orbit_info.orientations_offset + i)
                .read()
        }
    }

    // TODO: dedup with PackedKTransformation, or at least implement as a trait?
    pub fn set_piece_or_permutation(
        &mut self,
        orbit_info: &PackedKPuzzleOrbitInfo,
        i: usize,
        value: u8,
    ) {
        unsafe {
            self.packed_orbit_data
                .bytes
                .add(orbit_info.pieces_or_pemutations_offset + i)
                .write(value)
        }
    }

    // TODO: dedup with PackedKTransformation, or at least implement as a trait?
    pub fn set_orientation(&mut self, orbit_info: &PackedKPuzzleOrbitInfo, i: usize, value: u8) {
        unsafe {
            self.packed_orbit_data
                .bytes
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
        let mut new_packed_ktransformation =
            PackedKTransformation::new(self.packed_orbit_data.packed_kpuzzle.clone());
        self.apply_transformation_into(transformation, &mut new_packed_ktransformation);
        new_packed_ktransformation
    }

    // Adapted from https://github.com/cubing/cubing.rs/blob/b737c6a36528e9984b45b29f9449a9a330c272fb/src/kpuzzle/transformation.rs#L32-L61
    // TODO: dedup the implementation (but avoid runtime overhead for the shared abstraction).
    // TODO: assign to self from another value, not into another
    pub fn apply_transformation_into(
        &self,
        transformation: &PackedKTransformation,
        into_packed_ktransformation: &mut PackedKTransformation,
    ) {
        for orbit_info in &self
            .packed_orbit_data
            .packed_kpuzzle
            .data
            .orbit_iteration_info
        {
            // TODO: optimization when either value is the identity.
            for i in 0..orbit_info.num_pieces {
                let transformation_idx = transformation.get_piece_or_permutation(orbit_info, i);

                let new_piece_permutation =
                    self.get_piece_or_permutation(orbit_info, u8_to_usize(transformation_idx));
                into_packed_ktransformation.set_piece_or_permutation(
                    orbit_info,
                    i,
                    new_piece_permutation,
                );

                let previous_packed_orientation =
                    self.get_orientation(orbit_info, u8_to_usize(transformation_idx));

                // TODO: lookup table?
                let new_orientation = (previous_packed_orientation
                    + transformation.get_orientation(orbit_info, i))
                    % orbit_info.num_orientations;
                into_packed_ktransformation.set_orientation(orbit_info, i, new_orientation);
            }
        }
    }

    pub fn byte_slice(&self) -> &[u8] {
        // yiss ☺️
        // https://stackoverflow.com/a/27150865
        unsafe {
            std::slice::from_raw_parts(
                self.packed_orbit_data.bytes,
                self.packed_orbit_data.packed_kpuzzle.data.num_bytes,
            )
        }
    }

    pub fn hash(&self) -> u64 {
        let h = cityhasher::CityHasher::new();
        h.hash_one(self.byte_slice())
    }

    pub fn unpack(&self) -> KTransformation {
        use std::sync::Arc;

        use cubing::kpuzzle::KTransformationData;

        let mut kpattern_data = KTransformationData::new();
        for orbit_info in &self
            .packed_orbit_data
            .packed_kpuzzle
            .data
            .orbit_iteration_info
        {
            let mut permutation = Vec::<usize>::new();
            let mut orientation_delta = Vec::<usize>::new();
            for i in 0..orbit_info.num_pieces {
                permutation.push(u8_to_usize(self.get_piece_or_permutation(orbit_info, i)));
                orientation_delta.push(u8_to_usize(self.get_orientation(orbit_info, i)));
            }
            let orbit_data = KTransformationOrbitData {
                permutation,
                orientation_delta,
            };
            kpattern_data.insert(orbit_info.name.clone(), orbit_data);
        }
        KTransformation {
            kpuzzle: self.packed_orbit_data.packed_kpuzzle.data.kpuzzle.clone(),
            ktransformation_data: Arc::new(kpattern_data),
        }
    }

    pub fn invert(&self) -> PackedKTransformation {
        let mut new_packed_ktransformation =
            PackedKTransformation::new(self.packed_orbit_data.packed_kpuzzle.clone());
        for orbit_info in &self
            .packed_orbit_data
            .packed_kpuzzle
            .data
            .orbit_iteration_info
        {
            let num_orientations = orbit_info.num_orientations;

            // TODO: optimization when either value is the identity.
            for i in 0..orbit_info.num_pieces {
                let from_idx = self.get_piece_or_permutation(orbit_info, i);
                new_packed_ktransformation.set_piece_or_permutation(
                    orbit_info,
                    u8_to_usize(from_idx),
                    usize_to_u8(i),
                );
                new_packed_ktransformation.set_orientation(
                    orbit_info,
                    u8_to_usize(from_idx),
                    (num_orientations - self.get_orientation(orbit_info, i))
                        .rem_euclid(num_orientations),
                )
            }
        }
        new_packed_ktransformation
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
                    kpuzzle: self.packed_orbit_data.packed_kpuzzle.data.kpuzzle.clone(),
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
    fn compose() -> Result<(), String> {
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

pub struct PackedKTransformationBuffer {
    pub current: PackedKTransformation,
    scratch_space: PackedKTransformation,
}

impl From<PackedKTransformation> for PackedKTransformationBuffer {
    fn from(initial: PackedKTransformation) -> Self {
        Self {
            scratch_space: initial.clone(), // TODO?
            current: initial,
        }
    }
}

impl PackedKTransformationBuffer {
    pub fn apply_transformation(&mut self, transformation: &PackedKTransformation) {
        self.current
            .apply_transformation_into(transformation, &mut self.scratch_space);
        swap(&mut self.current, &mut self.scratch_space);
    }
}

impl PartialEq for PackedKTransformationBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.current == other.current
    }
}
