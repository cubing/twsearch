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

static CUBE4X4X4_KPUZZLE_CELL: OnceLock<KPuzzle> = OnceLock::new();
// TODO: Require the WASM caller to pass in the JSON, to save on build size.
// TODO: Replace with non-super definition once we can handle that in the rest fo the code.
pub(crate) fn cube4x4x4_packed_kpuzzle() -> KPuzzle {
    CUBE4X4X4_KPUZZLE_CELL
        .get_or_init(|| {
            let json_bytes = include_bytes!("4x4x4-Speffz.kpuzzle.json");
            let def: KPuzzleDefinition = serde_json::from_slice(json_bytes).unwrap();
            let kpuzzle: KPuzzle = def.try_into().unwrap();
            KPuzzle::try_from(kpuzzle).unwrap()
        })
        .clone()
}

static CUBE4X4X4_PHASE1_PATTERN_CELL: OnceLock<KPattern> = OnceLock::new();
// TODO: Require the WASM caller to pass in the JSON, to save on build size.
// TODO: ignore edges and corners
pub(crate) fn cube4x4x4_phase1_target_pattern() -> KPattern {
    CUBE4X4X4_PHASE1_PATTERN_CELL
        .get_or_init(|| {
            let packed_kpuzzle = cube4x4x4_packed_kpuzzle();
            KPattern::try_from_json(&packed_kpuzzle, include_bytes!("4x4x4-Phase1.target.json"))
                .unwrap()
        })
        .clone()
}

static CUBE4X4X4_WITH_WING_PARITY_KPUZZLE_CELL: OnceLock<KPuzzle> = OnceLock::new();
// TODO: Require the WASM caller to pass in the JSON, to save on build size.
// TODO: Replace with non-super definition once we can handle that in the rest fo the code.
pub(crate) fn cube4x4x4_with_wing_parity_packed_kpuzzle() -> KPuzzle {
    CUBE4X4X4_WITH_WING_PARITY_KPUZZLE_CELL
        .get_or_init(|| {
            let json_bytes = include_bytes!("4x4x4-Speffz-with-wing-parity.kpuzzle.json");
            let def: KPuzzleDefinition = serde_json::from_slice(json_bytes).unwrap();
            let kpuzzle: KPuzzle = def.try_into().unwrap();
            KPuzzle::try_from(kpuzzle).unwrap()
        })
        .clone()
}

static CUBE4X4X4_PHASE2_PATTERN_CELL: OnceLock<KPattern> = OnceLock::new();
// TODO: Require the WASM caller to pass in the JSON, to save on build size.
// TODO: add a way to allow any of the 12 possibilities instead of requiring L and R to both be solved.
// TODO: ignore edges and corners
pub(crate) fn cube4x4x4_phase2_target_pattern() -> KPattern {
    CUBE4X4X4_PHASE2_PATTERN_CELL
        .get_or_init(|| {
            let packed_kpuzzle = cube4x4x4_with_wing_parity_packed_kpuzzle();
            KPattern::try_from_json(&packed_kpuzzle, include_bytes!("4x4x4-Phase2.target.json"))
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
