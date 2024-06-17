//! A scope containing a collection of [`Function`]s and constant values.
//!
//! A module also has a 'kind' corresponding to the different Sway module types.

use std::collections::{BTreeMap, HashMap};

use crate::{
    context::Context,
    function::{Function, FunctionIterator},
    value::Value,
    Constant, MetadataIndex, Type,
};

/// A wrapper around an [ECS](https://github.com/orlp/slotmap) handle into the
/// [`Context`].
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Module(pub slotmap::DefaultKey);

#[doc(hidden)]
pub struct ModuleContent {
    pub kind: Kind,
    pub functions: Vec<Function>,
    pub global_constants: HashMap<Vec<String>, Value>,
    pub configs: BTreeMap<String, ConfigContent>,
}

#[derive(Clone, Debug)]
pub enum ConfigContent {
    V0 {
        name: String,
        ty: Type,
        ptr_ty: Type,
        constant: Constant,
        opt_metadata: Option<MetadataIndex>,
    },
    V1 {
        name: String,
        ty: Type,
        ptr_ty: Type,
        encoded_bytes: Vec<u8>,
        decode_fn: Function,
        opt_metadata: Option<MetadataIndex>,
    },
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
            configs: BTreeMap::new(),
        };
        Module(context.modules.insert(content))
    }

    /// Get this module's [`Kind`].
    pub fn get_kind(&self, context: &Context) -> Kind {
        context.modules[self.0].kind
    }

    /// Return an iterator over each of the [`Function`]s in this module.
    pub fn function_iter(&self, context: &Context) -> FunctionIterator {
        FunctionIterator::new(context, self)
    }

    /// Add a global constant value to this module.
    pub fn add_global_constant(
        &self,
        context: &mut Context,
        call_path: Vec<String>,
        const_val: Value,
    ) {
        context.modules[self.0]
            .global_constants
            .insert(call_path, const_val);
    }

    /// Get a named global constant value from this module, if found.
    pub fn get_global_constant(&self, context: &Context, call_path: &Vec<String>) -> Option<Value> {
        context.modules[self.0]
            .global_constants
            .get(call_path)
            .copied()
    }

    /// Add a config value to this module.
    pub fn add_config(&self, context: &mut Context, name: String, content: ConfigContent) {
        context.modules[self.0].configs.insert(name, content);
    }

    /// Get a named config content from this module, if found.
    pub fn get_config<'a>(&self, context: &'a Context, name: &str) -> Option<&'a ConfigContent> {
        context.modules[self.0].configs.get(name)
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

    pub fn iter_configs<'a>(
        &'a self,
        context: &'a Context,
    ) -> impl Iterator<Item = &ConfigContent> + 'a {
        context.modules[self.0].configs.values()
    }
}

/// An iterator over [`Module`]s within a [`Context`].
pub struct ModuleIterator {
    modules: Vec<slotmap::DefaultKey>,
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
