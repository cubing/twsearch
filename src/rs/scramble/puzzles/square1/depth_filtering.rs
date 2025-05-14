use std::sync::Arc;

use cubing::kpuzzle::KPuzzle;

use crate::_internal::search::iterative_deepening::search_adaptations::StoredSearchAdaptations;

use super::square1_shape_traversal_filter::shape_traversal_filter_pattern;

// TODO: we currently take `square1_phase1_puzzle` as an argument to keep construction DRY. There's probably a better way to do this.
pub(crate) fn square1_depth_filtering_search_adaptations_without_prune_table(
) -> StoredSearchAdaptations<KPuzzle> {
    StoredSearchAdaptations {
        filter_move_transformation_fn: None,
        filter_pattern_fn: Some(Arc::new(Box::new(shape_traversal_filter_pattern))),
    }
}
