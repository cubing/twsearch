/* Run using:

    cargo run --release --example kociemba_multiphase

*/
use cubing::{
    alg::{parse_alg, parse_move},
    kpuzzle::{kpattern_from_json_file, kpuzzle_from_json_file, KPuzzle},
};
use twsearch::{
    _internal::{cli::args::VerbosityLevel, search::search_logger::SearchLogger},
    experimental_lib_api::{KPuzzleSimpleMaskPhase, MultiPhaseSearch},
};

kpuzzle_from_json_file!(pub(crate), cube3x3x3_centerless, "../scramble/puzzles/definitions/3x3x3/3x3x3-centerless.kpuzzle.json");
kpattern_from_json_file!(pub(crate), cube3x3x3_centerless_g1_target, "../scramble/puzzles/definitions/3x3x3/3x3x3-G1-centerless.target-pattern.json", cube3x3x3_centerless_kpuzzle());

struct KociembaTwoPhase(MultiPhaseSearch<KPuzzle>);

impl KociembaTwoPhase {
    pub fn new() -> Self {
        let kpuzzle = cube3x3x3_centerless_kpuzzle();
        Self(
            MultiPhaseSearch::try_new(
                kpuzzle.clone(),
                vec![
                    Box::new(
                        KPuzzleSimpleMaskPhase::try_new(
                            "G1 reduction".to_owned(),
                            cube3x3x3_centerless_g1_target_kpattern().clone(),
                            vec![
                                parse_move!("U").to_owned(),
                                parse_move!("L").to_owned(),
                                parse_move!("F").to_owned(),
                                parse_move!("R").to_owned(),
                                parse_move!("B").to_owned(),
                                parse_move!("D").to_owned(),
                            ],
                            Default::default(),
                        )
                        .unwrap(),
                    ),
                    Box::new(
                        KPuzzleSimpleMaskPhase::try_new(
                            "Domino".to_owned(),
                            kpuzzle.default_pattern().clone(),
                            vec![
                                parse_move!("U").to_owned(),
                                parse_move!("L2").to_owned(),
                                parse_move!("F2").to_owned(),
                                parse_move!("R2").to_owned(),
                                parse_move!("B2").to_owned(),
                                parse_move!("D").to_owned(),
                            ],
                            Default::default(),
                        )
                        .unwrap(),
                    ),
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
        .apply_alg(parse_alg!(
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
