use std::{fmt::Debug, sync::Arc};

use super::{
    byte_conversions::{u8_to_usize, PackedOrientationWithMod},
    packed_kpuzzle::PackedKPuzzleOrbitInfo,
    packed_orbit_data::PackedOrbitData,
    PackedKPuzzle, PackedKTransformation,
};

use cubing::kpuzzle::KPuzzle;
use cubing::kpuzzle::{KState, KStateData};

pub struct PackedKState {
    pub packed_orbit_data: PackedOrbitData,
}

impl PackedKState {
    pub fn new(packed_kpuzzle: PackedKPuzzle) -> Self {
        Self {
            packed_orbit_data: PackedOrbitData::new(packed_kpuzzle),
        }
    }

    pub fn get_piece_or_permutation(&self, orbit_info: &PackedKPuzzleOrbitInfo, i: usize) -> u8 {
        unsafe {
            self.packed_orbit_data
                .bytes
                .add(orbit_info.pieces_or_pemutations_offset + i)
                .read()
        }
    }

    pub fn get_packed_orientation(
        &self,
        orbit_info: &PackedKPuzzleOrbitInfo,
        i: usize,
    ) -> PackedOrientationWithMod {
        unsafe {
            self.packed_orbit_data
                .bytes
                .add(orbit_info.orientations_offset + i)
                .read()
        }
    }

    pub fn set_piece_or_permutation(
        &self,
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

    pub fn set_packed_orientation(
        &self,
        orbit_info: &PackedKPuzzleOrbitInfo,
        i: usize,
        value: PackedOrientationWithMod,
    ) {
        unsafe {
            self.packed_orbit_data
                .bytes
                .add(orbit_info.orientations_offset + i)
                .write(value)
        }
    }

    // Adapted from https://github.com/cubing/cubing.rs/blob/b737c6a36528e9984b45b29f9449a9a330c272fb/src/kpuzzle/state.rs#L31-L82
    // TODO: dedup the implementation (but avoid runtime overhead for the shared abstraction).
    pub fn apply_transformation(&self, transformation: &PackedKTransformation) -> PackedKState {
        let mut new_state = PackedKState::new(self.packed_orbit_data.packed_kpuzzle.clone());
        self.apply_transformation_into(transformation, &mut new_state);
        new_state
    }

    // Adapted from https://github.com/cubing/cubing.rs/blob/b737c6a36528e9984b45b29f9449a9a330c272fb/src/kpuzzle/state.rs#L31-L82
    // TODO: dedup the implementation (but avoid runtime overhead for the shared abstraction).
    // TODO: assign to self from another value, not into another
    pub fn apply_transformation_into(
        &self,
        transformation: &PackedKTransformation,
        into_state: &mut PackedKState,
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

                let new_piece_value =
                    self.get_piece_or_permutation(orbit_info, u8_to_usize(transformation_idx));
                into_state.set_piece_or_permutation(orbit_info, i, new_piece_value);

                let previous_packed_orientation =
                    self.get_packed_orientation(orbit_info, u8_to_usize(transformation_idx));

                let new_packed_orientation = {
                    orbit_info.orientation_packer.transform(
                        previous_packed_orientation,
                        u8_to_usize(transformation.get_orientation(orbit_info, i)),
                    )
                };
                into_state.set_packed_orientation(orbit_info, i, new_packed_orientation);
            }
        }
    }

    pub fn byte_slice(&self) -> &[u8] {
        self.packed_orbit_data.byte_slice()
    }

    pub fn hash(&self) -> u64 {
        self.packed_orbit_data.hash()
    }

    pub fn unpack(&self) -> KState {
        let mut state_data = KStateData::new();
        for orbit_info in &self
            .packed_orbit_data
            .packed_kpuzzle
            .data
            .orbit_iteration_info
        {
            let mut pieces = Vec::<usize>::new();
            let mut orientation = Vec::<usize>::new();
            let mut orientation_mod = Vec::<usize>::new();
            for i in 0..orbit_info.num_pieces {
                pieces.push(u8_to_usize(self.get_piece_or_permutation(orbit_info, i)));
                let orientation_with_mod = orbit_info
                    .orientation_packer
                    .unpack(self.get_packed_orientation(orbit_info, i));
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
            kpuzzle: self.packed_orbit_data.packed_kpuzzle.data.kpuzzle.clone(),
            state_data: Arc::new(state_data),
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

impl Debug for PackedKState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackedKState")
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
