use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
};

use crate::{declaration_engine::DeclarationEngine, TypeEngine};

#[derive(Clone, Copy)]
pub struct Engines<'a> {
    type_engine: &'a TypeEngine,
    declaration_engine: &'a DeclarationEngine,
}

impl Engines<'_> {
    pub fn new<'a>(
        type_engine: &'a TypeEngine,
        declaration_engine: &'a DeclarationEngine,
    ) -> Engines<'a> {
        Engines {
            type_engine,
            declaration_engine,
        }
    }

    pub fn te(&self) -> &TypeEngine {
        self.type_engine
    }

    pub fn de(&self) -> &DeclarationEngine {
        self.declaration_engine
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
        self.thing.fmt(f, self.engines.te())
    }
}

impl<T: HashWithEngines> Hash for WithEngines<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.thing.hash(state, self.engines.te())
    }
}

impl<T: PartialEqWithEngines> PartialEq for WithEngines<'_, T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.thing.eq(&rhs.thing, self.engines.te())
    }
}

impl<T: EqWithEngines> Eq for WithEngines<'_, T> {}

pub(crate) trait DisplayWithEngines {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, type_engine: &TypeEngine) -> fmt::Result;
}

impl<T: DisplayWithEngines> DisplayWithEngines for &T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, type_engine: &TypeEngine) -> fmt::Result {
        (*self).fmt(f, type_engine)
    }
}

pub trait HashWithEngines {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine);
}

impl<T: HashWithEngines + ?Sized> HashWithEngines for &T {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        (*self).hash(state, type_engine)
    }
}

impl<T: HashWithEngines> HashWithEngines for Option<T> {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        match self {
            None => state.write_u8(0),
            Some(x) => x.hash(state, type_engine),
        }
    }
}

impl<T: HashWithEngines> HashWithEngines for [T] {
    fn hash<H: Hasher>(&self, state: &mut H, type_engine: &TypeEngine) {
        for x in self {
            x.hash(state, type_engine)
        }
    }
}

pub trait EqWithEngines: PartialEqWithEngines {}

pub trait PartialEqWithEngines {
    fn eq(&self, rhs: &Self, type_engine: &TypeEngine) -> bool;
}

pub trait OrdWithEngines {
    fn cmp(&self, rhs: &Self, type_engine: &TypeEngine) -> Ordering;
}

impl<T: EqWithEngines + ?Sized> EqWithEngines for &T {}
impl<T: PartialEqWithEngines + ?Sized> PartialEqWithEngines for &T {
    fn eq(&self, rhs: &Self, type_engine: &TypeEngine) -> bool {
        (*self).eq(*rhs, type_engine)
    }
}
impl<T: OrdWithEngines + ?Sized> OrdWithEngines for &T {
    fn cmp(&self, rhs: &Self, type_engine: &TypeEngine) -> Ordering {
        (*self).cmp(*rhs, type_engine)
    }
}

impl<T: EqWithEngines> EqWithEngines for Option<T> {}
impl<T: PartialEqWithEngines> PartialEqWithEngines for Option<T> {
    fn eq(&self, rhs: &Self, type_engine: &TypeEngine) -> bool {
        match (self, rhs) {
            (None, None) => true,
            (Some(x), Some(y)) => x.eq(y, type_engine),
            _ => false,
        }
    }
}

impl<T: EqWithEngines> EqWithEngines for [T] {}
impl<T: PartialEqWithEngines> PartialEqWithEngines for [T] {
    fn eq(&self, rhs: &Self, type_engine: &TypeEngine) -> bool {
        self.len() == rhs.len()
            && self
                .iter()
                .zip(rhs.iter())
                .all(|(x, y)| x.eq(y, type_engine))
    }
}
impl<T: OrdWithEngines> OrdWithEngines for [T] {
    fn cmp(&self, rhs: &Self, type_engine: &TypeEngine) -> Ordering {
        self.iter()
            .zip(rhs.iter())
            .map(|(x, y)| x.cmp(y, type_engine))
            .find(|o| o.is_ne())
            .unwrap_or_else(|| self.len().cmp(&rhs.len()))
    }
}
