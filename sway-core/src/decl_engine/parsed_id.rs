use super::{
    parsed_engine::{ParsedDeclEngine, ParsedDeclEngineGet, ParsedDeclEngineIndex},
    DeclUniqueId,
};
use crate::{
    engine_threading::{
        DebugWithEngines, EqWithEngines, HashWithEngines, PartialEqWithEngines,
        PartialEqWithEnginesContext,
    },
    Engines,
};
use serde::{Deserialize, Serialize};
use std::{
    hash::{DefaultHasher, Hasher},
    marker::PhantomData,
    {fmt, hash::Hash},
};
use sway_types::{Named, Spanned};

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

    pub fn unique_id(&self) -> DeclUniqueId
    where
        T: 'static,
    {
        let mut hasher = DefaultHasher::default();
        std::any::TypeId::of::<T>().hash(&mut hasher);
        self.0.hash(&mut hasher);

        DeclUniqueId(hasher.finish())
    }
}

impl<T> Copy for ParsedDeclId<T> {}
impl<T> Clone for ParsedDeclId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Eq for ParsedDeclId<T> {}
impl<T> PartialEq for ParsedDeclId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> DebugWithEngines for ParsedDeclId<T>
where
    ParsedDeclEngine: ParsedDeclEngineIndex<T>,
    T: Named + Spanned + DebugWithEngines,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        let decl = engines.pe().get(self);
        DebugWithEngines::fmt(&decl, f, engines)
    }
}

impl<T> EqWithEngines for ParsedDeclId<T> {}
impl<T> PartialEqWithEngines for ParsedDeclId<T> {
    fn eq(&self, other: &Self, _ctx: &PartialEqWithEnginesContext) -> bool {
        self.0 == other.0
    }
}

impl<T> Hash for ParsedDeclId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T> HashWithEngines for ParsedDeclId<T>
where
    ParsedDeclEngine: ParsedDeclEngineIndex<T>,
    T: Named + Spanned + HashWithEngines,
{
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        let decl_engine = engines.pe();
        let decl = decl_engine.get(self);
        decl.name().hash(state);
        decl.hash(state, engines);
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

impl<T> Serialize for ParsedDeclId<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for ParsedDeclId<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = usize::deserialize(deserializer)?;
        Ok(ParsedDeclId::new(id))
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
