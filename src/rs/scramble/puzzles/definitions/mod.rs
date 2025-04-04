use cubing::{
    kpuzzle::{kpattern_from_json_file, kpuzzle_from_json_file},
    puzzles::cube3x3x3_kpuzzle,
};

kpuzzle_from_json_file!(pub(crate), cube3x3x3_centerless, "3x3x3-centerless.kpuzzle.json");
kpattern_from_json_file!(pub(crate), cube3x3x3_centerless_g1_target, "3x3x3-G1-centerless.target-pattern.json", cube3x3x3_centerless_kpuzzle());

kpattern_from_json_file!(pub(crate), cube3x3x3_g1_target, "3x3x3-G1.target-pattern.json", cube3x3x3_kpuzzle());

// TODO: Handle a default pattern with indistinguishable pieces on `apply_mask(…)`.
kpuzzle_from_json_file!(pub(crate), cube4x4x4, "4x4x4.kpuzzle.json");
kpattern_from_json_file!(pub(crate), cube4x4x4_phase1_target, "4x4x4-phase1.target.json", cube4x4x4_kpuzzle());
// Note that this does not track wing separation.
kpuzzle_from_json_file!(pub(crate), cube4x4x4_phase2_search, "4x4x4-phase2-search.kpuzzle.json");
kpuzzle_from_json_file!(pub(crate), cube4x4x4_phase3_search, "4x4x4-phase3-search.kpuzzle.json");

kpuzzle_from_json_file!(pub(crate), cube5x5x5, "5x5x5.kpuzzle.json");
kpuzzle_from_json_file!(pub(crate), cube6x6x6, "6x6x6.kpuzzle.json");
kpuzzle_from_json_file!(pub(crate), cube7x7x7, "7x7x7.kpuzzle.json");
kpuzzle_from_json_file!(pub(crate), skewb_fixed_corner_with_co_tweaks, "skewb-fixed-corner-with-co-tweaks.kpuzzle.json");

// kpuzzle_from_json_file!(pub(crate), tetraminx, "tetraminx.kpuzzle.json");
kpuzzle_from_json_file!(pub(crate), pyraminx, "pyraminx.kpuzzle.json");

kpuzzle_from_json_file!(pub(crate), square1_unbandaged, "square1-unbandaged.kpuzzle.json");
kpattern_from_json_file!(pub(crate), square1_square_square_shape, "square1-square-square-shape.target-pattern.json", square1_unbandaged_kpuzzle());
kpattern_from_json_file!(pub(crate), square1_shape, "square1-shape.mask-pattern.json", square1_unbandaged_kpuzzle());
kpattern_from_json_file!(pub(crate), square1_edges, "square1-edges.mask-pattern.json", square1_unbandaged_kpuzzle());
kpattern_from_json_file!(pub(crate), square1_corners, "square1-corners.mask-pattern.json", square1_unbandaged_kpuzzle());
kpuzzle_from_json_file!(pub(crate), square0_equatorless, "square0-equatorless.kpuzzle.json");

kpuzzle_from_json_file!(pub(crate), baby_fto, "baby_fto.kpuzzle.json");

// TODO: Rebase this definition to make it smaller.
kpuzzle_from_json_file!(pub(crate), megaminx, "megaminx.kpuzzle.json");
kpattern_from_json_file!(pub(crate), megaminx_phase1_target, "megaminx-phase1.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase2_target, "megaminx-phase2.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase3_target, "megaminx-phase3.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase4_target, "megaminx-phase4.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase5_target, "megaminx-phase5.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase6_target, "megaminx-phase6.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase7_target, "megaminx-phase7.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase8_target, "megaminx-phase8.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase9_target, "megaminx-phase9.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase10_target, "megaminx-phase10.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase11_target, "megaminx-phase11.target-pattern.json", megaminx_kpuzzle());
