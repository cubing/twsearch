use std::{
    alloc::{alloc, dealloc},
    fmt::Debug,
    hash::{BuildHasher, Hash},
};

use super::{
    byte_conversions::PackedOrientationWithMod, packed_kpuzzle::PackedKPuzzleOrbitInfo,
    PackedKPuzzle, PackedKTransformation,
};

pub struct PackedOrbitData {
    pub packed_kpuzzle: PackedKPuzzle,
    pub bytes: *mut u8,
}

impl Drop for PackedOrbitData {
    fn drop(&mut self) {
        unsafe { dealloc(self.bytes, self.packed_kpuzzle.data.layout) }
    }
}

trait KPatternOrKTransformation {
    fn apply_transformation(&self, transformation: &Self) -> Self;
    fn apply_transformation_into(
        &self,
        transformation: &PackedKTransformation,
        into_other: &mut Self,
    );

    fn byte_slice(&self) -> &[u8];

    fn hash(&self);

    // pub fn unpack(&self) -> KPattern | KTransformation; // TODO
}

impl PackedOrbitData {
    pub fn new_with_uninitialized_bytes(packed_kpuzzle: PackedKPuzzle) -> Self {
        let bytes = unsafe { alloc(packed_kpuzzle.data.layout) };
        Self {
            packed_kpuzzle,
            bytes,
        }
    }

    pub fn get_packed_piece_or_permutation(
        &self,
        orbit_info: &PackedKPuzzleOrbitInfo,
        i: usize,
    ) -> u8 {
        unsafe {
            self.bytes
                .add(orbit_info.pieces_or_permutations_offset + i)
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
                .add(orbit_info.pieces_or_permutations_offset + i)
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

    pub fn byte_slice(&self) -> &[u8] {
        // yiss ☺️
        // https://stackoverflow.com/a/27150865
        unsafe { std::slice::from_raw_parts(self.bytes, self.packed_kpuzzle.data.num_bytes) }
    }

    pub fn hash(&self) -> u64 {
        let h = cityhasher::CityHasher::new();
        h.hash_one(self.byte_slice())
    }
}

impl Debug for PackedOrbitData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackedKPattern")
            .field("packed_kpuzzle", &self.packed_kpuzzle)
            .field("bytes", &self.byte_slice())
            .finish()
    }
}

impl Hash for PackedOrbitData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.byte_slice().hash(state); // TODO: would hashing the kpuzzle significantly affect performance?
    }
}

impl PartialEq for PackedOrbitData {
    fn eq(&self, other: &Self) -> bool {
        // TODO: would comparing the kpuzzles significantly affect performance?
        self.byte_slice() == other.byte_slice()
    }
}

impl Eq for PackedOrbitData {}

impl Clone for PackedOrbitData {
    fn clone(&self) -> Self {
        let new_packed_orbit_data =
            PackedOrbitData::new_with_uninitialized_bytes(self.packed_kpuzzle.clone());
        unsafe {
            std::ptr::copy(
                self.bytes,
                new_packed_orbit_data.bytes,
                self.packed_kpuzzle.data.num_bytes,
            )
        };
        new_packed_orbit_data
    }
}

// TODO
unsafe impl Send for PackedOrbitData {}
unsafe impl Sync for PackedOrbitData {}
