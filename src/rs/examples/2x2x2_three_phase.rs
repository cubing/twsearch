use cubing::{
    alg::{parse_alg, parse_move},
    kpuzzle::kpattern_from_json_file,
    puzzles::cube2x2x2_kpuzzle,
};
use twsearch::{
    _internal::{cli::args::VerbosityLevel, search::search_logger::SearchLogger},
    experimental_lib_api::{SimpleMaskMultiphaseSearch, SimpleMaskPhaseInfo},
};

kpattern_from_json_file!(
    pub(crate),
    phase1,
    "./2x2x2_three_phase/phase1-D-orientation.kpattern.json",
    cube2x2x2_kpuzzle()
);

kpattern_from_json_file!(
    pub(crate),
    phase2,
    "./2x2x2_three_phase/phase2-U-orientation.kpattern.json",
    cube2x2x2_kpuzzle()
);

kpattern_from_json_file!(
    pub(crate),
    phase3,
    "./2x2x2_three_phase/phase3-PBL.kpattern.json",
    cube2x2x2_kpuzzle()
);

pub fn main() {
    let kpuzzle = cube2x2x2_kpuzzle();

    // Note: this solver isn't really a "good idea". It's just a convenient test
    // of three phases, on a puzzle with few-ish states.
    let mut search = SimpleMaskMultiphaseSearch::try_new(
        kpuzzle,
        vec![
            SimpleMaskPhaseInfo {
                name: "D orientation".to_owned(),
                mask: phase1_kpattern().clone(),
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
                name: "U orientation".to_owned(),
                mask: phase2_kpattern().clone(),
                generator_moves: vec![parse_move!("U"), parse_move!("F"), parse_move!("R")],
                individual_search_options: None,
            },
            SimpleMaskPhaseInfo {
                name: "PBL".to_owned(),
                mask: phase3_kpattern().clone(),
                generator_moves: vec![
                    parse_move!("U"),
                    parse_move!("R2"),
                    parse_move!("F2"),
                    parse_move!("L2"),
                    parse_move!("D"),
                ],
                individual_search_options: None,
            },
        ],
        Some(SearchLogger {
            verbosity: VerbosityLevel::Info,
        }),
    )
    .unwrap();

    // let scramble = random_scramble_for_event(Event::Cube2x2x2Speedsolving).unwrap();
    let scramble = kpuzzle
        .default_pattern()
        .apply_alg(&parse_alg!("U2 F2 R' U' L F2 U R2 F' R' F'"))
        .unwrap();
    println!(
        "{}",
        search
            .chain_first_solution_for_each_phase(&scramble)
            .unwrap()
    );
}
