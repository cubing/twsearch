extern crate cubing;
use cubing::{
    kpuzzle::{KPuzzle, KPuzzleDefinition, KStateData, KTransformationData},
    parse_alg,
};

use lazy_static::lazy_static;
use regex::Regex;

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

fn serialize_move_transformation(name: &str, t: &KTransformationData) -> String {
    let mut builder = LiteStringBuilder::new();
    builder.push(&format!("MoveTransformation {}", sanitize(name)));
    // outputLines.push(`MoveTransformation ${sanitize(name)}`);
    for (orbit_name, orbit_data) in t {
        builder.push(&sanitize(orbit_name));
        builder.push_vec(&orbit_data.permutation);
        builder.push_vec(&orbit_data.orientation);
    }
    builder.push(END);
    builder.push(BLANK_LINE);
    builder.build()
}

fn serialize_state_data(t: &KStateData) -> String {
    let mut builder = LiteStringBuilder::new();
    for (orbit_name, orbit_data) in t {
        builder.push(&sanitize(orbit_name));
        builder.push_vec(&orbit_data.pieces);
        builder.push_vec(&orbit_data.orientation);
    }
    builder.push(END);
    builder.push(BLANK_LINE);
    builder.build()
}

pub fn serialize_scramble_state_data(name: &str, t: &KStateData) -> String {
    let mut builder = LiteStringBuilder::new();
    builder.push(&format!("ScrambleState {}", name));
    builder.push(&serialize_state_data(t));
    builder.build()
}

pub struct KPuzzleSerializationOptions {
    move_subset: Option<Vec<String>>,
    custom_start_state: Option<KStateData>,
}

pub fn serialize_kpuzzle_definition(
    def: KPuzzleDefinition, // TODO: take reference (requires a change in `cubing.rs`?)
    options: Option<KPuzzleSerializationOptions>,
) -> Result<String, String> {
    let options = options.unwrap_or(KPuzzleSerializationOptions {
        move_subset: None,
        custom_start_state: None,
    });
    let mut builder = LiteStringBuilder::new();

    builder.push(&format!("Name {}", sanitize(&def.name)));
    builder.push(BLANK_LINE);

    for (orbit_name, orbit_info) in &def.orbits {
        builder.push(&format!(
            "Set {} {} {}",
            &sanitize(orbit_name),
            orbit_info.num_pieces,
            orbit_info.num_orientations
        ));
    }
    builder.push(BLANK_LINE);

    builder.push("StartState");
    if let Some(start_state) = &options.custom_start_state {
        builder.push(&serialize_state_data(start_state));
    } else {
        builder.push(&serialize_state_data(&def.start_state_data));
    }
    builder.push(BLANK_LINE);

    for (move_name, move_def) in &def.moves {
        builder.push(&serialize_move_transformation(move_name, move_def))
    }

    if let Some(experimental_derived_moves) = &def.experimental_derived_moves.clone() {
        let kpuzzle = KPuzzle::new(def);
        for (move_name, move_alg_def) in experimental_derived_moves {
            let include = match &options.move_subset {
                Some(move_subset) => move_subset.contains(move_name),
                None => true,
            };
            if include {
                let alg = match parse_alg!(move_alg_def) {
                    Ok(alg) => alg,
                    Err(e) => return Err(e),
                };
                let transformation = match kpuzzle.transformation_from_alg(&alg) {
                    Ok(transformation) => transformation,
                    Err(e) => return Err(e),
                };
                builder.push(&serialize_move_transformation(
                    move_name,
                    &transformation.transformation_data,
                ))
            }
        }
    };
    Ok(builder.build())
}
