use crate::{
    decl_engine::{parsed_engine::ParsedDeclEngine, DeclEngine},
    query_engine::QueryEngine,
    type_system::TypeEngine,
};
use std::{
    cmp::Ordering,
    fmt,
    hash::{BuildHasher, Hash, Hasher},
};
use sway_types::{SourceEngine, Span};

#[derive(Clone, Debug, Default)]
pub struct Engines {
    type_engine: TypeEngine,
    decl_engine: DeclEngine,
    parsed_decl_engine: ParsedDeclEngine,
    query_engine: QueryEngine,
    source_engine: SourceEngine,
}

impl Engines {
    pub fn te(&self) -> &TypeEngine {
        &self.type_engine
    }

    pub fn de(&self) -> &DeclEngine {
        &self.decl_engine
    }

    pub fn pe(&self) -> &ParsedDeclEngine {
        &self.parsed_decl_engine
    }

    pub fn qe(&self) -> &QueryEngine {
        &self.query_engine
    }

    pub fn se(&self) -> &SourceEngine {
        &self.source_engine
    }

    /// Removes all data associated with `program_id` from the declaration and type engines.
    /// It is intended to be used during garbage collection to remove any data that is no longer needed.
    pub fn clear_program(&mut self, program_id: &sway_types::ProgramId) {
        self.type_engine.clear_program(program_id);
        self.decl_engine.clear_program(program_id);
        self.parsed_decl_engine.clear_program(program_id);
    }

    /// Helps out some `thing: T` by adding `self` as context.
    pub fn help_out<T>(&self, thing: T) -> WithEngines<'_, T> {
        WithEngines {
            thing,
            engines: self,
        }
    }
}

#[derive(Clone, Copy)]
pub struct WithEngines<'a, T> {
    pub thing: T,
    pub engines: &'a Engines,
}

impl<'a, T> WithEngines<'a, T> {
    pub fn new(thing: T, engines: &'a Engines) -> Self {
        WithEngines { thing, engines }
    }
}

/// Displays the user-friendly formatted view of `thing` using `engines` as context.
impl<T: DisplayWithEngines> fmt::Display for WithEngines<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.thing.fmt(f, self.engines)
    }
}

/// Displays the internals of `thing` using `engines` as context. Useful for debugging.
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
        self.thing
            .eq(&rhs.thing, &PartialEqWithEnginesContext::new(self.engines))
    }
}

impl<T: EqWithEngines> Eq for WithEngines<'_, T> {}

impl<T: OrdWithEngines> PartialOrd for WithEngines<'_, T>
where
    T: PartialEqWithEngines,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.thing
                .cmp(&other.thing, &OrdWithEnginesContext::new(self.engines)),
        )
    }
}

impl<T: OrdWithEngines> Ord for WithEngines<'_, T>
where
    T: EqWithEngines,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.thing
            .cmp(&other.thing, &OrdWithEnginesContext::new(self.engines))
    }
}

pub(crate) trait DisplayWithEngines {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result;
}

impl<T: DisplayWithEngines> DisplayWithEngines for &T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        (*self).fmt(f, engines)
    }
}

impl<T: DisplayWithEngines> DisplayWithEngines for Option<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        match self {
            None => Ok(()),
            Some(x) => x.fmt(f, engines),
        }
    }
}

impl<T: DisplayWithEngines> DisplayWithEngines for Box<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        (**self).fmt(f, engines)
    }
}

impl<T: DisplayWithEngines> DisplayWithEngines for Vec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        let text = self
            .iter()
            .map(|e| format!("{}", engines.help_out(e)))
            .collect::<Vec<_>>()
            .join(", ")
            .to_string();
        f.write_str(&text)
    }
}

impl DisplayWithEngines for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        let file = self
            .source_id()
            .and_then(|id| engines.source_engine.get_file_name(id));
        f.write_fmt(format_args!("Span {{ {:?}, {} }}", file, self.line_col()))
    }
}

pub(crate) trait DebugWithEngines {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result;
}

impl<T: DebugWithEngines> DebugWithEngines for &T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        (*self).fmt(f, engines)
    }
}

impl<T: DebugWithEngines> DebugWithEngines for std::sync::Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        (**self).fmt(f, engines)
    }
}

impl<T: DebugWithEngines> DebugWithEngines for Option<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        match self {
            None => Ok(()),
            Some(x) => x.fmt(f, engines),
        }
    }
}

impl<T: DebugWithEngines> DebugWithEngines for Box<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        (**self).fmt(f, engines)
    }
}

