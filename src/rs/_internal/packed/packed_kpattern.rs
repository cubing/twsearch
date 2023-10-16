use std::{fmt::Debug, hash::Hash, sync::Arc};

use super::{
    byte_conversions::{u8_to_usize, PackedOrientationWithMod},
    packed_kpuzzle::PackedKPuzzleOrbitInfo,
    packed_orbit_data::PackedOrbitData,
    ConversionError, PackedKPuzzle, PackedKTransformation,
};

use cubing::kpuzzle::KPuzzle;
use cubing::kpuzzle::{KPattern, KPatternData};

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct PackedKPattern {
    pub packed_orbit_data: PackedOrbitData,
}

impl PackedKPattern {
    pub fn new(packed_kpuzzle: PackedKPuzzle) -> Self {
        Self {
            packed_orbit_data: PackedOrbitData::new_with_uninitialized_bytes(packed_kpuzzle),
        }
    }

    pub fn try_from_json(
        packed_kpuzzle: &PackedKPuzzle,
        json_bytes: &[u8],
    ) -> Result<Self, ConversionError> {
        // TODO: implement this directly
        let kpattern_data: KPatternData = match serde_json::from_slice(json_bytes) {
            Ok(kpattern_data) => kpattern_data,
            Err(e) => {
                return Err(ConversionError::InvalidPatternData(
                    super::InvalidPatternDataError {
                        description: format!("Could not parse JSON for KPattern data: {}", e),
                    },
                ))
            }
        };
        let kpattern = KPattern {
            kpuzzle: packed_kpuzzle.data.kpuzzle.clone(),
            kpattern_data: Arc::new(kpattern_data),
        };
        packed_kpuzzle.try_pack_pattern(kpattern)
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

    pub fn set_packed_orientation(
        &mut self,
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

    // Adapted from https://github.com/cubing/cubing.rs/blob/b737c6a36528e9984b45b29f9449a9a330c272fb/src/kpuzzle/pattern.rs#L31-L82
    // TODO: dedup the implementation (but avoid runtime overhead for the shared abstraction).
    pub fn apply_transformation(&self, transformation: &PackedKTransformation) -> PackedKPattern {
        let mut new_packed_kpattern =
            PackedKPattern::new(self.packed_orbit_data.packed_kpuzzle.clone());
        self.apply_transformation_into(transformation, &mut new_packed_kpattern);
        new_packed_kpattern
    }

    // Adapted from https://github.com/cubing/cubing.rs/blob/b737c6a36528e9984b45b29f9449a9a330c272fb/src/kpuzzle/pattern.rs#L31-L82
    // TODO: dedup the implementation (but avoid runtime overhead for the shared abstraction).
    // TODO: assign to self from another value, not into another
    pub fn apply_transformation_into(
        &self,
        transformation: &PackedKTransformation,
        into_packed_kpattern: &mut PackedKPattern,
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
                into_packed_kpattern.set_piece_or_permutation(orbit_info, i, new_piece_value);

                let previous_packed_orientation =
                    self.get_packed_orientation(orbit_info, u8_to_usize(transformation_idx));

                let new_packed_orientation = {
                    orbit_info.orientation_packer.transform(
                        previous_packed_orientation,
                        u8_to_usize(transformation.get_orientation(orbit_info, i)),
                    )
                };
                into_packed_kpattern.set_packed_orientation(orbit_info, i, new_packed_orientation);
            }
        }
    }

    pub fn byte_slice(&self) -> &[u8] {
        self.packed_orbit_data.byte_slice()
    }

    pub fn hash(&self) -> u64 {
        self.packed_orbit_data.hash()
    }

