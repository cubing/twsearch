use cubing::kpuzzle::{KPattern, KPuzzle};

use crate::{
    _internal::{
        cli::args::VerbosityLevel,
        search::{
            coordinates::masked_kpuzzle_deriver::MaskedDerivedKPuzzle,
            hash_prune_table::HashPruneTable,
            iterative_deepening::iterative_deepening_search::{
                IterativeDeepeningSearch, IterativeDeepeningSearchConstructionOptions,
            },
            search_logger::SearchLogger,
        },
    },
    experimental_lib_api::{derived_puzzle_search_phase::DerivedPuzzleSearchPhase, SearchPhase},
    scramble::{
        puzzles::definitions::cube4x4x4_phase3_target_kpattern, scramble_search::move_list_from_vec,
    },
};

// pub(crate) fn cube4x4x4_phase3_search()
pub(crate) type Cube4x4x4Phase3Puzzle = MaskedDerivedKPuzzle;

pub(crate) struct Cube4x4x4Phase3Search {
    derived_puzzle_search_phase: DerivedPuzzleSearchPhase<KPuzzle, Cube4x4x4Phase3Puzzle>,
}

impl Default for Cube4x4x4Phase3Search {
    fn default() -> Self {
        let phase3_generator_moves = move_list_from_vec(vec![
            "Uw2", "U", "Lw2", "L", "Fw", "F2", "Rw2", "R", "Bw2", "B2", "Dw2", "D",
        ]);
        let derived_puzzle =
            MaskedDerivedKPuzzle::new_from_mask(cube4x4x4_phase3_target_kpattern().clone());
        let phase3_iterative_deepening_search =
                IterativeDeepeningSearch::<Cube4x4x4Phase3Puzzle>::try_new_prune_table_construction_shim::<
                    HashPruneTable<Cube4x4x4Phase3Puzzle>,
                >(
                  derived_puzzle.clone(),
                    phase3_generator_moves,
                    vec![cube4x4x4_phase3_target_kpattern().clone()],
                    IterativeDeepeningSearchConstructionOptions {
                        search_logger: SearchLogger {
                            verbosity: VerbosityLevel::Info, // TODO
                        }.into(),
                        ..Default::default()
                    },
                    None,
                )
                .unwrap();
        let derived_puzzle_search_phase =
            DerivedPuzzleSearchPhase::<KPuzzle, MaskedDerivedKPuzzle>::new(
                "4×4×4 reduction with parity avoidance".to_owned(),
                derived_puzzle,
                phase3_iterative_deepening_search,
                Default::default(),
            );
        Self {
            derived_puzzle_search_phase,
        }
    }
}

impl SearchPhase<KPuzzle> for Cube4x4x4Phase3Search {
    fn phase_name(&self) -> &str {
        self.derived_puzzle_search_phase.phase_name()
    }

    fn first_solution(
        &mut self,
        phase_search_pattern: &KPattern,
    ) -> Result<Option<cubing::alg::Alg>, crate::_internal::errors::SearchError> {
        self.derived_puzzle_search_phase
            .first_solution(phase_search_pattern)
    }
}
