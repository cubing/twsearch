use std::{
    ops::{BitAnd, Index, IndexMut},
    slice::Iter,
};

use crate::{_internal::search::indexed_vec::IndexedVec, whole_number_newtype};

whole_number_newtype!(MoveClassIndex, usize);

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Debug, Hash, Clone)]
pub(crate) struct MoveClassMask(IndexedVec<MoveClassIndex, bool>);

impl From<Vec<bool>> for MoveClassMask {
    fn from(value: Vec<bool>) -> Self {
        Self(IndexedVec::new(value))
    }
}

impl PartialEq for MoveClassMask {
    fn eq(&self, other: &Self) -> bool {
        // We explicitly avoid implementing `Eq` for `IndexedVec` to avoid accidental unperformant operations, so we have to call into the lowest-level `Vec` here.
        self.0 .0 == other.0 .0
    }
}

impl Eq for MoveClassMask {}

impl BitAnd for &MoveClassMask {
    type Output = MoveClassMask;

    fn bitand(self, rhs: Self) -> MoveClassMask {
        debug_assert_eq!(self.len(), rhs.len());
        self.iter()
            .zip(rhs.iter())
            .map(|(lhs, rhs)| *lhs & *rhs)
            .collect::<Vec<bool>>()
            .into()
    }
}

impl Index<MoveClassIndex> for MoveClassMask {
    type Output = bool;

    fn index(&self, index: MoveClassIndex) -> &bool {
        &self.0[index]
    }
}

impl IndexMut<MoveClassIndex> for MoveClassMask {
    fn index_mut(&mut self, index: MoveClassIndex) -> &mut bool {
        &mut self.0[index]
    }
}

impl MoveClassMask {
    fn iter(&self) -> Iter<'_, bool> {
        self.0 .0.iter()
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}
