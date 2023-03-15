use std::marker::PhantomData;
use std::{fmt, hash::Hash};

/// An ID used to refer to an item in the [DeclEngine](super::decl_engine::DeclEngine)
pub struct DeclId<T>(usize, PhantomData<T>);

impl<T> fmt::Debug for DeclId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DeclId").field(&self.0).finish()
    }
}

impl<T> DeclId<T> {
    pub(crate) fn inner(&self) -> usize {
        self.0
    }
}

impl<T> Copy for DeclId<T> {}
impl<T> Clone for DeclId<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<T> Eq for DeclId<T> {}
impl<T> Hash for DeclId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}
impl<T> PartialEq for DeclId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<T> PartialOrd for DeclId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl<T> Ord for DeclId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> DeclId<T> {
    pub(crate) fn new(id: usize) -> Self {
        DeclId(id, PhantomData)
    }
}

#[allow(clippy::from_over_into)]
impl<T> Into<usize> for DeclId<T> {
    fn into(self) -> usize {
        self.0
    }
}
