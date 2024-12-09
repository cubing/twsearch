use cubing::kpuzzle::{kpattern_from_json_file, kpuzzle_from_json_file};

kpuzzle_from_json_file!(pub(crate), cube3x3x3_centerless, "3x3x3-centerless.kpuzzle.json");
kpattern_from_json_file!(pub(crate), cube3x3x3_centerless_g1_target, "3x3x3-G1-centerless.target-pattern.json", cube3x3x3_centerless_kpuzzle());

kpuzzle_from_json_file!(pub(crate), cube5x5x5, "5x5x5.kpuzzle.json");
kpuzzle_from_json_file!(pub(crate), cube6x6x6, "6x6x6.kpuzzle.json");
kpuzzle_from_json_file!(pub(crate), cube7x7x7, "7x7x7.kpuzzle.json");
kpuzzle_from_json_file!(pub(crate), tetraminx, "tetraminx.kpuzzle.json");
kpuzzle_from_json_file!(pub(crate), skewb_fixed_corner_with_co_tweaks, "skewb-fixed-corner-with-co-tweaks.kpuzzle.json");

kpuzzle_from_json_file!(pub(crate), square1_unbandaged, "square1-unbandaged.kpuzzle.json");
kpattern_from_json_file!(pub(crate), square1_square_square_shape, "square1-square-square-shape.target-pattern.json", square1_unbandaged_kpuzzle());
kpattern_from_json_file!(pub(crate), square1_edges, "square1-edges.mask-pattern.json", square1_unbandaged_kpuzzle());
kpattern_from_json_file!(pub(crate), square1_corners, "square1-corners.mask-pattern.json", square1_unbandaged_kpuzzle());
kpattern_from_json_file!(pub(crate), square1_equator, "square1-equator.mask-pattern.json", square1_unbandaged_kpuzzle());
