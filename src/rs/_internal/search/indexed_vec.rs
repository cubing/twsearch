use std::marker::PhantomData;

/// Contains some direct convenience methods. Use `.0` to access the underlying array.
#[derive(Clone, Debug)]
pub struct IndexedVec<K: From<usize> + Into<usize> + Default, V>(pub Vec<V>, PhantomData<K>);

impl<K: From<usize> + Into<usize> + Default, V> IndexedVec<K, V> {
    pub fn new(v: Vec<V>) -> Self {
        Self(v, Default::default())
    }

    // Convenience wrapper
    pub fn push(&mut self, v: V) {
        self.0.push(v);
    }

    pub fn at(&self, k: K) -> &V {
        &self.0[k.into()]
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    // TODO Implement `IntoIterator`
    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.0
            .iter()
            .enumerate()
            .map(|(i, v)| (std::convert::Into::<K>::into(i), v))
    }
}

impl<K: From<usize> + Into<usize> + Default, V> Default for IndexedVec<K, V> {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}
