use std::{
    alloc::{alloc, dealloc},
    fmt::Debug,
    marker::PhantomData,
    sync::Arc,
};

use cubing::kpuzzle::{KState, KStateData, KTransformation, KTransformationOrbitData};

use crate::PackedKPuzzle;

use super::{
    byte_conversions::{u8_to_usize, PackedOrientationWithMod},
    packed_kpuzzle::PackedKPuzzleOrbitInfo,
};

pub trait KStateOrKTransformation {
    fn transform(
        orbit_info: &PackedKPuzzleOrbitInfo,
        self_packed_orientation: PackedOrientationWithMod,
        transformation_packed_orientation: PackedOrientationWithMod,
    ) -> PackedOrientationWithMod;

    fn unpack(packed: &Packed<Self>) -> Self
    where
        Self: Sized;
    fn generic_name() -> &'static str;
}

impl KStateOrKTransformation for KState {
    fn transform(
        orbit_info: &PackedKPuzzleOrbitInfo,
        self_packed_orientation: PackedOrientationWithMod,
        transformation_packed_orientation: PackedOrientationWithMod,
    ) -> PackedOrientationWithMod {
        orbit_info.orientation_packer.transform(
            self_packed_orientation,
            u8_to_usize(transformation_packed_orientation),
        )
    }

    fn unpack(packed: &Packed<KState>) -> KState {
        let mut state_data = KStateData::new();
        for orbit_info in &packed.packed_kpuzzle.data.orbit_iteration_info {
            let mut pieces = Vec::<usize>::new();
            let mut orientation = Vec::<usize>::new();
            let mut orientation_mod = Vec::<usize>::new();
            for i in 0..orbit_info.num_pieces {
                pieces.push(u8_to_usize(
                    packed.get_packed_piece_or_permutation(orbit_info, i),
                ));
                let orientation_with_mod = orbit_info
                    .orientation_packer
                    .unpack(packed.get_packed_orientation(orbit_info, i));
                orientation.push(orientation_with_mod.orientation);
                orientation_mod.push(orientation_with_mod.orientation_mod);
            }
            let orbit_data = cubing::kpuzzle::KStateOrbitData {
                pieces,
                orientation,
                orientation_mod: Some(orientation_mod),
            };
            state_data.insert(orbit_info.name.clone(), orbit_data);
        }
        KState {
            kpuzzle: packed.packed_kpuzzle.data.kpuzzle.clone(),
            state_data: Arc::new(state_data),
        }
    }

    fn generic_name() -> &'static str {
        "KState"
    }
}

impl KStateOrKTransformation for KTransformation {
    fn transform(
        orbit_info: &PackedKPuzzleOrbitInfo,
        self_packed_orientation: PackedOrientationWithMod,
        transformation_packed_orientation: PackedOrientationWithMod,
    ) -> PackedOrientationWithMod {
        (self_packed_orientation + transformation_packed_orientation) % orbit_info.num_orientations
    }

    fn unpack(packed: &Packed<KTransformation>) -> KTransformation {
        use cubing::kpuzzle::KTransformationData;

        let mut state_data = KTransformationData::new();
        for orbit_info in &packed.packed_kpuzzle.data.orbit_iteration_info {
            let mut permutation = Vec::<usize>::new();
            let mut orientation = Vec::<usize>::new();
            for i in 0..orbit_info.num_pieces {
                permutation.push(u8_to_usize(
                    packed.get_packed_piece_or_permutation(orbit_info, i),
                ));
                orientation.push(u8_to_usize(packed.get_packed_orientation(orbit_info, i)));
            }
            let orbit_data = KTransformationOrbitData {
                permutation,
                orientation,
            };
            state_data.insert(orbit_info.name.clone(), orbit_data);
        }
        KTransformation {
            kpuzzle: packed.packed_kpuzzle.data.kpuzzle.clone(),
            transformation_data: Arc::new(state_data),
        }
    }

    fn generic_name() -> &'static str {
        "KTransformation"
    }
}

pub trait PackedOrbitDataDelegate<K: KStateOrKTransformation> {
    fn unpack(&self) -> K;
}

