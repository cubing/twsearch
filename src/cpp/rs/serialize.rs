extern crate cubing;

use cubing::{
    alg::Move,
    kpuzzle::{
        InvalidDefinitionError, KPatternData, KPuzzle, KPuzzleDefinition, KTransformationData,
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

    pub fn push_vec(&mut self, vec: &[usize]) {
        self.push(&serialize_usize_vec(vec))
    }

    pub fn push_str_vec(&mut self, vec: &[String]) {
        self.push(&vec.join(" "))
    }

    pub fn build(self) -> String {
        self.s
    }
}

fn serialize_usize_vec(vec: &[usize]) -> String {
    vec.iter()
        .map(|v| v.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

fn serialize_move_transformation(r#move: &Move, t: &KTransformationData) -> String {
    let mut builder = LiteStringBuilder::new();
    builder.push(&format!(
        "MoveTransformation {}",
        sanitize(&r#move.to_string())
    ));
    // outputLines.push(`MoveTransformation ${sanitize(name)}`);
    // TODO: use `orbit_ordering` if available?
    for (orbit_name, orbit_data) in t {
        builder.push(&sanitize(&orbit_name.to_string()));
        builder.push_vec(&orbit_data.permutation);
        builder.push_vec(&orbit_data.orientation_delta);
    }
    builder.push(END);
    builder.push(BLANK_LINE);
    builder.build()
}

fn serialize_state_data(t: &KPatternData) -> Result<String, String> {
    let mut builder = LiteStringBuilder::new();
    // TODO: use `orbit_ordering` if available?
    for (orbit_name, orbit_data) in t {
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

pub fn serialize_scramble_state_data(name: &str, t: &KPatternData) -> Result<String, String> {
    let mut builder = LiteStringBuilder::new();
    builder.push(&format!("ScrambleState {}", name));
    builder.push(&serialize_state_data(t)?);
    Ok(builder.build())
}

pub struct KPuzzleSerializationOptions {
    pub move_subset: Option<Vec<Move>>,
    pub custom_start_state: Option<KPatternData>,
}

pub fn serialize_kpuzzle_definition(
    def: KPuzzleDefinition, // TODO: take reference (requires a change in `cubing.rs`?)
    options: Option<&KPuzzleSerializationOptions>,
) -> Result<String, InvalidDefinitionError> {
    let options = options.unwrap_or(&KPuzzleSerializationOptions {
        move_subset: None,
        custom_start_state: None,
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
    if let Some(start_state) = &options.custom_start_state {
        builder.push(&serialize_state_data(start_state)?);
    } else {
        builder.push(&serialize_state_data(&def.default_pattern)?);
    }
    builder.push(BLANK_LINE);

    for (move_name, move_def) in &def.moves {
        if include(options, move_name) {
            builder.push(&serialize_move_transformation(move_name, move_def))
        }
    }

    if let Some(experimental_derived_moves) = &def.experimental_derived_moves.clone() {
        let kpuzzle = KPuzzle::try_new(def)?;
        for (move_name, alg) in experimental_derived_moves {
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
                    move_name,
                    &transformation.ktransformation_data,
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

pub fn serialize_scramble_list(scramble_list: &ScrambleList) -> Result<String, String> {
    let mut scramble_idx = 0;
    let scramble_strings: Result<Vec<String>, String> = scramble_list
        .iter()
        .map(|entry| {
            serialize_scramble_state_data(
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
