use cubing::alg::{parse_alg, parse_move, Alg, Move};
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::{
    _internal::{AlwaysValid, CheckPattern},
    scramble::{
        randomize::{basic_parity, PieceZeroConstraint},
        scramble_search::FilteredSearch,
    },
};

use super::{
    super::{
        randomize::{
            randomize_orbit_naïve, OrbitOrientationConstraint, OrbitPermutationConstraint,
        },
        scramble_search::{generators_from_vec_str, simple_filtered_search},
    },
    definitions::{square1_cube_shape_kpattern, square1_unbandaged_kpuzzle},
    mask_pattern::mask,
};

#[derive(PartialEq, Eq)]
enum WedgeType {
    CornerLower,
    CornerUpper,
    Edge,
}

const WEDGE_TYPE_LOOKUP: [WedgeType; 24] = [
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
    WedgeType::Edge,
    WedgeType::CornerLower,
    WedgeType::CornerUpper,
];

struct Phase1Checker;

const SLOTS_THAT_ARE_AFTER_SLICES: [u8; 4] = [0, 6, 12, 18];

impl CheckPattern for Phase1Checker {
    fn is_valid(pattern: &cubing::kpuzzle::KPattern) -> bool {
        let orbit_info = &pattern.kpuzzle().data.ordered_orbit_info[0];
        assert_eq!(orbit_info.name.0, "WEDGES");

        for slot in SLOTS_THAT_ARE_AFTER_SLICES {
            let value = unsafe {
                pattern
                    .packed_orbit_data()
                    .get_raw_piece_or_permutation_value(orbit_info, slot)
            };

            // TODO: consider removing this lookup. We know that the wedge values are only 0, 1, or
            // 2 during this phase.
            if WEDGE_TYPE_LOOKUP[value as usize] == WedgeType::CornerUpper {
                return false;
            }
        }

        true
    }
}

struct Phase2Checker;

impl CheckPattern for Phase2Checker {
    fn is_valid(pattern: &cubing::kpuzzle::KPattern) -> bool {
        let orbit_info = &pattern.kpuzzle().data.ordered_orbit_info[0];
        assert_eq!(orbit_info.name.0, "WEDGES");

        for slot in [0, 1, 2, 12, 13, 14] {
            let value = unsafe {
                pattern
                    .packed_orbit_data()
                    .get_raw_piece_or_permutation_value(orbit_info, slot)
            };
            let wedge_type = &WEDGE_TYPE_LOOKUP[value as usize];

            if *wedge_type == WedgeType::CornerUpper && (slot == 0 || slot == 12) {
                // We can't slice.
                return false;
            }

            for slot_offset in [3, 6, 9] {
                let offset_value = unsafe {
                    pattern
                        .packed_orbit_data()
                        .get_raw_piece_or_permutation_value(orbit_info, slot + slot_offset)
                };
                let offset_wedge_type = &WEDGE_TYPE_LOOKUP[offset_value as usize];

                if wedge_type != offset_wedge_type {
                    return false;
                }
            }
        }

        true
    }
}

