use cubing::kpuzzle::{KPuzzle, KPuzzleDefinition};

// TODO: avoid re-parsing every time
pub(crate) fn tetraminx_kpuzzle() -> KPuzzle {
    let json_bytes = include_bytes!("tetraminx.kpuzzle.json");
    let def: KPuzzleDefinition = serde_json::from_slice(json_bytes).unwrap();
    def.try_into().unwrap()
}
