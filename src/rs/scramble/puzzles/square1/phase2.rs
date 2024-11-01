use cubing::kpuzzle::KPuzzle;

use crate::{
    _internal::search::{
        check_pattern::PatternValidityChecker, hash_prune_table::HashPruneTable,
        idf_search::SearchOptimizations,
    },
    scramble::puzzles::square1::wedges::{WedgeType, WEDGE_TYPE_LOOKUP},
};

struct Phase2Checker;

impl PatternValidityChecker<KPuzzle> for Phase2Checker {
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

struct Square1Phase2Optimizations {}

impl SearchOptimizations<KPuzzle> for Square1Phase2Optimizations {
    type PatternValidityChecker = Phase2Checker;

    type PruneTable = HashPruneTable<KPuzzle, Phase2Checker>;
}
