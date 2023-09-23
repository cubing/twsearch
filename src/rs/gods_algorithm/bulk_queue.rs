use std::{slice::Iter, vec::IntoIter};

const INITIAL_CAPACITY: usize = 1024;

pub struct BulkQueue<T> {
    pub(crate) list: Vec<T>,
}

// Sort of like a vector, but avoiding large reallocations on resize. Meant to be used as a queue.
// TODO: is there a crate for large queues?
// TODO: does this actually help with performance?
// TODO: do we actually want something like a `Set` instead?
impl<T> BulkQueue<T> {
    pub fn new(initial_value: Option<T>) -> Self {
        let mut bulk_queue = Self::default();
        if let Some(initial_value) = initial_value {
            bulk_queue.push(initial_value)
        }
        bulk_queue
    }

    pub fn push(&mut self, t: T) {
        self.list.push(t);
    }

    pub fn size(&self) -> usize {
        self.list.len()
    }

    pub fn iter(&self) -> Iter<T> {
        self.list.iter()
    }

    pub fn into_iter(self) -> IntoIter<T> {
        self.list.into_iter()
    }
}

impl<T> Default for BulkQueue<T> {
    fn default() -> Self {
        Self {
            list: Vec::<T>::with_capacity(INITIAL_CAPACITY),
        }
    }
}
