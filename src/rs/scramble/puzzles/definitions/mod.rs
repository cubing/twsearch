use std::sync::OnceLock;

use cubing::kpuzzle::{KPattern, KPuzzle, KPuzzleDefinition};

static CUBE3X3X3_CENTERLESS_KPUZZLE_CELL: OnceLock<KPuzzle> = OnceLock::new();
// TODO: avoid re-parsing every time
pub(crate) fn cube3x3x3_centerless_kpuzzle() -> KPuzzle {
    CUBE3X3X3_CENTERLESS_KPUZZLE_CELL
        .get_or_init(|| {
            let json_bytes = include_bytes!("3x3x3-centerless.kpuzzle.json");
            let def: KPuzzleDefinition = serde_json::from_slice(json_bytes).unwrap();
            let kpuzzle: KPuzzle = def.try_into().unwrap();
            kpuzzle
        })
        .clone()
}

static CUBE3X3X3_G1_CENTERLESS_PATTERN_CELL: OnceLock<KPattern> = OnceLock::new();
pub(crate) fn cube3x3x3_g1_target_pattern() -> KPattern {
    CUBE3X3X3_G1_CENTERLESS_PATTERN_CELL
        .get_or_init(|| {
            let kpuzzle = cube3x3x3_centerless_kpuzzle();
            KPattern::try_from_json(
                &kpuzzle,
                include_bytes!("3x3x3-G1-centerless.target-pattern.json"),
            )
            .unwrap()
        })
        .clone()
}

static CUBE5X5X5_KPUZZLE_CELL: OnceLock<KPuzzle> = OnceLock::new();
// TODO: Require the WASM caller to pass in the JSON, to save on build size.
pub(crate) fn cube5x5x5_kpuzzle() -> KPuzzle {
    CUBE5X5X5_KPUZZLE_CELL
        .get_or_init(|| {
            let json_bytes = include_bytes!("5x5x5.kpuzzle.json");
            let def: KPuzzleDefinition = serde_json::from_slice(json_bytes).unwrap();
            let kpuzzle: KPuzzle = def.try_into().unwrap();
            kpuzzle
        })
        .clone()
}

static CUBE6X6X6_KPUZZLE_CELL: OnceLock<KPuzzle> = OnceLock::new();
// TODO: Require the WASM caller to pass in the JSON, to save on build size.
pub(crate) fn cube6x6x6_kpuzzle() -> KPuzzle {
    CUBE6X6X6_KPUZZLE_CELL
        .get_or_init(|| {
            let json_bytes = include_bytes!("6x6x6.kpuzzle.json");
            let def: KPuzzleDefinition = serde_json::from_slice(json_bytes).unwrap();
            let kpuzzle: KPuzzle = def.try_into().unwrap();
            kpuzzle
        })
        .clone()
}

static CUBE7X7X7_KPUZZLE_CELL: OnceLock<KPuzzle> = OnceLock::new();
// TODO: Require the WASM caller to pass in the JSON, to save on build size.
pub(crate) fn cube7x7x7_kpuzzle() -> KPuzzle {
    CUBE7X7X7_KPUZZLE_CELL
        .get_or_init(|| {
            let json_bytes = include_bytes!("7x7x7.kpuzzle.json");
            let def: KPuzzleDefinition = serde_json::from_slice(json_bytes).unwrap();
            let kpuzzle: KPuzzle = def.try_into().unwrap();
            kpuzzle
        })
        .clone()
}

static TETRAMINX_KPUZZLE_CELL: OnceLock<KPuzzle> = OnceLock::new();
pub(crate) fn tetraminx_kpuzzle() -> KPuzzle {
    TETRAMINX_KPUZZLE_CELL
        .get_or_init(|| {
            let json_bytes = include_bytes!("tetraminx.kpuzzle.json");
            let def: KPuzzleDefinition = serde_json::from_slice(json_bytes).unwrap();
            let kpuzzle: KPuzzle = def.try_into().unwrap();
            kpuzzle
        })
        .clone()
}