pub struct Packed<K: KStateOrKTransformation> {
    pub packed_kpuzzle: PackedKPuzzle,
    pub bytes: *mut u8,
    phantom: PhantomData<K>,
}

impl<K: KStateOrKTransformation> Drop for Packed<K> {
    fn drop(&mut self) {
        unsafe { dealloc(self.bytes, self.packed_kpuzzle.data.layout) }
    }
}

impl<K: KStateOrKTransformation> PartialEq<Packed<K>> for Packed<K> {
    fn eq(&self, other: &Self) -> bool {
        self.byte_slice() == other.byte_slice()
    }
}

impl<K: KStateOrKTransformation> Packed<K> {
    pub fn new(packed_kpuzzle: PackedKPuzzle) -> Self {
        let bytes = unsafe { alloc(packed_kpuzzle.data.layout) };
        Self {
            packed_kpuzzle,
            bytes,
            phantom: PhantomData,
        }
    }

    pub fn get_packed_piece_or_permutation(
        &self,
        orbit_info: &PackedKPuzzleOrbitInfo,
        i: usize,
    ) -> u8 {
        unsafe {
            self.bytes
                .add(orbit_info.pieces_or_pemutations_offset + i)
                .read()
        }
    }

    pub fn get_packed_orientation(
        &self,
        orbit_info: &PackedKPuzzleOrbitInfo,
        i: usize,
    ) -> PackedOrientationWithMod {
        unsafe { self.bytes.add(orbit_info.orientations_offset + i).read() }
    }

    pub fn set_packed_piece_or_permutation(
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

    pub fn set_packed_orientation(
        &self,
        orbit_info: &PackedKPuzzleOrbitInfo,
        i: usize,
        value: PackedOrientationWithMod,
    ) {
        unsafe {
            self.bytes
                .add(orbit_info.orientations_offset + i)
                .write(value)
        }
    }

    pub fn apply_transformation(&self, transformation: &Packed<KTransformation>) -> Self {
        let mut new_self = Self::new(self.packed_kpuzzle.clone());
        self.apply_transformation_into(transformation, &mut new_self);
        new_self
    }

    // TODO: assign to self from another value, not into another
    pub fn apply_transformation_into(
        &self,
        transformation: &Packed<KTransformation>,
        into_other: &mut Self,
    ) {
        for orbit_info in &self.packed_kpuzzle.data.orbit_iteration_info {
            // TODO: optimization when either value is the identity.
            for i in 0..orbit_info.num_pieces {
                let transformation_idx =
                    transformation.get_packed_piece_or_permutation(orbit_info, i);

                let new_piece_permutation = self
                    .get_packed_piece_or_permutation(orbit_info, u8_to_usize(transformation_idx));
                into_other.set_packed_piece_or_permutation(orbit_info, i, new_piece_permutation);

                let previous_packed_orientation =
                    self.get_packed_orientation(orbit_info, u8_to_usize(transformation_idx));

                // TODO: lookup table?
                let new_orientation = K::transform(
                    orbit_info,
                    previous_packed_orientation,
                    transformation.get_packed_orientation(orbit_info, i),
                );
                into_other.set_packed_orientation(orbit_info, i, new_orientation);
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

    pub fn unpack(&self) -> K {
        K::unpack(self)
    }
}

impl<K: KStateOrKTransformation> Debug for Packed<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("PackedOrbitData<{}>", K::generic_name()))
            .field("packed_kpuzzle", &self.packed_kpuzzle)
            .field("bytes", &self.byte_slice())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use cubing::alg::AlgParseError;
    use cubing::kpuzzle::KTransformation;
    use cubing::parse_move;
    use cubing::puzzles::cube3x3x3_kpuzzle;

    use crate::packed::packed_kpuzzle::ConversionError;
    use crate::{Packed, PackedKPuzzle};

    #[test]
    fn test_orientation_mod() -> Result<(), String> {
        let kpuzzle = cube3x3x3_kpuzzle();
        let packed_kpuzzle = PackedKPuzzle::try_from(kpuzzle).map_err(|e| e.description)?;

        let from_move = |move_str: &str| -> Result<Packed<KTransformation>, String> {
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
