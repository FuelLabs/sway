//! A scope containing a collection of [`Function`]s and constant values.
//!
//! A module also has a 'kind' corresponding to the different Sway module types.

use std::collections::{HashMap, BTreeMap};

use crate::{
    context::Context,
    function::{Function, FunctionIterator},
    value::Value,
};

/// A wrapper around an [ECS](https://github.com/fitzgen/generational-arena) handle into the
/// [`Context`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Module(pub generational_arena::Index);

#[doc(hidden)]
pub struct ModuleContent {
    pub kind: Kind,
    pub functions: Vec<Function>,
    pub global_constants: HashMap<String, Value>,
    pub global_configurable: BTreeMap<String, Value>,
}

/// The different 'kinds' of Sway module: `Contract`, `Library`, `Predicate` or `Script`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    Contract,
    Library,
    Predicate,
    Script,
}

impl Module {
    /// Return a new module of a specific kind.
    pub fn new(context: &mut Context, kind: Kind) -> Module {
        let content = ModuleContent {
            kind,
            functions: Vec::new(),
            global_constants: HashMap::new(),
            global_configurable: BTreeMap::new(),
        };
        Module(context.modules.insert(content))
    }

    /// Get this module's [`Kind`].
    pub fn get_kind(&self, context: &Context) -> Kind {
        context.modules[self.0].kind
    }

    /// Return an interator over each of the [`Function`]s in this module.
    pub fn function_iter(&self, context: &Context) -> FunctionIterator {
        FunctionIterator::new(context, self)
    }

    /// Add a global value to this module.
    pub fn add_global_constant(&self, context: &mut Context, name: String, const_val: Value) {
        context.modules[self.0]
            .global_constants
            .insert(name, const_val);
    }

    /// Get a named global value from this module, if found.
    pub fn get_global_constant(&self, context: &Context, name: &str) -> Option<Value> {
        context.modules[self.0].global_constants.get(name).copied()
    }

    /// Add a global value to this module.
    pub fn add_global_configurable(&self, context: &mut Context, name: String, config_val: Value) {
        context.modules[self.0]
            .global_configurable
            .insert(name, config_val);
    }

    /// Get a named global value from this module, if found.
    pub fn get_global_configurable(&self, context: &Context, name: &str) -> Option<Value> {
        context.modules[self.0]
            .global_configurable
            .get(name)
            .copied()
    }

    /// Removed a function from the module.  Returns true if function was found and removed.
    ///
    /// **Use with care!  Be sure the function is not an entry point nor called at any stage.**
    pub fn remove_function(&self, context: &mut Context, function: &Function) {
        context
            .modules
            .get_mut(self.0)
            .expect("Module must exist in context.")
            .functions
            .retain(|mod_fn| mod_fn != function);
    }
}

/// An iterator over [`Module`]s within a [`Context`].
pub struct ModuleIterator {
    modules: Vec<generational_arena::Index>,
    next: usize,
}

impl ModuleIterator {
    /// Return a new [`Module`] iterator.
    pub fn new(context: &Context) -> ModuleIterator {
        // Copy all the current modules indices, so they may be modified in the context during
        // iteration.
        ModuleIterator {
            modules: context.modules.iter().map(|pair| pair.0).collect(),
            next: 0,
        }
    }
}

impl Iterator for ModuleIterator {
    type Item = Module;

    fn next(&mut self) -> Option<Module> {
        if self.next < self.modules.len() {
            let idx = self.next;
            self.next += 1;
            Some(Module(self.modules[idx]))
        } else {
            None
        }
    }
}
