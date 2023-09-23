use crate::{CanonicalFSMState, PackedKPattern};

// use super::bulk_queue::{BulkQueue, BulkQueueIterator};

#[derive(Eq)]
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

// impl PartialOrd for QueueItem {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         self.pattern
//             .byte_slice()
//             .partial_cmp(other.pattern.byte_slice())
//     }
// }

// impl Ord for QueueItem {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.pattern.byte_slice().cmp(other.pattern.byte_slice())
//     }
// }

// // TODO: genericize
// impl BulkQueue<QueueItem> {
//     pub fn merge_next(
//         previous: &BulkQueue<QueueItem>,
//         current: &BulkQueue<QueueItem>,
//         next: BulkQueue<QueueItem>,
//     ) -> BulkQueue<QueueItem> {
//         let merged_queue = BulkQueue::<QueueItem>::new(None);

//         let mut previous_iter = previous.iter();
//         let mut previous_iter_latest = previous_iter.next();
//         let mut current_iter = current.iter();
//         let mut current_iter_latest = current_iter.next();
//         let mut next_iter = &mut next.iter();
//         let mut next_iter_latest = match next_iter.next() {
//             Some(next_iter_latest) => next_iter_latest,
//             None => return merged_queue,
//         };

//         loop {
//           previous_iter.take_while(predicate)
//             // if let Some(some_previous_iter_latest) = &previous_iter_latest {
//             //     let mut some_previous_iter_latest = some_previous_iter_latest;
//             //     while some_previous_iter_latest < next_iter_latest {
//             //         some_previous_iter_latest = match previous_iter.next() {
//             //             Some(some_previous_iter_latest) => some_previous_iter_latest,
//             //             None => todo!(),
//             //         }
//             //     }
//             //     previous_iter_latest = some_previous_iter_latest
//             // };

//             loop {
//                 match next_iter.next() {
//                     Some(maybe_next) => {
//                         if maybe_next != next_iter_latest {
//                             next_iter_latest = maybe_next;
//                             break;
//                         }
//                     }
//                     None => return merged_queue,
//                 }
//             }
//         }
//     }
// }