    pub fn unpack(&self) -> KPattern {
        let mut kpattern_data = KPatternData::new();
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
            let orbit_data = cubing::kpuzzle::KPatternOrbitData {
                pieces,
                orientation,
                orientation_mod: Some(orientation_mod),
            };
            kpattern_data.insert(orbit_info.name.clone(), orbit_data);
        }
        KPattern {
            kpuzzle: self.packed_orbit_data.packed_kpuzzle.data.kpuzzle.clone(),
            kpattern_data: Arc::new(kpattern_data),
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

impl Debug for PackedKPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PackedKPattern")
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use cubing::alg::AlgParseError;
    use cubing::kpuzzle::{KPattern, KPatternData};
    use cubing::parse_move;
    use cubing::puzzles::cube3x3x3_kpuzzle;

    use crate::_internal::packed::packed_kpuzzle::ConversionError;
    use crate::_internal::{PackedKPuzzle, PackedKTransformation};

    #[test]
    fn compose() -> Result<(), String> {
        let kpuzzle = cube3x3x3_kpuzzle();
        let packed_kpuzzle = PackedKPuzzle::try_from(&kpuzzle).map_err(|e| e.description)?;

        let from_move = |move_str: &str| -> Result<PackedKTransformation, String> {
            let r#move = parse_move!(move_str).map_err(|e: AlgParseError| e.description)?;
            packed_kpuzzle
                .transformation_from_move(&r#move)
                .map_err(|e: ConversionError| e.to_string())
        };

        let start_pattern_data: KPatternData = serde_json::from_str(
            /* Cross */
            r#"
{
    "EDGES": {
        "pieces": [0, 0, 0, 0, 1, 2, 3, 4, 0, 0, 0, 0],
        "orientation": [1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1]
    },
    "CORNERS": {
        "pieces": [0, 0, 0, 0, 0, 0, 0, 0],
        "orientation": [1, 1, 1, 1, 1, 1, 1, 1]
    },
    "CENTERS": {
        "pieces": [0, 1, 2, 3, 4, 5],
        "orientation": [0, 0, 0, 0, 0, 0],
        "orientationMod": [1, 1, 1, 1, 1, 1]
    }
}"#,
        )
        .unwrap();
        let start_pattern = KPattern {
            kpuzzle,
            kpattern_data: Arc::new(start_pattern_data),
        };
        let packed_start_pattern = packed_kpuzzle
            .try_pack_pattern(start_pattern.clone())
            .unwrap();

        let t1 = from_move("R")?;

        assert_eq!(
            packed_start_pattern.apply_transformation(&t1).byte_slice(),
            vec![
                /* EP */ 0, 0, 0, 0, 1, 0, 3, 4, 2, 0, 0, 0, /* EO */ 1, 1, 1, 1, 0, 1,
                0, 0, 0, 1, 1, 1, /* CP */ 0, 0, 0, 0, 0, 0, 0, 0, /* CO */ 0, 2, 1, 1,
                2, 1, 1, 0, /* MP */ 0, 1, 2, 3, 4, 5, /* MO */ 4, 4, 4, 4, 4, 4
            ]
        );
        assert_eq!(
            packed_start_pattern.apply_transformation(&t1).byte_slice(),
            packed_kpuzzle
                .try_pack_pattern(
                    start_pattern
                        .apply_move(&parse_move!("R").unwrap())
                        .unwrap()
                )
                .unwrap()
                .byte_slice()
        );

        Ok(())
    }
}

pub struct PackedKPatternBuffer {
    a: PackedKPattern,
    b: PackedKPattern,
    // In some rough benchmarks, using a boolean to track the current pattern was just a tad faster than using `std::mem::swap(…)`.
    // TODO: measure this properly across devices, and updated `PackedKTransformationBuffer` to match.
    a_is_current: bool,
}

impl From<PackedKPattern> for PackedKPatternBuffer {
    fn from(initial: PackedKPattern) -> Self {
        Self {
            b: initial.clone(), // TODO?
            a: initial,
            a_is_current: true,
        }
    }
}

impl PackedKPatternBuffer {
    pub fn apply_transformation(&mut self, transformation: &PackedKTransformation) {
        if self.a_is_current {
            self.a
                .apply_transformation_into(transformation, &mut self.b);
        } else {
            self.b
                .apply_transformation_into(transformation, &mut self.a);
        }
        self.a_is_current = !self.a_is_current
    }

    pub fn current(&self) -> &PackedKPattern {
        if self.a_is_current {
            &self.a
        } else {
            &self.b
        }
    }
}

impl PartialEq for PackedKPatternBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.current() == other.current()
    }
}
