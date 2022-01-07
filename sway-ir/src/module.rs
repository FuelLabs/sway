use std::collections::HashMap;

use crate::{
    context::Context,
    function::{Function, FunctionIterator},
    value::Value,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Module(pub generational_arena::Index);

pub struct ModuleContent {
    pub name: String,
    pub kind: Kind,
    pub functions: Vec<Function>,
    pub globals: HashMap<String, Value>,
}

#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Contract,
    Library,
    Predicate,
    Script,
}

impl Module {
    pub fn new(context: &mut Context, kind: Kind, name: &str) -> Module {
        let content = ModuleContent {
            name: name.to_owned(),
            kind,
            functions: Vec::new(),
            globals: HashMap::new(),
        };
        Module(context.modules.insert(content))
    }

    pub fn get_kind(&self, context: &Context) -> Kind {
        context.modules[self.0].kind
    }

    pub fn function_iter(&self, context: &Context) -> FunctionIterator {
        FunctionIterator::new(context, self)
    }

    pub fn add_global_constant(&self, context: &mut Context, name: String, const_val: Value) {
        context.modules[self.0].globals.insert(name, const_val);
    }

    pub fn get_global_constant(&self, context: &Context, name: &str) -> Option<Value> {
        context.modules[self.0].globals.get(name).copied()
    }
}

pub struct ModuleIterator {
    modules: Vec<generational_arena::Index>,
    next: usize,
}

impl ModuleIterator {
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
