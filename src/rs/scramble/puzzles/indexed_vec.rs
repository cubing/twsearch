use std::marker::PhantomData;

/// Contains some direct convenience methods. Use `.0` to access the underlying array.
#[derive(Debug)]
pub struct IndexedVec<K: From<usize> + Into<usize> + Default, V>(pub(crate) Vec<V>, PhantomData<K>);

impl<K: From<usize> + Into<usize> + Default, V> IndexedVec<K, V> {
    // pub fn new(v: Vec<V>) -> Self {
    //     Self(v, Default::default())
    // }

    // Convenience wrapper
    pub fn push(&mut self, v: V) {
        self.0.push(v);
    }

    pub fn at(&self, k: K) -> &V {
        &self.0[k.into()]
    }

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

// TODO: use inside the project without exporting.
#[macro_export]
/// Do not use outside `twsearch`.
macro_rules! index_type {
    ($e: ident) => {
      #[derive(PartialEq, Eq, Copy, Clone, Debug, Hash, Default)]
      pub struct $e(pub usize);

      // TODO: Make this a derived trait?
      impl From<$e> for usize {
          fn from(v: $e) -> usize {
              v.0
          }
      }

      // TODO: Make this a derived trait?
      impl From<usize> for $e {
          fn from(v: usize) -> Self {
              Self(v)
          }
      }
    };
}

pub use index_type;
