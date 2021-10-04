use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) struct Module(pub(crate) generational_arena::Index);

pub(crate) struct ModuleContent {
    pub(crate) name: String,
    pub(crate) kind: Kind,
    pub(crate) functions: Vec<Function>,
}

#[derive(Debug)]
pub(crate) enum Kind {
    _Contract,
    _Library,
    _Predicate,
    Script,
}

impl Module {
    pub(crate) fn new(context: &mut Context, kind: Kind, name: &str) -> Module {
        let content = ModuleContent {
            name: name.to_owned(),
            kind,
            functions: Vec::new(),
        };
        Module(context.modules.insert(content))
    }

    pub(crate) fn function_iter(&self, context: &Context) -> FunctionIterator {
        FunctionIterator::new(context, self)
    }
}

pub(crate) struct ModuleIterator {
    modules: Vec<generational_arena::Index>,
    next: usize,
}

impl ModuleIterator {
    pub(crate) fn new(context: &Context) -> ModuleIterator {
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