pub fn scramble_square1() -> Alg {
    let kpuzzle = square1_unbandaged_kpuzzle();
    loop {
        let mut scramble_pattern = kpuzzle.default_pattern();

        // <<< let mut rng = thread_rng();
        // <<<
        // <<< let mut deep_wedges = vec![
        // <<<     vec![0, 1],
        // <<<     vec![2],
        // <<<     vec![3, 4],
        // <<<     vec![5],
        // <<<     vec![6, 7],
        // <<<     vec![8],
        // <<<     vec![9, 10],
        // <<<     vec![11],
        // <<<     vec![12, 13],
        // <<<     vec![14],
        // <<<     vec![15, 16],
        // <<<     vec![17],
        // <<<     vec![18, 19],
        // <<<     vec![20],
        // <<<     vec![21, 22],
        // <<<     vec![23],
        // <<< ];
        // <<< deep_wedges.shuffle(&mut rng);
        // <<<
        // <<< let orbit_info = &scramble_pattern.kpuzzle().clone().data.ordered_orbit_info[0];
        // <<< assert_eq!(orbit_info.name.0, "WEDGES");
        // <<< for (i, value) in deep_wedges.into_iter().flatten().enumerate() {
        // <<<     unsafe {
        // <<<         scramble_pattern.packed_orbit_data_mut().set_raw_piece_or_permutation_value(orbit_info, i as u8, value);
        // <<<     }
        // <<< }
        // <<<
        // <<< randomize_orbit_naïve(
        // <<<     &mut scramble_pattern,
        // <<<     1,
        // <<<     "EQUATOR",
        // <<<     OrbitPermutationConstraint::AnyPermutation,
        // <<<     OrbitOrientationConstraint::AnySum,
        // <<<     PieceZeroConstraint::KeepSolved,
        // <<< );

        // <<< let scramble_pattern = scramble_pattern.apply_alg(&parse_alg!("U_SQ_3 D_SQ_ _SLASH_")).unwrap();
        // <<< let scramble_pattern = scramble_pattern.apply_alg(&parse_alg!("_SLASH_ U_SQ_3 D_SQ_ _SLASH_")).unwrap();

        // <<< let scramble_pattern = scramble_pattern.apply_alg(&parse_alg!("U_SQ_2' _SLASH_ U_SQ_5 D_SQ_2 _SLASH_ U_SQ_4 D_SQ_2' _SLASH_")).unwrap();
        // <<< let scramble_pattern = scramble_pattern.apply_alg(&parse_alg!("U_SQ_3 D_SQ_2 _SLASH_ D_SQ_")).unwrap();
        // <<< let scramble_pattern = scramble_pattern.apply_alg(&parse_alg!("(U_SQ_5' D_SQ_0) / (U_SQ_0 D_SQ_3) / (U_SQ_3 D_SQ_0) / (U_SQ_' D_SQ_4') / (U_SQ_4 D_SQ_2') / (U_SQ_5 D_SQ_4') / (U_SQ_2' D_SQ_0) / (U_SQ_0 D_SQ_3') / (U_SQ_' D_SQ_0) / (U_SQ_3 D_SQ_4') / (U_SQ_4 D_SQ_2') /")).unwrap();
        // <<< let scramble_pattern = scramble_pattern.apply_alg(&parse_alg!("(U_SQ_4 D_SQ_3) / (U_SQ_' D_SQ_') / (U_SQ_0 D_SQ_3') / (U_SQ_3' D_SQ_3') / (U_SQ_ D_SQ_2') / (U_SQ_3' D_SQ_4') / (U_SQ_3 D_SQ_0) / (U_SQ_4' D_SQ_5') / (U_SQ_3' D_SQ_0) / (U_SQ_4' D_SQ_0) / (U_SQ_0 D_SQ_2')")).unwrap();
        // <<< let scramble_pattern = scramble_pattern.apply_alg(&parse_alg!("(U_SQ_0 D_SQ_5) / (U_SQ_ D_SQ_5') / (U_SQ_0 D_SQ_3') / (U_SQ_3 D_SQ_0) / (U_SQ_4' D_SQ_') / (U_SQ_3' D_SQ_3') / (U_SQ_0 D_SQ_5') / (U_SQ_3' D_SQ_3') / (U_SQ_4' D_SQ_0) / (U_SQ_0 D_SQ_5') / (U_SQ_4 D_SQ_3') / (U_SQ_0 D_SQ_2') /")).unwrap();
        let scramble_pattern = scramble_pattern.apply_alg(&parse_alg!("
(U_SQ_0 D_SQ_') / (U_SQ_0 D_SQ_3') / (U_SQ_0 D_SQ_3') / (U_SQ_4 D_SQ_5') / (U_SQ_5 D_SQ_4') / (U_SQ_5' D_SQ_2') / (U_SQ_5 D_SQ_0) / (U_SQ_3' D_SQ_3') / (U_SQ_2' D_SQ_0) / (U_SQ_6 D_SQ_') / (U_SQ_5' D_SQ_4') / (U_SQ_2' D_SQ_0)
")).unwrap();

        let phase1_start_pattern = mask(&scramble_pattern, square1_cube_shape_kpattern()).unwrap();

        if !Phase1Checker::is_valid(&phase1_start_pattern) {
            println!("discarding invalid scramble"); //<<<
            continue;
        }

        println!(
            "{}",
            serde_json::to_string_pretty(&phase1_start_pattern.to_data()).unwrap()
        );

        println!(
            "{}",
            serde_json::to_string_pretty(&square1_cube_shape_kpattern().to_data()).unwrap()
        );

        let generators = generators_from_vec_str(vec!["U_SQ_", "D_SQ_", "_SLASH_"]); // TODO: cache
                                                                                     // <<< if let Some(solution) = simple_filtered_search(&phase1_start_pattern, generators, 11, None) {

        let mut phase1_filtered_search = FilteredSearch::<Phase1Checker>::new(
            kpuzzle,
            generators.clone(),
            None,
            square1_cube_shape_kpattern().clone(),
        );

        let mut phase1_solution = phase1_filtered_search
            .generate_scramble(&phase1_start_pattern, None)
            .invert();

        // <<< return Alg {
        // <<<     nodes: phase1_solution.nodes,
        // <<< }; //<<<

        // <<< while let Some(cubing::alg::AlgNode::MoveNode(r#move)) = phase1_solution.nodes.last() {
        // <<<     if r#move == &parse_move!("_SLASH_'")
        // <<<     // TODO: redundant parsing
        // <<<     {
        // <<<         break;
        // <<<     }
        // <<<     phase1_solution.nodes.pop();
        // <<< }
        // <<< dbg!(&parse_move!("/")); //<<<

        let phase2_start_pattern = scramble_pattern.apply_alg(&phase1_solution).unwrap();

        // dbg!(basic_parity(phase2_start_pattern.));

        let mut phase2_filtered_search = FilteredSearch::<Phase2Checker>::new(
            kpuzzle,
            generators,
            None,
            kpuzzle.default_pattern(),
        );
        let mut phase2_solution = phase2_filtered_search
            .generate_scramble(&phase2_start_pattern, None)
            .invert();

        let mut nodes = phase1_solution.nodes;
        nodes.append(&mut phase2_solution.nodes);
        // <<< return Alg { nodes }.invert()
        return Alg { nodes }; // because slash' is not a valid move we can print

        // if let Some(solution) = filtered_search.generate_scramble(&phase1_start_pattern, 0) {
        //     //<<<
        //     return solution.invert();
        // }
    }
}
