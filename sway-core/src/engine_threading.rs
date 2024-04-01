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
use sway_types::SourceEngine;

#[derive(Clone, Debug)]
pub struct Engines {
    type_engine: TypeEngine,
    decl_engine: DeclEngine,
    parsed_decl_engine: ParsedDeclEngine,
    query_engine: QueryEngine,
    source_engine: SourceEngine,
}

impl Default for Engines {
    fn default() -> Self {
        let engines = Self {
            type_engine: Default::default(),
            decl_engine: Default::default(),
            parsed_decl_engine: Default::default(),
            query_engine: Default::default(),
            source_engine: Default::default(),
        };
        engines.te().init(&engines);
        engines
    }
}

impl Engines {
    // pub fn new(
    //     type_engine: TypeEngine,
    //     decl_engine: DeclEngine,
    //     parsed_decl_engine: ParsedDeclEngine,
    //     query_engine: QueryEngine,
    //     source_engine: SourceEngine,
    // ) -> Engines {
    //     Engines {
    //         type_engine,
    //         decl_engine,
    //         parsed_decl_engine,
    //         query_engine,
    //         source_engine,
    //     }
    // }

    pub fn print_stats(&self) {
        // println!("Engine Stats");
        // println!("------------");

        // println!("    Type Engine");
        // println!(
        //     "        Slab: {} items ({})",
        //     self.type_engine.slab.len(),
        //     human_format::Formatter::new().format(self.type_engine.slab.len() as f64)
        // );
        // let size = self.type_engine.slab.deep_size_of();
        // println!(
        //     "        Slab Size: {} bytes ({})",
        //     size,
        //     human_bytes::human_bytes(size as f64)
        // );

        // println!("    Decl Engine");
        // println!(
        //     "        Function Decl Slab: {} items ({})",
        //     self.decl_engine.function_slab.len(),
        //     human_format::Formatter::new().format(self.decl_engine.function_slab.len() as f64)
        // );
        // let size = self.decl_engine.function_slab.deep_size_of();
        // println!(
        //     "        Function Decl Slab: {} bytes ({})",
        //     size,
        //     human_bytes::human_bytes(size as f64)
        // );

        // Count by name
        // let items = self.decl_engine.function_slab.inner.read().unwrap();
        // let map = items.items.iter().filter_map(|x| x.clone()).fold(
        //     hashbrown::HashMap::<String, usize>::new(),
        //     |mut map, item| {
        //         *(map.entry(item.name.as_str().to_string()).or_default()) += 1;
        //         map
        //     },
        // );
        // let mut map = map.into_iter().collect::<Vec<_>>();
        // map.sort_by(|a, b| a.1.cmp(&b.1));
        // for (k, v) in map {
        //     println!("{} -> {}", k, v);
        // }
    }

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

    /// Removes all data associated with `module_id` from the declaration and type engines.
    /// It is intended to be used during garbage collection to remove any data that is no longer needed.
    pub fn clear_module(&mut self, module_id: &sway_types::ModuleId) {
        self.type_engine.clear_module(module_id);
        self.decl_engine.clear_module(module_id);
        self.parsed_decl_engine.clear_module(module_id);
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
        self.thing.eq(&rhs.thing, self.engines)
    }
}

impl<T: EqWithEngines> Eq for WithEngines<'_, T> {}

impl<T: OrdWithEngines> PartialOrd for WithEngines<'_, T>
where
    T: PartialEqWithEngines,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.thing.cmp(&other.thing, self.engines))
    }
}

impl<T: OrdWithEngines> Ord for WithEngines<'_, T>
where
    T: EqWithEngines,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.thing.cmp(&other.thing, self.engines)
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

pub trait PartialEqWithEngines {
    fn eq(&self, other: &Self, engines: &Engines) -> bool;
}

pub trait OrdWithEngines {
    fn cmp(&self, other: &Self, engines: &Engines) -> Ordering;
}

impl<T: EqWithEngines + ?Sized> EqWithEngines for &T {}
impl<T: PartialEqWithEngines + ?Sized> PartialEqWithEngines for &T {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        (*self).eq(*other, engines)
    }
}
impl<T: OrdWithEngines + ?Sized> OrdWithEngines for &T {
    fn cmp(&self, other: &Self, engines: &Engines) -> Ordering {
        (*self).cmp(*other, engines)
    }
}

impl<T: OrdWithEngines> OrdWithEngines for Option<T> {
    fn cmp(&self, other: &Self, engines: &Engines) -> Ordering {
        match (self, other) {
            (Some(x), Some(y)) => x.cmp(y, engines),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    }
}

impl<T: OrdWithEngines> OrdWithEngines for Box<T> {
    fn cmp(&self, other: &Self, engines: &Engines) -> Ordering {
        (**self).cmp(&(**other), engines)
    }
}

impl<T: EqWithEngines> EqWithEngines for Option<T> {}
impl<T: PartialEqWithEngines> PartialEqWithEngines for Option<T> {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        match (self, other) {
            (None, None) => true,
            (Some(x), Some(y)) => x.eq(y, engines),
            _ => false,
        }
    }
}

impl<T: EqWithEngines> EqWithEngines for Box<T> {}
impl<T: PartialEqWithEngines> PartialEqWithEngines for Box<T> {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        (**self).eq(&(**other), engines)
    }
}

impl<T: EqWithEngines> EqWithEngines for [T] {}
impl<T: PartialEqWithEngines> PartialEqWithEngines for [T] {
    fn eq(&self, other: &Self, engines: &Engines) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(x, y)| x.eq(y, engines))
    }
}
impl<T: OrdWithEngines> OrdWithEngines for [T] {
    fn cmp(&self, other: &Self, engines: &Engines) -> Ordering {
        self.iter()
            .zip(other.iter())
            .map(|(x, y)| x.cmp(y, engines))
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
