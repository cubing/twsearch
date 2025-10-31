// Run using: cargo run --package twsearch --release --example 2x2x2_three_phase

use cubing::{alg::parse_move, kpuzzle::kpattern_from_json_file, puzzles::cube2x2x2_kpuzzle};
use twsearch::{
    _internal::{cli::args::VerbosityLevel, search::search_logger::SearchLogger},
    experimental_lib_api::{KPuzzleSimpleMaskPhase, MultiPhaseSearch, MultiPhaseSearchOptions},
    scramble::{random_scramble_for_event, Event},
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

pub fn main() {
    let kpuzzle = cube2x2x2_kpuzzle();

    // Note: this solver isn't really a "good idea". It's just a convenient test
    // of three phases, on a puzzle with few-ish states.
    let mut search = MultiPhaseSearch::try_new(
        kpuzzle.clone(),
        vec![
            Box::new(
                KPuzzleSimpleMaskPhase::try_new(
                    "D orientation".to_owned(),
                    phase1_kpattern().clone(),
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
                    "U orientation".to_owned(),
                    phase2_kpattern().clone(),
                    vec![
                        parse_move!("U").to_owned(),
                        parse_move!("F").to_owned(),
                        parse_move!("R").to_owned(),
                    ],
                    Default::default(),
                )
                .unwrap(),
            ),
            Box::new(
                KPuzzleSimpleMaskPhase::try_new(
                    "PBL".to_owned(),
                    kpuzzle.default_pattern().clone(),
                    vec![
                        parse_move!("U").to_owned(),
                        parse_move!("R2").to_owned(),
                        parse_move!("F2").to_owned(),
                        parse_move!("L2").to_owned(),
                        parse_move!("D").to_owned(),
                    ],
                    Default::default(),
                )
                .unwrap(),
            ),
        ],
        MultiPhaseSearchOptions {
            search_logger: SearchLogger {
                verbosity: VerbosityLevel::Info,
            },
            ..Default::default()
        },
    )
    .unwrap();

    let scramble_alg = random_scramble_for_event(Event::Cube2x2x2Speedsolving).unwrap();
    let scramble = kpuzzle.default_pattern().apply_alg(&scramble_alg).unwrap();
    println!(
        "{} // scramble alg
{} // solution",
        scramble_alg,
        search
            .chain_first_solution_for_each_phase(&scramble)
            .unwrap()
    );
}
