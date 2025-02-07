use cubing::{alg::parse_move, kpuzzle::kpattern_from_json_file, puzzles::cube2x2x2_kpuzzle};
use twsearch::{
    _internal::{cli::args::VerbosityLevel, search::search_logger::SearchLogger},
    experimental_lib_api::{SimpleMaskMultiphaseSearch, SimpleMaskPhaseInfo},
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
    let mut search = SimpleMaskMultiphaseSearch::try_new(
        kpuzzle,
        vec![
            SimpleMaskPhaseInfo {
                name: "D orientation".to_owned(),
                mask: phase1_kpattern().clone(),
                generator_moves: vec![
                    parse_move!("U").to_owned(),
                    parse_move!("L").to_owned(),
                    parse_move!("F").to_owned(),
                    parse_move!("R").to_owned(),
                    parse_move!("B").to_owned(),
                    parse_move!("D").to_owned(),
                ],
                individual_search_options: None,
            },
            SimpleMaskPhaseInfo {
                name: "U orientation".to_owned(),
                mask: phase2_kpattern().clone(),
                generator_moves: vec![
                    parse_move!("U").to_owned(),
                    parse_move!("F").to_owned(),
                    parse_move!("R").to_owned(),
                ],
                individual_search_options: None,
            },
            SimpleMaskPhaseInfo {
                name: "PBL".to_owned(),
                mask: kpuzzle.default_pattern().clone(),
                generator_moves: vec![
                    parse_move!("U").to_owned(),
                    parse_move!("R2").to_owned(),
                    parse_move!("F2").to_owned(),
                    parse_move!("L2").to_owned(),
                    parse_move!("D").to_owned(),
                ],
                individual_search_options: None,
            },
        ],
        Some(SearchLogger {
            verbosity: VerbosityLevel::Info,
        }),
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
