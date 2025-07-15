use std::{collections::HashSet, sync::Arc};

use cubing::{
    alg::Alg,
    kpuzzle::{KPattern, KPuzzle},
};

use crate::{
    _internal::{
        errors::SearchError,
        search::{
            blank_prune_table::BlankPruneTable,
            filter::filtering_decision::FilteringDecision,
            iterative_deepening::{
                iterative_deepening_search::{ImmutableSearchData, IterativeDeepeningSearch},
                search_adaptations::{IndividualSearchAdaptations, StoredSearchAdaptations},
                solution_moves::SolutionMoves,
            },
        },
    },
    experimental_lib_api::SearchPhase,
    scramble::puzzles::definitions::{kilominx_kpuzzle, kilominx_phase1_bogus_mask_kpattern},
};

use super::kilominx_scramble_finder::kilominx_front_moves;

const BACK_PIECES: [u8; 5] = [10, 11, 13, 15, 18];
const FRONT_LOCATIONS: [u8; 5] = [0, 2, 3, 8, 19];

fn back_pieces() -> HashSet<u8> {
    HashSet::<u8>::from(BACK_PIECES)
}

pub struct KilominxPhase1Search {
    back_pieces: HashSet<u8>,
    search: IterativeDeepeningSearch,
}

impl SearchPhase<KPuzzle> for KilominxPhase1Search {
    fn phase_name(&self) -> &str {
        "Move all back pieces out of the F face"
    }

    fn solutions(
        &mut self,
        phase_search_pattern: &KPattern,
    ) -> Result<Box<dyn Iterator<Item = Alg> + '_>, SearchError> {
        let back_pieces_owned = self.back_pieces.clone(); // TODO: avoid a clone
        let phase_search_pattern_owned = phase_search_pattern.clone();
        let filter_search_solution_fn =
            move |_pattern: &KPattern, solution_moves: &SolutionMoves| -> FilteringDecision {
                let orbit_info = &kilominx_kpuzzle().data.ordered_orbit_info[0];
                debug_assert_eq!(orbit_info.name.0, "CORNERS");
                let alg: Alg = Alg::from(solution_moves);
                let pattern = phase_search_pattern_owned.apply_alg(&alg).unwrap();

                for location in FRONT_LOCATIONS {
                    let piece = pattern.get_piece(orbit_info, location);
                    if back_pieces_owned.contains(&piece) {
                        return FilteringDecision::Reject;
                    }
                }

                FilteringDecision::Accept
            };
        // Since the mask is bogus, we don't actually need to apply it to `phase_search_pattern`.
        let bogus_search_pattern = kilominx_phase1_bogus_mask_kpattern();
        Ok(Box::new(self.search.search(
            bogus_search_pattern,
            Default::default(),
            IndividualSearchAdaptations {
                filter_search_solution_fn: Some(Arc::new(filter_search_solution_fn)),
            },
        )))
    }
}

impl Default for KilominxPhase1Search {
    fn default() -> Self {
        let kpuzzle = kilominx_kpuzzle();
        // TODO: support swapping out the full calculation of whether a pattern is solved?
        let search = IterativeDeepeningSearch::new(
            ImmutableSearchData::try_from_common_options_with_auto_search_generators(
                kpuzzle.clone(),
                kilominx_front_moves(),
                vec![kilominx_phase1_bogus_mask_kpattern().clone()],
                Default::default(),
            )
            .unwrap(),
            StoredSearchAdaptations {
                filter_move_transformation_fn: None,
                filter_pattern_fn: None,
            },
            Box::new(BlankPruneTable {}),
        );
        Self {
            back_pieces: back_pieces(),
            search,
        }
    }
}
