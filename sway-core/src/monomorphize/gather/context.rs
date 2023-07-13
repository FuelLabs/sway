use hashbrown::{hash_map::RawEntryMut, HashMap};
use std::sync::RwLock;

use crate::{engine_threading::*, monomorphize::priv_prelude::*, namespace};

/// Contextual state tracked and accumulated throughout gathering the trait
/// constraints.
pub(crate) struct GatherContext<'a> {
    /// The namespace context accumulated throughout type-checking.
    pub(crate) namespace: &'a GatherNamespace<'a>,

    pub(crate) engines: &'a Engines,

    /// The list of constraints.
    /// NOTE: This only needs to be a [HashSet][hashbrown::HashSet], but there
    /// isn't the right method implement on that data type for what we need, so
    /// instead we use a dummy [HashMap][hashbrown::HashMap].
    constraints: &'a RwLock<HashMap<Constraint, usize>>,
}

impl<'a> GatherContext<'a> {
    /// Initialize a context at the top-level of a module with its namespace.
    pub(crate) fn from_root(
        root_namespace: &'a GatherNamespace<'a>,
        engines: &'a Engines,
        constraints: &'a RwLock<HashMap<Constraint, usize>>,
    ) -> GatherContext<'a> {
        Self::from_module_namespace(root_namespace, engines, constraints)
    }

    fn from_module_namespace(
        namespace: &'a GatherNamespace<'a>,
        engines: &'a Engines,
        constraints: &'a RwLock<HashMap<Constraint, usize>>,
    ) -> Self {
        Self {
            namespace,
            engines,
            constraints,
        }
    }

    /// Create a new context that mutably borrows the inner [Namespace] with a
    /// lifetime bound by `self`.
    pub(crate) fn by_ref(&mut self) -> GatherContext<'_> {
        GatherContext {
            namespace: self.namespace,
            engines: self.engines,
            constraints: self.constraints,
        }
    }

    /// Scope the [Context] with the given [Namespace].
    pub(crate) fn scoped(self, namespace: &'a GatherNamespace<'a>) -> GatherContext<'a> {
        GatherContext {
            namespace,
            engines: self.engines,
            constraints: self.constraints,
        }
    }

    pub(crate) fn add_constraint(&self, constraint: Constraint) {
        let mut constraints = self.constraints.write().unwrap();
        let hash_builder = constraints.hasher().clone();
        let constraint_hash = make_hasher(&hash_builder, self.engines)(&constraint);
        let raw_entry = constraints
            .raw_entry_mut()
            .from_hash(constraint_hash, |x| x.eq(&constraint, self.engines));
        if let RawEntryMut::Vacant(v) = raw_entry {
            v.insert_with_hasher(
                constraint_hash,
                constraint,
                0,
                make_hasher(&hash_builder, self.engines),
            );
        }
    }
}

/// The set of items that represent the namespace context passed throughout
/// gathering the trait constraints.
#[derive(Debug)]
pub(crate) struct GatherNamespace<'a> {
    /// The `root` of the project namespace.
    pub(crate) root: &'a namespace::Module,

    /// An absolute path from the `root` that represents the current module
    /// being gathered.
    pub(crate) mod_path: PathBuf,
}

impl<'a> GatherNamespace<'a> {
    /// Initialize the namespace at its root from the given initial namespace.
    pub(crate) fn init_root(root: &'a namespace::Module) -> GatherNamespace<'a> {
        let mod_path = vec![];
        Self { root, mod_path }
    }

    pub(crate) fn new_with_module(&self, module: &namespace::Module) -> GatherNamespace<'_> {
        let mut mod_path = self.mod_path.clone();
        if let Some(name) = &module.name {
            mod_path.push(name.clone());
        }
        GatherNamespace {
            root: self.root,
            mod_path,
        }
    }

    /// Access to the current [Module], i.e. the module at the inner `mod_path`.
    pub(crate) fn module(&self) -> &namespace::Module {
        &self.root[&self.mod_path]
    }
}

impl<'a> std::ops::Deref for GatherNamespace<'a> {
    type Target = namespace::Module;
    fn deref(&self) -> &Self::Target {
        self.module()
    }
}
