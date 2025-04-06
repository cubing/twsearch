use cubing::kpuzzle::{KPattern, KPuzzle};

use crate::{
    _internal::search::{
        filter::filtering_decision::FilteringDecision,
        iterative_deepening::iterative_deepening_search::IndividualSearchOptions,
        search_logger::SearchLogger,
    },
    experimental_lib_api::{
        KPuzzleSimpleMaskPhase, KPuzzleSimpleMaskPhaseConstructionOptions, MultiPhaseSearch,
    },
    scramble::{
        collapse::collapse_adjacent_moves,
        get_kpuzzle::GetKPuzzle,
        puzzles::definitions::{
            megaminx_kpuzzle, megaminx_phase10_target_kpattern, megaminx_phase11_target_kpattern,
            megaminx_phase1_target_kpattern, megaminx_phase2_target_kpattern,
            megaminx_phase3_target_kpattern, megaminx_phase4_target_kpattern,
            megaminx_phase5_target_kpattern, megaminx_phase6_target_kpattern,
            megaminx_phase7_target_kpattern, megaminx_phase8_target_kpattern,
            megaminx_phase9_target_kpattern,
        },
        randomize::{
            randomize_orbit, OrbitOrientationConstraint, OrbitPermutationConstraint,
            OrbitRandomizationConstraints,
        },
        scramble_search::move_list_from_vec,
        solving_based_scramble_finder::{
            NoScrambleAssociatedData, NoScrambleOptions, SolvingBasedScrambleFinder,
        },
    },
};

// Note this would be called `MegaminxScrambleFinder`, but we've avoiding that
// name because we're not using it to generate scrambles.
pub struct MegaminxSolver {
    multi_phase_search: MultiPhaseSearch<KPuzzle>,
}

impl Default for MegaminxSolver {
    fn default() -> Self {
        let search_logger = SearchLogger {
            verbosity: crate::_internal::cli::args::VerbosityLevel::Info,
        };
        let kpuzzle = megaminx_kpuzzle();

        let construct_phase = |phase_number: usize,
                               mask: &KPattern,
                               move_list_vec: Vec<&str>|
         -> Box<KPuzzleSimpleMaskPhase> {
            Box::new(
                KPuzzleSimpleMaskPhase::try_new(
                    format!("Megaminx phase #{}", phase_number),
                    mask.clone(),
                    move_list_from_vec(move_list_vec),
                    KPuzzleSimpleMaskPhaseConstructionOptions {
                        search_logger: Some(search_logger.clone()),
                        individual_search_options: Some(IndividualSearchOptions {
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                )
                .unwrap(),
            )
        };
        let multi_phase_search = MultiPhaseSearch::try_new(
            kpuzzle.clone(),
            vec![
                construct_phase(1, megaminx_phase1_target_kpattern(), vec!["Uv", "Lv"]),
                construct_phase(
                    2,
                    megaminx_phase2_target_kpattern(),
                    vec!["U", "L", "F", "R", "BR", "BL", "FL", "FR", "DL", "DR", "B"],
                ),
                construct_phase(
                    3,
                    megaminx_phase3_target_kpattern(),
                    vec!["U", "L", "F", "R", "BR", "BL", "FL", "FR", "DR"],
                ),
                construct_phase(
                    4,
                    megaminx_phase4_target_kpattern(),
                    vec!["U", "L", "F", "R", "BR", "BL", "FL", "FR"],
                ),
                construct_phase(
                    5,
                    megaminx_phase5_target_kpattern(),
                    vec!["U", "L", "F", "R", "BR", "BL", "FR"], // TODO: Allow FL?
                ),
                construct_phase(
                    6,
                    megaminx_phase6_target_kpattern(),
                    vec!["U", "L", "F", "R", "BR", "BL"],
                ),
                construct_phase(
                    7,
                    megaminx_phase7_target_kpattern(),
                    vec!["U", "L", "F", "R", "BR"],
                ),
                construct_phase(
                    8,
                    megaminx_phase8_target_kpattern(),
                    vec!["U", "L", "F", "R"],
                ),
                construct_phase(9, megaminx_phase9_target_kpattern(), vec!["U", "F", "R"]),
                construct_phase(10, megaminx_phase10_target_kpattern(), vec!["U", "F", "R"]),
                construct_phase(11, megaminx_phase11_target_kpattern(), vec!["U", "F", "R"]),
                construct_phase(12, &kpuzzle.default_pattern(), vec!["U", "F", "R"]),
            ],
            Some(search_logger),
        )
        .unwrap();
        Self { multi_phase_search }
    }
}

impl SolvingBasedScrambleFinder for MegaminxSolver {
    type TPuzzle = KPuzzle;
    type ScrambleAssociatedData = NoScrambleAssociatedData;
    type ScrambleOptions = NoScrambleOptions;

    fn generate_fair_unfiltered_random_pattern(
        &mut self,
        _scramble_options: &Self::ScrambleOptions,
    ) -> (KPattern, Self::ScrambleAssociatedData) {
        // TODO: centers?
        let mut pattern = megaminx_kpuzzle().default_pattern();
        randomize_orbit(
            &mut pattern,
            0,
            "CORNERS",
            OrbitRandomizationConstraints {
                orientation: Some(OrbitOrientationConstraint::SumToZero),
                permutation: Some(OrbitPermutationConstraint::EvenParity),
                ..Default::default()
            },
        );
        randomize_orbit(
            &mut pattern,
            0,
            "EDGES",
            OrbitRandomizationConstraints {
                orientation: Some(OrbitOrientationConstraint::SumToZero),
                permutation: Some(OrbitPermutationConstraint::EvenParity),
                ..Default::default()
            },
        );
        (pattern, NoScrambleAssociatedData {})
    }

    fn filter_pattern(
        &mut self,
        _pattern: &KPattern,
        _scramble_associated_data: &Self::ScrambleAssociatedData,
        _scramble_options: &Self::ScrambleOptions,
    ) -> FilteringDecision {
        dbg!("WARNING: Megaminx filtering is not implemented yet.");
        FilteringDecision::Accept
    }

    fn solve_pattern(
        &mut self,
        pattern: &KPattern,
        _scramble_associated_data: &Self::ScrambleAssociatedData,
        _scramble_options: &Self::ScrambleOptions,
    ) -> Result<cubing::alg::Alg, crate::_internal::errors::SearchError> {
        self.multi_phase_search
            .chain_first_solution_for_each_phase(pattern)
    }

    fn collapse_inverted_alg(&mut self, alg: cubing::alg::Alg) -> cubing::alg::Alg {
        collapse_adjacent_moves(alg, 5, -1)
    }
}

impl GetKPuzzle for MegaminxSolver {
    fn get_kpuzzle(&self) -> &'static KPuzzle {
        megaminx_kpuzzle()
    }
}
