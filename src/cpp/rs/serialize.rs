use std::fmt::Display;

use cubing::{
    alg::Move,
    kpuzzle::{
        InvalidDefinitionError, KPatternData, KPuzzle, KPuzzleDefinition, KPuzzleOrbitName,
        KTransformation, KTransformationData,
    },
};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

const BLANK_LINE: &str = "";
const END: &str = "End";

pub fn sanitize(s: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"[^A-Za-z0-9]").unwrap();
    }
    RE.replace_all(s, "_").to_string() // TODO: Can we avoid calling `.to_string()`?
}

struct LiteStringBuilder {
    s: String, // TODO: use an actual builder instead of updating this in place.
    has_first_line: bool,
}

impl LiteStringBuilder {
    pub fn new() -> LiteStringBuilder {
        LiteStringBuilder {
            s: "".to_owned(),
            has_first_line: false,
        }
    }

    pub fn push(&mut self, line: &str) {
        if self.has_first_line {
            self.s.push('\n')
        } else {
            self.has_first_line = true;
        }
        self.s.push_str(line)
    }

    pub fn push_vec<T: Display>(&mut self, vec: &[T]) {
        self.push(&serialize_vec(vec))
    }

    pub fn push_str_vec(&mut self, vec: &[String]) {
        self.push(&vec.join(" "))
    }

    pub fn build(self) -> String {
        self.s
    }
}

