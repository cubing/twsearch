use std::collections::HashMap;

use cubing::kpuzzle::{KPattern, KPuzzle};

use crate::{
    _internal::{
        options::{Generators, MetricEnum}, CheckPattern, SearchGenerators
    },
    scramble::randomize::BasicParity,
};

use super::{mask_pattern::mask, square1::wedge_parity};

#[derive(PartialEq, Eq, Hash, Clone)]
struct LookupPattern {
    masked_pattern: KPattern,
    parity: BasicParity,
}

pub struct PhaseLookupTable {
    index_to_lookup_pattern: Vec<LookupPattern>, // TODO: support optimizations when the size is known ahead of time
    lookup_pattern_to_index: HashMap<LookupPattern, usize>,
}

pub fn build_phase_lookup_table<C: CheckPattern>(
    kpuzzle: KPuzzle,
    generators: &Generators,
    phase_mask: KPattern,
) -> PhaseLookupTable {
    let random_start = false; // TODO: for scrambles, we may want this to be true
    let search_generators =
        SearchGenerators::try_new(&kpuzzle, generators, &MetricEnum::Hand, random_start)
            .expect("Couldn't build SearchGenerators while building PhaseLookupTable");

    let mut fringe = vec![kpuzzle.default_pattern()];

    let mut index_to_lookup_pattern = Vec::<LookupPattern>::default();
    let mut lookup_pattern_to_index = HashMap::<LookupPattern, usize>::default();

    while let Some(full_pattern) = fringe.pop() {
        let parity = wedge_parity(&full_pattern);
        let masked_pattern = mask(&full_pattern, &phase_mask).unwrap();

        if !C::is_valid(&masked_pattern) {
            continue;
        }

        let lookup_pattern = LookupPattern {
            masked_pattern,
            parity,
        };

        if lookup_pattern_to_index.contains_key(&lookup_pattern) {
            // TODO: consider avoiding putting things in the fringe that are already in the fringe
            // or lookup table.
            continue;
        }

        let index = index_to_lookup_pattern.len();
        index_to_lookup_pattern.push(lookup_pattern.clone());
        lookup_pattern_to_index.insert(lookup_pattern, index);

        for move_transformation_info in &search_generators.flat {
            // <<< let flat_move_index = move_transformation_info.flat_move_index;
            fringe
                .push(full_pattern.apply_transformation(&move_transformation_info.transformation));
        }
    }
    println!(
        "PhaseLookupTable has size {}",
        index_to_lookup_pattern.len()
    );

    return PhaseLookupTable {
        index_to_lookup_pattern,
        lookup_pattern_to_index,
    };
}
