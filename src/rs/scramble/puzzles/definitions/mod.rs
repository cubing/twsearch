use cubing::kpuzzle::{kpattern_from_json_file, kpuzzle_from_json_file};

kpuzzle_from_json_file!(pub(crate), cube3x3x3_centerless, "3x3x3-centerless.kpuzzle.json");
kpattern_from_json_file!(pub(crate), cube3x3x3_centerless_g1_target, "3x3x3-G1-centerless.target-pattern.json", cube3x3x3_centerless_kpuzzle());

kpuzzle_from_json_file!(pub(crate), cube4x4x4, "4x4x4-Speffz.kpuzzle.json");
kpattern_from_json_file!(pub(crate), cube4x4x4_phase1_target, "4x4x4-Phase1.target.json", cube4x4x4_kpuzzle());

kpuzzle_from_json_file!(pub(crate), cube4x4x4_with_wing_parity, "4x4x4-Speffz-with-wing-parity.kpuzzle.json");
kpattern_from_json_file!(pub(crate), cube4x4x4_phase2_target, "4x4x4-Phase2.target.json", cube4x4x4_with_wing_parity_kpuzzle());

kpuzzle_from_json_file!(pub(crate), cube5x5x5, "5x5x5.kpuzzle.json");
kpuzzle_from_json_file!(pub(crate), cube6x6x6, "6x6x6.kpuzzle.json");
kpuzzle_from_json_file!(pub(crate), cube7x7x7, "7x7x7.kpuzzle.json");
kpuzzle_from_json_file!(pub(crate), tetraminx, "tetraminx.kpuzzle.json");