impl<T: DebugWithEngines> DebugWithEngines for Vec<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: &Engines) -> fmt::Result {
        let text = self
            .iter()
            .map(|e| format!("{:?}", engines.help_out(e)))
            .collect::<Vec<_>>()
            .join(", ")
            .to_string();
        f.write_str(&text)
    }
}

pub trait HashWithEngines {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines);
}

impl<T: HashWithEngines + ?Sized> HashWithEngines for &T {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        (*self).hash(state, engines)
    }
}

impl<T: HashWithEngines> HashWithEngines for Option<T> {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        match self {
            None => state.write_u8(0),
            Some(x) => x.hash(state, engines),
        }
    }
}

impl<T: HashWithEngines> HashWithEngines for [T] {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        for x in self {
            x.hash(state, engines)
        }
    }
}

impl<T: HashWithEngines> HashWithEngines for Box<T> {
    fn hash<H: Hasher>(&self, state: &mut H, engines: &Engines) {
        (**self).hash(state, engines)
    }
}

pub trait EqWithEngines: PartialEqWithEngines {}

pub struct PartialEqWithEnginesContext<'a> {
    engines: &'a Engines,
    is_inside_trait_constraint: bool,
}

impl<'a> PartialEqWithEnginesContext<'a> {
    pub(crate) fn new(engines: &'a Engines) -> Self {
        Self {
            engines,
            is_inside_trait_constraint: false,
        }
    }

    pub(crate) fn with_is_inside_trait_constraint(&self) -> Self {
        Self {
            is_inside_trait_constraint: true,
            ..*self
        }
    }

    pub(crate) fn engines(&self) -> &Engines {
        self.engines
    }

    pub(crate) fn is_inside_trait_constraint(&self) -> bool {
        self.is_inside_trait_constraint
    }
}

pub trait PartialEqWithEngines {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool;
}

pub struct OrdWithEnginesContext<'a> {
    engines: &'a Engines,
    is_inside_trait_constraint: bool,
}

impl<'a> OrdWithEnginesContext<'a> {
    pub(crate) fn new(engines: &'a Engines) -> Self {
        Self {
            engines,
            is_inside_trait_constraint: false,
        }
    }

    pub(crate) fn with_is_inside_trait_constraint(&self) -> Self {
        Self {
            is_inside_trait_constraint: true,
            ..*self
        }
    }

    pub(crate) fn engines(&self) -> &Engines {
        self.engines
    }

    pub(crate) fn is_inside_trait_constraint(&self) -> bool {
        self.is_inside_trait_constraint
    }
}

pub trait OrdWithEngines {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering;
}

impl<T: EqWithEngines + ?Sized> EqWithEngines for &T {}
impl<T: PartialEqWithEngines + ?Sized> PartialEqWithEngines for &T {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        (*self).eq(*other, ctx)
    }
}
impl<T: OrdWithEngines + ?Sized> OrdWithEngines for &T {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
        (*self).cmp(*other, ctx)
    }
}

impl<T: OrdWithEngines> OrdWithEngines for Option<T> {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
        match (self, other) {
            (Some(x), Some(y)) => x.cmp(y, ctx),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    }
}

impl<T: OrdWithEngines> OrdWithEngines for Box<T> {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
        (**self).cmp(&(**other), ctx)
    }
}

impl<T: EqWithEngines> EqWithEngines for Option<T> {}
impl<T: PartialEqWithEngines> PartialEqWithEngines for Option<T> {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        match (self, other) {
            (None, None) => true,
            (Some(x), Some(y)) => x.eq(y, ctx),
            _ => false,
        }
    }
}

impl<T: EqWithEngines> EqWithEngines for Box<T> {}
impl<T: PartialEqWithEngines> PartialEqWithEngines for Box<T> {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        (**self).eq(&(**other), ctx)
    }
}

impl<T: EqWithEngines> EqWithEngines for [T] {}
impl<T: PartialEqWithEngines> PartialEqWithEngines for [T] {
    fn eq(&self, other: &Self, ctx: &PartialEqWithEnginesContext) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(x, y)| x.eq(y, ctx))
    }
}
impl<T: OrdWithEngines> OrdWithEngines for [T] {
    fn cmp(&self, other: &Self, ctx: &OrdWithEnginesContext) -> Ordering {
        self.iter()
            .zip(other.iter())
            .map(|(x, y)| x.cmp(y, ctx))
            .find(|o| o.is_ne())
            .unwrap_or_else(|| self.len().cmp(&other.len()))
    }
}

pub(crate) fn make_hasher<'a: 'b, 'b, K>(
    hash_builder: &'a impl BuildHasher,
    engines: &'b Engines,
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

pub trait SpannedWithEngines {
    fn span(&self, engines: &Engines) -> Span;
}
