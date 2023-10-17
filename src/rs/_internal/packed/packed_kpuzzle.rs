use std::{alloc::Layout, sync::Arc, fmt::Debug};

use cubing::{
    alg::{Move, Alg},
    kpuzzle::{
        InvalidAlgError, InvalidDefinitionError, KPuzzle, KPuzzleOrbitName, KTransformation, KPattern, KPuzzleDefinition,
    },
};

use crate::_internal::PackedKPattern;

use super::{byte_conversions::{usize_to_u8, u8_to_usize},  PackedKTransformation, orientation_packer::{OrientationPacker, OrientationWithMod}, InvalidPatternDataError};

// TODO: allow certain values over 107?
const MAX_NUM_ORIENTATIONS_INCLUSIVE: usize = 107;

#[derive(Debug)]
pub struct PackedKPuzzleOrbitInfo {
    pub name: KPuzzleOrbitName,
    pub pieces_or_pemutations_offset: usize,
    pub orientations_offset: usize,
    pub num_pieces: usize,
    pub num_orientations: u8,
    pub orientation_packer: OrientationPacker,
}

#[derive(Debug)]
pub struct PackedKPuzzleData {
    pub kpuzzle: KPuzzle,
    // Private cached values.
    pub num_bytes: usize,
    pub orbit_iteration_info: Vec<PackedKPuzzleOrbitInfo>,
    pub layout: Layout,
}

#[derive(Clone)]
pub struct PackedKPuzzle {
    pub data: Arc<PackedKPuzzleData>, // TODO
                                      // pub data: PackedKPuzzleData,
}

impl TryFrom<&KPuzzle> for PackedKPuzzle {
    type Error = InvalidDefinitionError;

    fn try_from(kpuzzle: &KPuzzle) -> Result<Self, Self::Error> {
        PackedKPuzzle::try_from(kpuzzle.clone())
    }
}

impl TryFrom<KPuzzle> for PackedKPuzzle {
    type Error = InvalidDefinitionError;

    fn try_from(kpuzzle: KPuzzle) -> Result<Self, Self::Error> {
        let def = kpuzzle.definition();

        let mut bytes_offset = 0;
        let mut orbit_iteration_info: Vec<PackedKPuzzleOrbitInfo> = vec![];

        for orbit_definition in &def.orbits {
            let num_orientations = orbit_definition.num_orientations;
            if num_orientations > MAX_NUM_ORIENTATIONS_INCLUSIVE {
                return Err(InvalidDefinitionError { description: format!("`num_orientations` for orbit {} is too large ({}). Maximum is {} for the current build." , orbit_definition.orbit_name, num_orientations, MAX_NUM_ORIENTATIONS_INCLUSIVE)});
            }
            orbit_iteration_info.push({
                PackedKPuzzleOrbitInfo {
                    name: orbit_definition.orbit_name.clone(),
                    num_pieces: orbit_definition.num_pieces,
                    num_orientations: usize_to_u8(num_orientations),
                    pieces_or_pemutations_offset: bytes_offset,
                    orientations_offset: bytes_offset + orbit_definition.num_pieces,
                    orientation_packer: OrientationPacker::new(orbit_definition.num_orientations),
                }
            });
            bytes_offset += orbit_definition.num_pieces * 2;
        }

        Ok(Self {
            data: Arc::new(PackedKPuzzleData {
                kpuzzle,
                num_bytes: bytes_offset,
                orbit_iteration_info,
                layout: Layout::array::<u8>(bytes_offset).map_err(|_| InvalidDefinitionError {
                    description: "Could not construct packed layout.".to_owned(),
                })?,
            }),
        })
    }
}

impl TryFrom<&[u8]> for PackedKPuzzle {
    type Error = InvalidDefinitionError;

    fn try_from(json_bytes: &[u8]) -> Result<Self, Self::Error>  {
        // TODO: implement this directly
        let kpuzzle_data: KPuzzleDefinition = match serde_json::from_slice(json_bytes) {
            Ok(kpuzzle_data) => kpuzzle_data,
            Err(e) => {
                return Err(InvalidDefinitionError { description: e.to_string().to_owned() })
            }
        };
        let kpuzzle = KPuzzle::try_new(Arc::new(kpuzzle_data))?;
        Self::try_from(kpuzzle)
    }
}

/// An error type that can indicate multiple error causes, when parsing and applying an alg at the same time.
#[derive(derive_more::From, Debug, derive_more::Display)]
pub enum ConversionError {
    InvalidAlg(InvalidAlgError),
    InvalidDefinition(InvalidDefinitionError),
    InvalidPatternData(InvalidPatternDataError),
}

