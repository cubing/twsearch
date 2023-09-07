use std::{collections::HashMap, hash::Hash};

// This is a simple class to group a key ordering with an associated hash map.
// The "idiomatic" way to implement this would be
// https://docs.rs/indexmap/latest/indexmap/ instead of a fresh implementation.
// However, this implemenation avoids pulling in another crate, and may help
// reduce code size (important for WASM targets).
// TODO: Measure if this actually helps.
#[derive(Debug)]
pub struct HashMapWithKeyOrdering<U, V> {
    hash_map: HashMap<U, V>,
    key_ordering: Vec<U>,
}

impl<U: Clone + Eq + Hash + Sized, V: Sized + Clone> HashMapWithKeyOrdering<U, V> {
    pub fn modify_or_set(
        &mut self,
        key: &U,
        modify: impl FnOnce(&V) -> V,
        set: impl FnOnce() -> V,
    ) {
        match self.hash_map.get(key) {
            Some(v) => {
                modify(v);
            }
            None => {
                self.key_ordering.push(key.clone());
                self.hash_map.insert(key.clone(), set());
            }
        }
    }

    pub fn values(mut self) -> Vec<V> {
        let mut out = Vec::<V>::new();
        for key in self.key_ordering {
            out.push(self.hash_map.remove(&key).unwrap())
        }
        out
    }
}

impl<U, V> Default for HashMapWithKeyOrdering<U, V> {
    fn default() -> Self {
        Self {
            key_ordering: vec![],
            hash_map: HashMap::new(),
        }
    }
}
