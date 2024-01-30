use std::marker::PhantomData;
use std::{fmt, hash::Hash};

pub type ParsedDeclIdIndexType = usize;

/// An ID used to refer to an item in the [ParsedDeclEngine](super::decl_engine::ParsedDeclEngine)
pub struct ParsedDeclId<T>(ParsedDeclIdIndexType, PhantomData<T>);

impl<T> fmt::Debug for ParsedDeclId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ParsedDeclId").field(&self.0).finish()
    }
}

impl<T> ParsedDeclId<T> {
    pub(crate) fn inner(&self) -> ParsedDeclIdIndexType {
        self.0
    }
}

impl<T> Copy for ParsedDeclId<T> {}
impl<T> Clone for ParsedDeclId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Eq for ParsedDeclId<T> {}
impl<T> Hash for ParsedDeclId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}
impl<T> PartialEq for ParsedDeclId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl<T> PartialOrd for ParsedDeclId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Ord for ParsedDeclId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> ParsedDeclId<T> {
    pub(crate) fn new(id: usize) -> Self {
        ParsedDeclId(id, PhantomData)
    }

    #[allow(dead_code)]
    pub(crate) fn replace_id(&mut self, index: Self) {
        self.0 = index.0;
    }

    #[allow(dead_code)]
    pub(crate) fn dummy() -> Self {
        // we assume that `usize::MAX` id is not possible in practice
        Self(usize::MAX, PhantomData)
    }
}

#[allow(clippy::from_over_into)]
impl<T> Into<usize> for ParsedDeclId<T> {
    fn into(self) -> usize {
        self.0
    }
}
