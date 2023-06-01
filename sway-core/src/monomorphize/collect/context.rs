use hashbrown::{hash_map::RawEntryMut, HashMap};
use std::sync::RwLock;

use crate::{decl_engine::*, engine_threading::*, monomorphize::priv_prelude::*, TypeEngine};

/// Contextual state tracked and accumulated throughout collecting information
/// for monomorphization.
#[derive(Clone, Copy)]
pub(crate) struct CollectContext<'a> {
    /// The type engine storing types.
    pub(crate) type_engine: &'a TypeEngine,

    /// The declaration engine holds declarations.
    pub(crate) decl_engine: &'a DeclEngine,

    /// The list of constraints.
    /// NOTE: This only needs to be a [HashSet][hashbrown::HashSet], but there
    /// isn't the right method implement on that data type for what we need, so
    /// instead we use a dummy [HashMap][hashbrown::HashMap].
    constraints: &'a RwLock<HashMap<Constraint, usize>>,
}

impl<'a> CollectContext<'a> {
    /// Initialize a context.
    pub(crate) fn new(
        engines: Engines<'a>,
        constraints: &'a RwLock<HashMap<Constraint, usize>>,
    ) -> CollectContext<'a> {
        let (type_engine, decl_engine) = engines.unwrap();
        Self {
            type_engine,
            decl_engine,
            constraints,
        }
    }

    pub(crate) fn add_constraint(&self, constraint: Constraint) {
        let engines = Engines::new(self.type_engine, self.decl_engine);
        let mut constraints = self.constraints.write().unwrap();
        let hash_builder = constraints.hasher().clone();
        let constraint_hash = make_hasher(&hash_builder, engines)(&constraint);
        let raw_entry = constraints
            .raw_entry_mut()
            .from_hash(constraint_hash, |x| x.eq(&constraint, engines));
        if let RawEntryMut::Vacant(v) = raw_entry {
            v.insert_with_hasher(
                constraint_hash,
                constraint,
                0,
                make_hasher(&hash_builder, engines),
            );
        }
    }
}
