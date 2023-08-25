use std::alloc::{alloc, dealloc};

use super::{packed_kpuzzle::PackedKPuzzleOrbitInfo, PackedKPuzzle};
pub struct PackedKTransformation {
    pub packed_kpuzzle: PackedKPuzzle,
    pub bytes: *mut u8,
}
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

    #[cfg(not(feature = "no_orientation_mod"))]
    pub fn unpack(&self) -> KTransformation {
        use std::sync::Arc;

        use cubing::kpuzzle::KTransformationData;

        use crate::packed::byte_conversions::u8_to_usize;

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
