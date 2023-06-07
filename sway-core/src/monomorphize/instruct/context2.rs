use crate::{decl_engine::*, monomorphize::priv_prelude::*, namespace, Engines, TypeEngine};

/// Contextual state tracked and accumulated throughout applying the
/// monomorphization instructions.
pub(crate) struct InstructContext<'a, 'b: 'a> {
    /// The namespace context accumulated throughout applying the
    /// monomorphization instructions.
    pub(crate) namespace: &'a mut InstructNamespace<'b>,

    /// The type engine storing types.
    pub(crate) type_engine: &'a TypeEngine,

    /// The declaration engine holds declarations.
    pub(crate) decl_engine: &'a DeclEngine,

    /// The list of instructions.
    instructions: &'a [Instruction],
}

impl<'a, 'b> InstructContext<'a, 'b> {
    /// Initialize a context at the top-level of a module with its namespace.
    pub(crate) fn from_root(
        root_namespace: &'a mut InstructNamespace<'b>,
        engines: Engines<'a>,
        instructions: &'a [Instruction],
    ) -> Self {
        Self::from_module_namespace(root_namespace, engines, instructions)
    }

    fn from_module_namespace(
        namespace: &'a mut InstructNamespace<'b>,
        engines: Engines<'a>,
        instructions: &'a [Instruction],
    ) -> Self {
        let (type_engine, decl_engine) = engines.unwrap();
        Self {
            namespace,
            type_engine,
            decl_engine,
            instructions,
        }
    }

    /// Create a new context that mutably borrows the inner [Namespace] with a
    /// lifetime bound by `self`.
    pub(crate) fn by_ref(&mut self) -> InstructContext<'_, 'b> {
        InstructContext {
            namespace: self.namespace,
            type_engine: self.type_engine,
            decl_engine: self.decl_engine,
            instructions: self.instructions,
        }
    }

    /// Scope the [InstructContext] with the given [Namespace].
    pub(crate) fn scoped(
        self,
        namespace: &'a mut InstructNamespace<'b>,
    ) -> InstructContext<'a, 'b> {
        InstructContext {
            namespace,
            type_engine: self.type_engine,
            decl_engine: self.decl_engine,
            instructions: self.instructions,
        }
    }
}

/// The set of items that represent the namespace context passed throughout
/// applying the monomorphization instructions.
pub(crate) struct InstructNamespace<'a> {
    /// An absolute path from the `root` that represents the current module
    /// for which we are applying instructions.
    pub(crate) mod_path: PathBuf,

    /// The `root` of the project namespace.
    pub(crate) root: &'a mut namespace::Module,
}

impl<'a> InstructNamespace<'a> {
    /// Initialize the namespace at its root from the given initial namespace.
    pub(crate) fn init_root(root: &'a mut namespace::Module) -> Self {
        let mod_path = vec![];
        Self { root, mod_path }
    }

    pub(crate) fn new_with_module(
        &mut self,
        module: &mut namespace::Module,
    ) -> InstructNamespace<'_> {
        let mut mod_path = self.mod_path.clone();
        if let Some(name) = &module.name {
            mod_path.push(name.clone());
        }
        InstructNamespace {
            root: self.root,
            mod_path,
        }
    }

    // /// A reference to the path of the module where constraints are currently
    // /// being gathered.
    // pub(crate) fn mod_path(&self) -> &Path {
    //     &self.mod_path
    // }

    /// Access to the current [namespace::Module], i.e. the module at the inner
    /// `mod_path`.
    pub(crate) fn module(&self) -> &namespace::Module {
        &self.root[&self.mod_path]
    }

    /// Mutable access to the current [namespace::Module], i.e. the module at
    /// the inner `mod_path`.
    pub(crate) fn module_mut(&mut self) -> &mut namespace::Module {
        &mut self.root[&self.mod_path]
    }
}

impl<'a> std::ops::Deref for InstructNamespace<'a> {
    type Target = namespace::Module;
    fn deref(&self) -> &Self::Target {
        self.module()
    }
}

impl<'a> std::ops::DerefMut for InstructNamespace<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.module_mut()
    }
}
