use crate::{CanonicalFSMState, PackedKPattern};

use super::bulk_queue::BulkQueue;

// use super::bulk_queue::{BulkQueue, BulkQueueIterator};

#[derive(Clone, Eq)]
pub(crate) struct QueueItem {
    pub(crate) canonical_fsm_state: CanonicalFSMState,
    // TODO: test if storing a byte slice reference improves performance
    pub(crate) pattern: PackedKPattern,
}

impl PartialEq for QueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
    }
}

impl PartialOrd for QueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.pattern
            .byte_slice()
            .partial_cmp(other.pattern.byte_slice())
    }
}

impl Ord for QueueItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.pattern.byte_slice().cmp(other.pattern.byte_slice())
    }
}

// TODO: genericize
impl BulkQueue<QueueItem> {
    pub fn sort_and_dedup(
        mut self,
        a_queue: &BulkQueue<QueueItem>,
        b_queue: &BulkQueue<QueueItem>,
    ) -> BulkQueue<QueueItem> {
        self.list.sort_unstable();
        let mut output_queue = BulkQueue::<QueueItem>::default();

        let mut a_iter = a_queue.iter().peekable();
        // let mut previous_iter_latest = previous_iter.next();
        let mut b_iter = b_queue.iter().peekable();
        // let mut current_iter_latest = current_iter.next();
        let sorted_iter = &mut self.into_iter().peekable();
        let mut sorted_iter_latest = match sorted_iter.next() {
            Some(next_iter_latest) => next_iter_latest,
            None => return output_queue,
        };

        loop {
            // TODO: calculate this so we can store and reuse the final comparison
            while a_iter.next_if(|v| v <= &&sorted_iter_latest).is_some() {}
            while b_iter.next_if(|v| v <= &&sorted_iter_latest).is_some() {}
            let equal = match a_iter.peek() {
                Some(v) => v == &&sorted_iter_latest,
                None => false,
            };
            let equal = equal
                || match b_iter.peek() {
                    Some(v) => v == &&sorted_iter_latest,
                    None => false,
                };
            while sorted_iter.next_if(|v| v == &sorted_iter_latest).is_some() {}
            if !equal {
                output_queue.push(sorted_iter_latest)
            }
            sorted_iter_latest = match sorted_iter.next() {
                Some(sorted_iter_latest) => sorted_iter_latest,
                None => return output_queue,
            }
        }
    }
}
