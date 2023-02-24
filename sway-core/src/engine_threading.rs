use std::{
    cmp::Ordering,
    fmt,
    hash::{BuildHasher, Hash, Hasher},
};

use crate::{decl_engine::DeclEngine, TypeEngine};

#[derive(Clone, Copy)]
pub struct Engines<'a> {
    type_engine: &'a TypeEngine,
    decl_engine: &'a DeclEngine,
}

impl<'a> Engines<'a> {
    pub fn new(type_engine: &'a TypeEngine, decl_engine: &'a DeclEngine) -> Engines<'a> {
        Engines {
            type_engine,
            decl_engine,
        }
    }

    pub fn te(&self) -> &TypeEngine {
        self.type_engine
    }

    pub fn de(&self) -> &DeclEngine {
        self.decl_engine
    }

    pub(crate) fn unwrap(self) -> (&'a TypeEngine, &'a DeclEngine) {
        (self.type_engine, self.decl_engine)
    }

    /// Helps out some `thing: T` by adding `self` as context.
    pub fn help_out<T>(&self, thing: T) -> WithEngines<'_, T> {
        WithEngines {
            thing,
            engines: *self,
        }
    }

    /// Helps out some `thing: T` by adding `self` as context.
    pub fn with_thing<T>(self, thing: T) -> WithEngines<'a, T> {
        WithEngines {
            thing,
            engines: self,
        }
    }
}

#[derive(Clone, Copy)]
pub struct WithEngines<'a, T> {
    pub thing: T,
    pub engines: Engines<'a>,
}

impl<'a, T> WithEngines<'a, T> {
    pub fn new(thing: T, engines: Engines<'a>) -> Self {
        WithEngines { thing, engines }
    }
}

impl<T: DisplayWithEngines> fmt::Display for WithEngines<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.thing.fmt(f, self.engines)
    }
}

impl<T: DebugWithEngines> fmt::Debug for WithEngines<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.thing.fmt(f, self.engines)
    }
}

impl<T: HashWithEngines> Hash for WithEngines<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.thing.hash(state, self.engines)
    }
}

impl<T: PartialEqWithEngines> PartialEq for WithEngines<'_, T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.thing.eq(&rhs.thing, self.engines)
    }
}

impl<T: EqWithEngines> Eq for WithEngines<'_, T> {}

impl<T: OrdWithEngines> PartialOrd for WithEngines<'_, T>
where
    T: PartialEqWithEngines,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.thing.cmp(&other.thing, self.engines.te()))
    }
}

impl<T: OrdWithEngines> Ord for WithEngines<'_, T>
where
    T: EqWithEngines,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.thing.cmp(&other.thing, self.engines.te())
    }
}

pub(crate) trait DisplayWithEngines {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result;
}

impl<T: DisplayWithEngines> DisplayWithEngines for &T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result {
        (*self).fmt(f, engines)
    }
}

pub(crate) trait DebugWithEngines {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result;
}

impl<T: DebugWithEngines> DebugWithEngines for &T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result {
        (*self).fmt(f, engines)
    }
}

pub trait HashWithEngines {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>);
}

impl<T: HashWithEngines + ?Sized> HashWithEngines for &T {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        (*self).hash(state, engines)
    }
}

impl<T: HashWithEngines> HashWithEngines for Option<T> {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        match self {
            None => state.write_u8(0),
            Some(x) => x.hash(state, engines),
        }
    }
}

impl<T: HashWithEngines> HashWithEngines for [T] {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        for x in self {
            x.hash(state, engines)
        }
    }
}

pub trait EqWithEngines: PartialEqWithEngines {}

pub trait PartialEqWithEngines {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool;
}

pub trait OrdWithEngines {
    fn cmp(&self, other: &Self, type_engine: &TypeEngine) -> Ordering;
}

impl<T: EqWithEngines + ?Sized> EqWithEngines for &T {}
impl<T: PartialEqWithEngines + ?Sized> PartialEqWithEngines for &T {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        (*self).eq(*other, engines)
    }
}
impl<T: OrdWithEngines + ?Sized> OrdWithEngines for &T {
    fn cmp(&self, other: &Self, type_engine: &TypeEngine) -> Ordering {
        (*self).cmp(*other, type_engine)
    }
}

impl<T: OrdWithEngines> OrdWithEngines for Option<T> {
    fn cmp(&self, other: &Self, type_engine: &TypeEngine) -> Ordering {
        match (self, other) {
            (Some(x), Some(y)) => x.cmp(y, type_engine),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    }
}

impl<T: EqWithEngines> EqWithEngines for Option<T> {}
impl<T: PartialEqWithEngines> PartialEqWithEngines for Option<T> {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        match (self, other) {
            (None, None) => true,
            (Some(x), Some(y)) => x.eq(y, engines),
            _ => false,
        }
    }
}

impl<T: EqWithEngines> EqWithEngines for [T] {}
impl<T: PartialEqWithEngines> PartialEqWithEngines for [T] {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(x, y)| x.eq(y, engines))
    }
}
impl<T: OrdWithEngines> OrdWithEngines for [T] {
    fn cmp(&self, other: &Self, type_engine: &TypeEngine) -> Ordering {
        self.iter()
            .zip(other.iter())
            .map(|(x, y)| x.cmp(y, type_engine))
            .find(|o| o.is_ne())
            .unwrap_or_else(|| self.len().cmp(&other.len()))
    }
}

pub(crate) fn make_hasher<'a: 'b, 'b, K>(
    hash_builder: &'a impl BuildHasher,
    engines: Engines<'b>,
) -> impl Fn(&K) -> u64 + 'b
where
    K: HashWithEngines + ?Sized,
{
    move |key: &K| {
        let mut state = hash_builder.build_hasher();
        key.hash(&mut state, engines);
        state.finish()
    }
}