fn serialize_vec<T: Display>(vec: &[T]) -> String {
    vec.iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

fn serialize_move_transformation(kpuzzle: &KPuzzle, r#move: &Move, t: &KTransformation) -> String {
    let mut builder = LiteStringBuilder::new();
    builder.push(&format!(
        "MoveTransformation {}",
        sanitize(&r#move.to_string())
    ));
    // outputLines.push(`MoveTransformation ${sanitize(name)}`);
    // TODO: use `orbit_ordering` if available? (TODO)
    for orbit_info in kpuzzle.orbit_info_iter() {
        builder.push(&sanitize(&orbit_info.name.to_string()));
        builder.push_vec(
            &unsafe {
                t.packed_orbit_data().byte_slice() /* TODO */
            }[orbit_info.pieces_or_permutations_offset
                ..(orbit_info.pieces_or_permutations_offset + (orbit_info.num_pieces as usize))],
        );
        builder.push_vec(
            &unsafe {
                t.packed_orbit_data().byte_slice() /* TODO */
            }[orbit_info.orientations_offset
                ..(orbit_info.orientations_offset + (orbit_info.num_pieces as usize))],
        );
    }
    builder.push(END);
    builder.push(BLANK_LINE);
    builder.build()
}

fn serialize_move_transformation_from_data(
    kpuzzle: &KPuzzle,
    r#move: &Move,
    t: &KTransformationData,
) -> String {
    let mut builder = LiteStringBuilder::new();
    builder.push(&format!(
        "MoveTransformation {}",
        sanitize(&r#move.to_string())
    ));
    // outputLines.push(`MoveTransformation ${sanitize(name)}`);
    // TODO: use `orbit_ordering` if available? (TODO)
    for orbit_definition in &kpuzzle.definition().orbits {
        builder.push(&sanitize(&orbit_definition.orbit_name.to_string()));
        builder.push_vec(&t[&orbit_definition.orbit_name].permutation);
        builder.push_vec(&t[&orbit_definition.orbit_name].orientation_delta);
    }
    builder.push(END);
    builder.push(BLANK_LINE);
    builder.build()
}

// TODO: don't accept an `Option` for `kpuzzle`?
fn serialize_kpattern_data(kpuzzle: Option<&KPuzzle>, t: &KPatternData) -> Result<String, String> {
    let mut builder = LiteStringBuilder::new();

    let mut process_orbit_name = |orbit_name: &KPuzzleOrbitName| {
        {
            let orbit_data = &t[orbit_name];
            builder.push(&sanitize(&orbit_name.to_string()));
            builder.push_vec(&orbit_data.pieces);

            let len = orbit_data.orientation.len();
            let mut str_vec = Vec::with_capacity(len);
            for i in 0..len {
                match orbit_data.orientation_mod.as_ref().map(|vec| vec[i]) {
                    None => str_vec.push(orbit_data.orientation[i].to_string()),
                    Some(0) => str_vec.push(orbit_data.orientation[i].to_string()),
                    Some(1) => str_vec.push("?".to_owned()), // TODO: assert that `orbit_data.orientation[i] == 0`?
                    Some(_) => {
                        return Err(
                            "Orientation mod entries other than 0 or 1 are not currently supported"
                                .to_owned(),
                        );
                    }
                }
            }
            builder.push_str_vec(&str_vec);
        };
        Ok(())
    };

    match kpuzzle {
        Some(kpuzzle) => {
            for orbit_definition in &kpuzzle.definition().orbits {
                process_orbit_name(&orbit_definition.orbit_name)?;
            }
        }
        None => {
            for orbit_name in t.keys() {
                process_orbit_name(orbit_name)?;
            }
        }
    }

    builder.push(END);
    builder.push(BLANK_LINE);
    Ok(builder.build())
}

fn include(options: &KPuzzleSerializationOptions, move_name: &Move) -> bool {
    match &options.move_subset {
        Some(move_subset) => move_subset.contains(move_name),
        None => true,
    }
}

pub fn serialize_scramble_kpattern_data(
    kpuzzle: Option<&KPuzzle>,
    name: &str,
    t: &KPatternData,
) -> Result<String, String> {
    let mut builder = LiteStringBuilder::new();
    builder.push(&format!("ScrambleState {}", name));
    builder.push(&serialize_kpattern_data(kpuzzle, t)?);
    Ok(builder.build())
}

pub struct KPuzzleSerializationOptions {
    pub move_subset: Option<Vec<Move>>,
    pub custom_start_pattern: Option<KPatternData>,
}

pub fn serialize_kpuzzle_definition(
    def: KPuzzleDefinition, // TODO: take reference (requires a change in `cubing.rs`?)
    options: Option<&KPuzzleSerializationOptions>,
) -> Result<String, InvalidDefinitionError> {
    let kpuzzle = KPuzzle::try_from(def)?;
    let def = kpuzzle.definition();

    let options = options.unwrap_or(&KPuzzleSerializationOptions {
        move_subset: None,
        custom_start_pattern: None,
    });
    let mut builder = LiteStringBuilder::new();

    builder.push(&format!("Name {}", sanitize(&def.name)));
    builder.push(BLANK_LINE);

    for orbit_definition in &def.orbits {
        builder.push(&format!(
            "Set {} {} {}",
            &sanitize(&orbit_definition.orbit_name.to_string()),
            orbit_definition.num_pieces,
            orbit_definition.num_orientations
        ));
    }
    builder.push(BLANK_LINE);

    builder.push("StartState");
    if let Some(start_pattern) = &options.custom_start_pattern {
        builder.push(&serialize_kpattern_data(Some(&kpuzzle), start_pattern)?);
    } else {
        builder.push(&serialize_kpattern_data(
            Some(&kpuzzle),
            &def.default_pattern,
        )?);
    }
    builder.push(BLANK_LINE);

    for (move_name, move_def) in &def.moves {
        if include(options, move_name) {
            builder.push(&serialize_move_transformation_from_data(
                &kpuzzle, move_name, move_def,
            ))
        }
    }

    if let Some(derived_moves) = &def.derived_moves.clone() {
        for (move_name, alg) in derived_moves {
            if include(options, move_name) {
                let transformation = match kpuzzle.transformation_from_alg(alg) {
                    Ok(transformation) => transformation,
                    Err(_) => {
                        return Err(InvalidDefinitionError {
                            description: format!(
                                "Derived move definition uses an invalid alg for: {}",
                                move_name
                            ),
                        })
                    }
                };
                builder.push(&serialize_move_transformation(
                    &kpuzzle,
                    move_name,
                    &transformation,
                ))
            }
        }
    };
    // let s = builder.build();
    // println!("{}", s);
    // Ok(s)
    Ok(builder.build())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrambleListEntry {
    pub pattern: KPatternData,
}

pub type ScrambleList = Vec<ScrambleListEntry>;

pub fn serialize_scramble_list(
    kpuzzle: Option<&KPuzzle>,
    scramble_list: &ScrambleList,
) -> Result<String, String> {
    let mut scramble_idx = 0;
    let scramble_strings: Result<Vec<String>, String> = scramble_list
        .iter()
        .map(|entry| {
            serialize_scramble_kpattern_data(
                kpuzzle,
                &format!("Scramble{}", {
                    scramble_idx += 1;
                    scramble_idx
                }),
                &entry.pattern,
            )
        })
        .collect();

    Ok(scramble_strings?.join("\n\n"))
}
