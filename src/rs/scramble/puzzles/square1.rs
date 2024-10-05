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

const SLOTS_THAT_ARE_BEFORE_SLICES: [u8; 4] = [5, 11, 17, 23];
// const LOWER_CORNER_CUBE_SLOTS: [u8; 4] = [0, 3, 6, 9, 13, 16, 19, 22];

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

struct Square1SlicableChecker;

impl CheckPattern for Square1SlicableChecker {
    fn is_valid(pattern: &cubing::kpuzzle::KPattern) -> bool {
        let orbit_info = &pattern.kpuzzle().data.ordered_orbit_info[0];
        assert_eq!(orbit_info.name.0, "WEDGES");

        for slot in SLOTS_THAT_ARE_BEFORE_SLICES {
            let value = unsafe {
                pattern
                    .packed_orbit_data()
                    .get_raw_piece_or_permutation_value(orbit_info, slot)
            };

            if WEDGE_TYPE_LOOKUP[value as usize] == WedgeType::CornerLower {
                return false;
            }
        }

        true
    }
}

struct Square1CubeShapeChecker;

impl CheckPattern for Square1CubeShapeChecker {
    fn is_valid(pattern: &cubing::kpuzzle::KPattern) -> bool {
        let orbit_info = &pattern.kpuzzle().data.ordered_orbit_info[0];
        assert_eq!(orbit_info.name.0, "WEDGES");

        for i in [0, 1, 2, 11, 12, 13] {
            let value = unsafe {
                pattern
                    .packed_orbit_data()
                    .get_raw_piece_or_permutation_value(orbit_info, i)
            };

            for j in [3, 6, 9] {
                if value
                    != unsafe {
                        pattern
                            .packed_orbit_data()
                            .get_raw_piece_or_permutation_value(orbit_info, i + j)
                    }
                {
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
        let scramble_pattern = scramble_pattern.apply_alg(&parse_alg!("(U_SQ_5' D_SQ_0) / (U_SQ_0 D_SQ_3) / (U_SQ_3 D_SQ_0) / (U_SQ_' D_SQ_4') / (U_SQ_4 D_SQ_2') / (U_SQ_5 D_SQ_4') / (U_SQ_2' D_SQ_0) / (U_SQ_0 D_SQ_3') / (U_SQ_' D_SQ_0) / (U_SQ_3 D_SQ_4') / (U_SQ_4 D_SQ_2') /")).unwrap();

        let phase1_start_pattern = mask(&scramble_pattern, square1_cube_shape_kpattern()).unwrap();

        if !Square1SlicableChecker::is_valid(&phase1_start_pattern) {
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

        let mut phase1_filtered_search = FilteredSearch::<Square1SlicableChecker>::new(
            kpuzzle,
            generators.clone(),
            None,
            square1_cube_shape_kpattern().clone(),
        );

        let mut phase1_solution =
            phase1_filtered_search.generate_scramble(&phase1_start_pattern, None);

        while let Some(cubing::alg::AlgNode::MoveNode(r#move)) = phase1_solution.nodes.last() {
            if r#move == &parse_move!("_SLASH_'")
            // TODO: redundant parsing
            {
                break;
            }
            phase1_solution.nodes.pop();
        }
        dbg!(&parse_move!("/"));

        let phase2_start_pattern = scramble_pattern.apply_alg(&phase1_solution).unwrap();

        // dbg!(basic_parity(phase2_start_pattern.));

        let mut phase2_filtered_search = FilteredSearch::<Square1CubeShapeChecker>::new(
            kpuzzle,
            generators,
            None,
            square1_cube_shape_kpattern().clone(),
        );
        let mut phase2_solution =
            phase2_filtered_search.generate_scramble(&phase2_start_pattern, None);

        let mut nodes = phase1_solution.nodes;
        nodes.append(&mut phase2_solution.nodes);
        Alg { nodes }.invert();

        // if let Some(solution) = filtered_search.generate_scramble(&phase1_start_pattern, 0) {
        //     //<<<
        //     return solution.invert();
        // }
    }
}
