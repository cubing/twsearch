use std::{mem, slice::Iter, vec::IntoIter};

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
    pub fn iter(&mut self) -> BulkQueueIterator<T> {
        self.finalize();
        BulkQueueIterator::new(self)
    }
}

pub struct BulkQueueIterator<'a, T> {
    sublist_iterator: Iter<'a, Vec<T>>,
    leaf_iterator: Option<Iter<'a, T>>,
    next_item: Option<&'a T>,
}

impl<'a, T> BulkQueueIterator<'a, T> {
    pub fn new(bulk_queue: &'a mut BulkQueue<T>) -> Self {
        let mut sublist_iterator = bulk_queue.finalized_sublists.iter();
        let (leaf_iterator, next_item) = match sublist_iterator.next() {
            Some(leaf_iterator) => {
                let mut leaf_iterator = leaf_iterator.iter();
                let next_item = leaf_iterator.next();
                (Some(leaf_iterator), next_item)
            }
            None => (None, None),
        };

        Self {
            sublist_iterator,
            leaf_iterator,
            next_item,
        }
    }
}

impl<'a, T> Iterator for BulkQueueIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let current_item = Some(match self.next_item {
            Some(current_item) => current_item,
            None => return None,
        });

        let leaf_iterator = match &mut self.leaf_iterator {
            Some(leaf_iterator) => leaf_iterator,
            None => return current_item,
        };

        match leaf_iterator.next() {
            Some(maybe_next_item) => {
                self.next_item = Some(maybe_next_item);
                return current_item;
            }
            None => {}
        };

        match self.sublist_iterator.next() {
            Some(sublist_iterator) => {
                self.next_item = Some(maybe_next_item);
                return current_item;
            }
            None => {}
        };
    }
}