impl PackedKPuzzle {
    pub fn default_pattern(&self) -> PackedKPattern {
        // TODO: check that `KPuzzle`s match?
        self.try_pack_pattern(self.data.kpuzzle.default_pattern()).expect("Default pattern is invalid?") // TODO: cache at construction time instead
    }

    // TODO: `try_pack_pattern`?
    pub fn try_pack_pattern(&self, pattern: KPattern) -> Result<PackedKPattern , ConversionError>{
        let pattern_data = pattern.kpattern_data;

        let mut new_packed_kpattern = PackedKPattern::new(self.clone());
        for orbit_info in &self.data.orbit_iteration_info {
            let orbit_data = match pattern_data
                .get(&orbit_info.name)
                 {
                    Some(orbit_data) => orbit_data,
                    None => {return Err( 
                        ConversionError::InvalidPatternData(InvalidPatternDataError {
                            description: format!("Missing data for orbit: {}", orbit_info.name),
                        })
                        )}
                };
            for i in 0..orbit_info.num_pieces {
                new_packed_kpattern.set_piece_or_permutation(
                    orbit_info,
                    i,
                    usize_to_u8(orbit_data.pieces[i]),
                );
                new_packed_kpattern.set_packed_orientation(
                    orbit_info,
                    i,
                    match &orbit_data.orientation_mod {
                        None => usize_to_u8(orbit_data.orientation[i]),
                        Some(orientation_mod) => {
                                if orientation_mod[i] != 0 && u8_to_usize(orbit_info.num_orientations) % orientation_mod[i] != 0 {
                                    return Err(ConversionError::InvalidPatternData(InvalidPatternDataError { description: format!(
                                        "`orientation_mod` of {} seen for piece at index {} in orbit {} in the start pattern for puzzle {}. This must be a factor of `num_orientations` for the orbit ({}). See: https://js.cubing.net/cubing/api/interfaces/kpuzzle.KPatternOrbitData.html#orientationMod",
                                        orientation_mod[i],
                                        i,
                                        orbit_info.name,
                                        self.data.kpuzzle.definition().name,
                                        orbit_info.num_orientations
                                    )}));
                                };
                                orbit_info.orientation_packer.pack(OrientationWithMod {
                                    orientation: orbit_data.orientation[i],
                                    orientation_mod: orientation_mod[i],
                                })
                            
                        }
                    },
                );
            }
        }

        Ok(new_packed_kpattern)
    }

    pub fn identity_transformation(&self) -> Result<PackedKTransformation, ConversionError> {
        let unpacked_ktransformation = self.data.kpuzzle.identity_transformation();
        self.pack_transformation(&unpacked_ktransformation)
    }

    // TODO: implement this as a `TryFrom`?
    pub fn transformation_from_move(
        &self,
        r#move: &Move,
    ) -> Result<PackedKTransformation, ConversionError> {
        let unpacked_ktransformation = self.data.kpuzzle.transformation_from_move(r#move)?;
        self.pack_transformation(&unpacked_ktransformation)
    }

    // TODO: implement this directly
    pub fn transformation_from_alg(
        &self,
        alg: &Alg,
    ) -> Result<PackedKTransformation, ConversionError> {
        let unpacked_ktransformation = self.data.kpuzzle.transformation_from_alg(alg)?;
        self.pack_transformation(&unpacked_ktransformation)
    }

    pub fn pack_transformation(
        &self,
        unpacked_ktransformation: &KTransformation,
    ) -> Result<PackedKTransformation, ConversionError> {
        let mut new_transformation = PackedKTransformation::new(self.clone());
        for orbit_info in &self.data.orbit_iteration_info {
            let unpacked_orbit_data = unpacked_ktransformation
                .ktransformation_data
                .get(&orbit_info.name);
            let unpacked_orbit_data =
                unpacked_orbit_data.ok_or_else(|| InvalidDefinitionError {
                    description: format!("Missing orbit: {}", orbit_info.name),
                })?;
            for i in 0..orbit_info.num_pieces {
                new_transformation.set_piece_or_permutation(
                    orbit_info,
                    i,
                    usize_to_u8(unpacked_orbit_data.permutation[i]),
                );
                new_transformation.set_orientation(
                    orbit_info,
                    i,
                    usize_to_u8(unpacked_orbit_data.orientation_delta[i]),
                )
            }
        }

        Ok(new_transformation)
    }
}

impl Debug for PackedKPuzzle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ … name: \"{}\" … }}", &self.data.kpuzzle.definition().name)
    }
}
