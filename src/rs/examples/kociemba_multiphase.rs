/* Run using:

    cargo run --release --example kociemba_multiphase

*/
use cubing::{
    alg::{parse_alg, parse_move},
    kpuzzle::{kpattern_from_json_file, kpuzzle_from_json_file},
};
use twsearch::{
    _internal::{cli::args::VerbosityLevel, search::search_logger::SearchLogger},
    experimental_lib_api::{SimpleMaskMultiphaseSearch, SimpleMaskPhaseInfo},
};

kpuzzle_from_json_file!(pub(crate), cube3x3x3_centerless, "../scramble/puzzles/definitions/3x3x3-centerless.kpuzzle.json");
kpattern_from_json_file!(pub(crate), cube3x3x3_centerless_g1_target, "../scramble/puzzles/definitions/3x3x3-G1-centerless.target-pattern.json", cube3x3x3_centerless_kpuzzle());

struct KociembaTwoPhase(SimpleMaskMultiphaseSearch);

impl KociembaTwoPhase {
    pub fn new() -> Self {
        let kpuzzle = cube3x3x3_centerless_kpuzzle();
        Self(
            SimpleMaskMultiphaseSearch::try_new(
                kpuzzle,
                vec![
                    SimpleMaskPhaseInfo {
                        name: "G1 redution".to_owned(),
                        mask: cube3x3x3_centerless_g1_target_kpattern().clone(),
                        generator_moves: vec![
                            parse_move!("U"),
                            parse_move!("L"),
                            parse_move!("F"),
                            parse_move!("R"),
                            parse_move!("B"),
                            parse_move!("D"),
                        ],
                        individual_search_options: None,
                    },
                    SimpleMaskPhaseInfo {
                        name: "Domino".to_owned(),
                        mask: kpuzzle.default_pattern().clone(),
                        generator_moves: vec![
                            parse_move!("U"),
                            parse_move!("L2"),
                            parse_move!("F2"),
                            parse_move!("R2"),
                            parse_move!("B2"),
                            parse_move!("D"),
                        ],
                        individual_search_options: None,
                    },
                ],
                Some(SearchLogger {
                    verbosity: VerbosityLevel::Info,
                }),
            )
            .unwrap(),
        )
    }
}

pub fn main() {
    let kpuzzle = cube3x3x3_centerless_kpuzzle();

    let mut kociemba_two_phase = KociembaTwoPhase::new();

    let scramble = kpuzzle
        .default_pattern()
        .apply_alg(&parse_alg!(
            "F B2 L' U2 D' B2 D B' U2 B2 U2 L2 B2 L2 D B2 U' F2 R2 F2 L"
        ))
        .unwrap();
    assert_ne!(scramble, cube3x3x3_centerless_kpuzzle().default_pattern());
    let solution = kociemba_two_phase
        .0
        .chain_first_solution_for_each_phase(&scramble)
        .unwrap();
    println!("{}", solution);
    assert_eq!(
        scramble.apply_alg(&solution).unwrap(),
        cube3x3x3_centerless_kpuzzle().default_pattern()
    );
}
