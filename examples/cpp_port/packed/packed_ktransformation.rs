use std::alloc::{alloc, dealloc};

use super::{packed_kpuzzle::PackedKPuzzleOrbitInfo, packed_part::PackedPart, PackedKPuzzle};
pub struct PackedKTransformation {
    pub packed_kpuzzle: PackedKPuzzle,
    pub bytes: *mut u8,
}

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

    // TODO: dedup with PackedKTransformation, or at least implement as a trait?
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

    pub fn byte_slice(&self) -> &[u8] {
        // yiss ☺️
        // https://stackoverflow.com/a/27150865
        unsafe { std::slice::from_raw_parts(self.bytes, self.packed_kpuzzle.data.num_bytes) }
    }

    pub fn hash(&self) -> u64 {
        cityhash::city_hash_64(self.byte_slice())
    }
}
