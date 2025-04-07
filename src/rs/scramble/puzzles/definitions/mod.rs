use cubing::{
    kpuzzle::{kpattern_from_json_file, kpuzzle_from_json_file},
    puzzles::{cube2x2x2_kpuzzle, cube3x3x3_kpuzzle},
};

/******************************** 3×3×3 ********************************/

kpattern_from_json_file!(pub(crate), cube3x3x3_orientation_canonicalization, "./3x3x3/3x3x3.orientation-canonicalization-pattern.json", cube3x3x3_kpuzzle());
kpattern_from_json_file!(pub(crate), cube3x3x3_g1_target, "./3x3x3/3x3x3-G1.target-pattern.json", cube3x3x3_kpuzzle());

/******************************** 2×2×2 ********************************/

// TODO: if we were sneaky, we could reuse the 3×3×3 or 4×4×4 definition for this (since 2×2×2 has a subset of those orbits with the same semantics). 🤣
kpattern_from_json_file!(pub(crate), cube2x2x2_orientation_canonicalization, "./2x2x2/2x2x2.orientation-canonicalization-pattern.json", cube2x2x2_kpuzzle());

/******************************** 4×4×4 ********************************/

// TODO: Handle a default pattern with indistinguishable pieces on `apply_mask(…)`.
kpuzzle_from_json_file!(pub(crate), cube4x4x4, "./4x4x4/4x4x4.kpuzzle.json");

kpattern_from_json_file!(pub(crate), cube4x4x4_orientation_canonicalization, "./4x4x4/4x4x4.orientation-canonicalization-pattern.json", cube4x4x4_kpuzzle());
// TODO: This should be in `4x4x4.kpuzzle.json` once we can support it.
kpattern_from_json_file!(pub(crate), cube4x4x4_solved, "./4x4x4/4x4x4.solved-pattern.json", cube4x4x4_kpuzzle());

kpattern_from_json_file!(pub(crate), cube4x4x4_phase1_target, "./4x4x4/4x4x4-phase1.target.json", cube4x4x4_kpuzzle());
// Note that this does not track wing separation.
kpuzzle_from_json_file!(pub(crate), cube4x4x4_phase2_search, "./4x4x4/4x4x4-phase2-search.kpuzzle.json");
kpuzzle_from_json_file!(pub(crate), cube4x4x4_phase3_search, "./4x4x4/4x4x4-phase3-search.kpuzzle.json");

/******************************** Big cubes ********************************/

kpuzzle_from_json_file!(pub(crate), cube5x5x5, "./big_cubes/5x5x5.kpuzzle.json");
kpuzzle_from_json_file!(pub(crate), cube6x6x6, "./big_cubes/6x6x6.kpuzzle.json");
kpuzzle_from_json_file!(pub(crate), cube7x7x7, "./big_cubes/7x7x7.kpuzzle.json");

/******************************** Clock ********************************/

kpuzzle_from_json_file!(pub(crate), clock, "./clock/clock.kpuzzle.json");
kpattern_from_json_file!(pub(crate), clock_orientation_canonicalization, "./clock/clock.orientation-canonicalization-pattern.json", clock_kpuzzle());

/******************************** Megaminx ********************************/

// TODO: Rebase this definition to make it smaller.
kpuzzle_from_json_file!(pub(crate), megaminx, "./megaminx/megaminx.kpuzzle.json");
kpattern_from_json_file!(pub(crate), megaminx_phase1_target, "./megaminx/megaminx-phase1.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase2_target, "./megaminx/megaminx-phase2.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase3_target, "./megaminx/megaminx-phase3.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase4_target, "./megaminx/megaminx-phase4.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase5_target, "./megaminx/megaminx-phase5.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase6_target, "./megaminx/megaminx-phase6.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase7_target, "./megaminx/megaminx-phase7.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase8_target, "./megaminx/megaminx-phase8.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase9_target, "./megaminx/megaminx-phase9.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase10_target, "./megaminx/megaminx-phase10.target-pattern.json", megaminx_kpuzzle());
kpattern_from_json_file!(pub(crate), megaminx_phase11_target, "./megaminx/megaminx-phase11.target-pattern.json", megaminx_kpuzzle());

/******************************** Pyraminx ********************************/

kpuzzle_from_json_file!(pub(crate), pyraminx, "./pyraminx/pyraminx.kpuzzle.json");
kpattern_from_json_file!(pub(crate), pyraminx_ignoring_tips, "./pyraminx/pyraminx.ignoring-tips-pattern.json", pyraminx_kpuzzle());

/******************************** Square-1 ********************************/

kpuzzle_from_json_file!(pub(crate), square1_unbandaged, "./square1/square1-unbandaged.kpuzzle.json");
kpattern_from_json_file!(pub(crate), square1_square_square_shape, "./square1/square1-square-square-shape.target-pattern.json", square1_unbandaged_kpuzzle());
kpattern_from_json_file!(pub(crate), square1_shape, "./square1/square1-shape.mask-pattern.json", square1_unbandaged_kpuzzle());
kpattern_from_json_file!(pub(crate), square1_edges, "./square1/square1-edges.mask-pattern.json", square1_unbandaged_kpuzzle());
kpattern_from_json_file!(pub(crate), square1_corners, "./square1/square1-corners.mask-pattern.json", square1_unbandaged_kpuzzle());
kpuzzle_from_json_file!(pub(crate), square0_equatorless, "./square1/square0-equatorless.kpuzzle.json");

/******************************** Other WCA puzzles ********************************/

// kpuzzle_from_json_file!(pub(crate), tetraminx, "tetraminx.kpuzzle.json");
kpuzzle_from_json_file!(pub(crate), skewb_fixed_corner_with_co_tweaks, "./other_wca/skewb-fixed-corner-with-co-tweaks.kpuzzle.json");

/******************************** Other puzzles ********************************/

kpuzzle_from_json_file!(pub(crate), baby_fto, "./other/baby_fto.kpuzzle.json");
