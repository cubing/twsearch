use std::{mem, vec::IntoIter};

const SUBLIST_INITIAL_CAPACITY: usize = 1024;
const SUBLIST_MAX_SIZE: usize = 1_000_000; // TODO: figure out how to work nicely with final reallocation.

pub struct BulkQueue<T> {
    finalized_sublists: Vec<Vec<T>>,
    current_sublist: Vec<T>,

    size: usize,
}

// Sort of like a vector, but avoiding large reallocations on resize. Meant to be used as a queue.
// TODO: is there a crate for large queues?
// TODO: does this actually help with performance?
// TODO: do we actually want something like a `Set` instead?
impl<T> BulkQueue<T> {
    pub fn bogus_new() -> Self {
        Self {
            finalized_sublists: vec![],
            current_sublist: vec![],
            size: 0,
        }
    }

    pub fn new(initial_value: Option<T>) -> Self {
        let mut mega_queue = BulkQueue {
            finalized_sublists: vec![],
            current_sublist: Vec::with_capacity(SUBLIST_INITIAL_CAPACITY),
            size: 0,
        };
        if let Some(initial_value) = initial_value {
            mega_queue.push(initial_value)
        }
        mega_queue
    }

    pub fn push(&mut self, t: T) {
        if self.current_sublist.len() > SUBLIST_MAX_SIZE {
            self.finalized_sublists.push(mem::replace(
                &mut self.current_sublist,
                Vec::with_capacity(SUBLIST_INITIAL_CAPACITY),
            ));
        }
        self.current_sublist.push(t);
        self.size += 1;
    }

    pub fn size(&self) -> usize {
        self.size
    }

    fn finalize(&mut self) {
        self.finalized_sublists
            .push(std::mem::take(&mut self.current_sublist));
    }

    // TODO: provide a flat iterator
    pub fn into_iter(mut self) -> BulkQueueIterator<T> {
        self.finalize();
        BulkQueueIterator::new(self)
    }
}

pub struct BulkQueueIterator<T> {
    sublist_iterator: IntoIter<Vec<T>>,
    leaf_iterator: IntoIter<T>,
}

impl<T> BulkQueueIterator<T> {
    pub fn new(bulk_queue: BulkQueue<T>) -> Self {
        let mut sublist_iterator = bulk_queue.finalized_sublists.into_iter();
        let leaf_iterator = match sublist_iterator.next() {
            Some(leaf_iterator) => leaf_iterator.into_iter(),
            None => vec![].into_iter(),
        };

        Self {
            sublist_iterator,
            leaf_iterator,
        }
    }
}

impl<T> Iterator for BulkQueueIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(leaf) = self.leaf_iterator.next() {
            return Some(leaf);
        }
        self.leaf_iterator = match self.sublist_iterator.next() {
            Some(leaf_iterator) => leaf_iterator.into_iter(),
            None => return None,
        };
        self.leaf_iterator.next()
    }
}
