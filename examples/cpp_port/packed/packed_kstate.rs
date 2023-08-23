use std::alloc::{alloc, dealloc};

use super::{
    packed_kpuzzle::PackedKPuzzleOrbitInfo, packed_part::PackedPart, PackedKPuzzle,
    PackedKTransformation,
};

pub struct PackedKState {
    pub packed_kpuzzle: PackedKPuzzle,
    pub bytes: *mut u8,
}

impl Drop for PackedKState {
    fn drop(&mut self) {
        unsafe { dealloc(self.bytes, self.packed_kpuzzle.data.layout) }
    }
}

impl PackedKState {
    pub fn new(packed_kpuzzle: PackedKPuzzle) -> Self {
        let bytes = unsafe { alloc(packed_kpuzzle.data.layout) };
        Self {
            packed_kpuzzle,
            bytes,
        }
    }

    pub fn set_part(
        &self,
        packed_part: PackedPart,
        orbit_info: &PackedKPuzzleOrbitInfo,
        i: usize,
        value: u8,
    ) {
        unsafe {
            self.bytes
                .add(
                    orbit_info.bytes_offset
                        + orbit_info.num_pieces * packed_part.offset_multiplier()
                        + i,
                )
                .write(value)
        }
    }

    pub fn get_part(
        &self,
        packed_part: PackedPart,
        orbit_info: &PackedKPuzzleOrbitInfo,
        i: usize,
    ) -> u8 {
        unsafe {
            self.bytes
                .add(
                    orbit_info.bytes_offset
                        + orbit_info.num_pieces * packed_part.offset_multiplier()
                        + i,
                )
                .read()
        }
    }

    // Adapted from https://github.com/cubing/cubing.rs/blob/b737c6a36528e9984b45b29f9449a9a330c272fb/src/kpuzzle/state.rs#L31-L82
    // TODO: dedup the implementation (but avoid runtime overhead for the shared abstraction).
    pub fn apply_transformation(
        &self,
        packed_kpuzzle: &PackedKPuzzle,
        transformation: &PackedKTransformation,
    ) -> PackedKState {
        let mut new_state = PackedKState::new(self.packed_kpuzzle.clone());
        self.apply_transformation_into(packed_kpuzzle, transformation, &mut new_state);
        new_state
    }

    // Adapted from https://github.com/cubing/cubing.rs/blob/b737c6a36528e9984b45b29f9449a9a330c272fb/src/kpuzzle/state.rs#L31-L82
    // TODO: dedup the implementation (but avoid runtime overhead for the shared abstraction).
    pub fn apply_transformation_into(
        &self,
        packed_kpuzzle: &PackedKPuzzle,
        transformation: &PackedKTransformation,
        into_state: &mut PackedKState,
    ) {
        for orbit_info in &packed_kpuzzle.data.orbit_iteration_info {
            // TODO: optimization when either value is the identity.
            for i in 0..orbit_info.num_pieces {
                let transformation_idx =
                    transformation.get_part(PackedPart::Permutation, orbit_info, i);

                let new_piece_permutation = self.get_part(
                    PackedPart::Permutation,
                    orbit_info,
                    std::convert::Into::<usize>::into(transformation_idx),
                );
                into_state.set_part(
                    PackedPart::Permutation,
                    orbit_info,
                    i,
                    new_piece_permutation,
                );

                let previous_piece_orientation = self.get_part(
                    PackedPart::Orientation,
                    orbit_info,
                    std::convert::Into::<usize>::into(transformation_idx),
                );
                // TODO: the lookup table doesn't seem to be significantly faster on M1 Max. Test if it helps significantly in other environments.
                // let new_piece_orientation = orbit_info.table[std::convert::Into::<usize>::into(
                //     previous_piece_orientation + transformation.get_orientation(orbit_info, i),
                // )];
                let new_piece_orientation =
                    if previous_piece_orientation == orbit_info.unknown_orientation_value {
                        previous_piece_orientation
                    } else {
                        (previous_piece_orientation
                            + transformation.get_part(PackedPart::Orientation, orbit_info, i))
                            % orbit_info.num_orientations
                    };
                into_state.set_part(
                    PackedPart::Orientation,
                    orbit_info,
                    i,
                    new_piece_orientation,
                );
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
}
