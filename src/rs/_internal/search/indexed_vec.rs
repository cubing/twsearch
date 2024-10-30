use std::marker::PhantomData;

/// Contains some direct convenience methods. Use `.0` to access the underlying array.
#[derive(Debug)]
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

// TODO: use inside the project without exporting.
#[macro_export]
/// Do not use outside `twsearch`.
macro_rules! index_type {
    ($e: ident, $u_type: ident) => {
        #[derive(PartialEq, Eq, Copy, Clone, Debug, Hash, Default, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
        pub struct $e(pub $u_type);

        // TODO: Make this a derived trait?
        impl From<$e> for $u_type {
            fn from(v: $e) -> $u_type {
                v.0
            }
        }

        // TODO: Make this a derived trait?
        impl From<$u_type> for $e {
            fn from(v: $u_type) -> Self {
                Self(v)
            }
        }

        // TODO: generalize these trait implementations?
        impl std::ops::Deref for $e {
            type Target = $u_type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::Add for $e {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl std::ops::Sub for $e {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl std::str::FromStr for $e {
            type Err = <$u_type as std::str::FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(<$u_type as std::str::FromStr>::from_str(s)?))
            }
        }
    };
}

pub use index_type;
