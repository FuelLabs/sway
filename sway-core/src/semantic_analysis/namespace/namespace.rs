use crate::{
    declaration_engine::{de_get_function, DeclarationId},
    engine_threading::*,
    error::*,
    language::{ty, CallPath},
    type_system::*,
    CompileResult, Ident,
};

use super::{module::Module, root::Root, submodule_namespace::SubmoduleNamespace, Path, PathBuf};

use sway_error::error::CompileError;
use sway_types::{span::Span, Spanned};

use std::collections::VecDeque;

/// The set of items that represent the namespace context passed throughout type checking.
#[derive(Clone, Debug)]
pub struct Namespace {
    /// An immutable namespace that consists of the names that should always be present, no matter
    /// what module or scope we are currently checking.
    ///
    /// These include external library dependencies and (when it's added) the `std` prelude.
    ///
    /// This is passed through type-checking in order to initialise the namespace of each submodule
    /// within the project.
    init: Module,
    /// The `root` of the project namespace.
    ///
    /// From the root, the entirety of the project's namespace can always be accessed.
    ///
    /// The root is initialised from the `init` namespace before type-checking begins.
    pub(crate) root: Root,
    /// An absolute path from the `root` that represents the current module being checked.
    ///
    /// E.g. when type-checking the root module, this is equal to `[]`. When type-checking a
    /// submodule of the root called "foo", this would be equal to `[foo]`.
    pub(crate) mod_path: PathBuf,
}

impl Namespace {
    /// Initialise the namespace at its root from the given initial namespace.
    pub fn init_root(init: Module) -> Self {
        let root = Root::from(init.clone());
        let mod_path = vec![];
        Self {
            init,
            root,
            mod_path,
        }
    }

    /// A reference to the path of the module currently being type-checked.
    pub fn mod_path(&self) -> &Path {
        &self.mod_path
    }

    /// Find the module that these prefixes point to
    pub fn find_module_path<'a>(
        &'a self,
        prefixes: impl IntoIterator<Item = &'a Ident>,
    ) -> PathBuf {
        self.mod_path.iter().chain(prefixes).cloned().collect()
    }

    /// A reference to the root of the project namespace.
    pub fn root(&self) -> &Root {
        &self.root
    }

    /// A mutable reference to the root of the project namespace.
    pub fn root_mut(&mut self) -> &mut Root {
        &mut self.root
    }

    /// Access to the current [Module], i.e. the module at the inner `mod_path`.
    ///
    /// Note that the [Namespace] will automatically dereference to this [Module] when attempting
    /// to call any [Module] methods.
    pub fn module(&self) -> &Module {
        &self.root.module[&self.mod_path]
    }

    /// Mutable access to the current [Module], i.e. the module at the inner `mod_path`.
    ///
    /// Note that the [Namespace] will automatically dereference to this [Module] when attempting
    /// to call any [Module] methods.
    pub fn module_mut(&mut self) -> &mut Module {
        &mut self.root.module[&self.mod_path]
    }

    /// Short-hand for calling [Root::resolve_symbol] on `root` with the `mod_path`.
    pub(crate) fn resolve_symbol(&self, symbol: &Ident) -> CompileResult<&ty::TyDeclaration> {
        self.root.resolve_symbol(&self.mod_path, symbol)
    }

    /// Short-hand for calling [Root::resolve_call_path] on `root` with the `mod_path`.
    pub(crate) fn resolve_call_path(
        &self,
        call_path: &CallPath,
    ) -> CompileResult<&ty::TyDeclaration> {
        self.root.resolve_call_path(&self.mod_path, call_path)
    }

    /// Short-hand for calling [Root::resolve_type_with_self] on `root` with the `mod_path`.
    pub(crate) fn resolve_type_with_self(
        &mut self,
        engines: Engines<'_>,
        type_id: TypeId,
        self_type: TypeId,
        span: &Span,
        enforce_type_arguments: EnforceTypeArguments,
        type_info_prefix: Option<&Path>,
    ) -> CompileResult<TypeId> {
        let mod_path = self.mod_path.clone();
        engines.te().resolve_type_with_self(
            engines.de(),
            type_id,
            self_type,
            span,
            enforce_type_arguments,
            type_info_prefix,
            self,
            &mod_path,
        )
    }

    /// Short-hand for calling [Root::resolve_type_without_self] on `root` and with the `mod_path`.
    pub(crate) fn resolve_type_without_self(
        &mut self,
        engines: Engines<'_>,
        type_id: TypeId,
        span: &Span,
        type_info_prefix: Option<&Path>,
    ) -> CompileResult<TypeId> {
        let mod_path = self.mod_path.clone();
        engines.te().resolve_type(
            engines.de(),
            type_id,
            span,
            EnforceTypeArguments::Yes,
            type_info_prefix,
            self,
            &mod_path,
        )
    }

    /// Given a method and a type (plus a `self_type` to potentially
    /// resolve it), find that method in the namespace. Requires `args_buf`
    /// because of some special casing for the standard library where we pull
    /// the type from the arguments buffer.
    ///
    /// This function will generate a missing method error if the method is not
    /// found.
    pub(crate) fn find_method_for_type(
        &mut self,
        mut type_id: TypeId,
        method_prefix: &Path,
        method_name: &Ident,
        self_type: TypeId,
        args_buf: &VecDeque<ty::TyExpression>,
        engines: Engines<'_>,
    ) -> CompileResult<DeclarationId> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = engines.te();

        // grab the local module
        let local_module = check!(
            self.root().check_submodule(&self.mod_path),
            return err(warnings, errors),
            warnings,
            errors
        );

        // grab the local methods from the local module
        let local_methods = local_module.get_methods_for_type(type_engine, type_id);

        type_id.replace_self_type(engines, self_type);

        // resolve the type
        let type_id = check!(
            type_engine.resolve_type(
                engines.de(),
                type_id,
                &method_name.span(),
                EnforceTypeArguments::No,
                None,
                self,
                method_prefix
            ),
            type_engine.insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors
        );

        // grab the module where the type itself is declared
        let type_module = check!(
            self.root().check_submodule(method_prefix),
            return err(warnings, errors),
            warnings,
            errors
        );

        // grab the methods from where the type is declared
        let mut type_methods = type_module.get_methods_for_type(type_engine, type_id);

        let mut methods = local_methods;
        methods.append(&mut type_methods);

        for decl_id in methods.into_iter() {
            let method = check!(
                CompileResult::from(de_get_function(decl_id.clone(), &decl_id.span())),
                return err(warnings, errors),
                warnings,
                errors
            );
            if &method.name == method_name {
                // if we find the method that we are looking for, we also need
                // to retrieve the impl definitions for the return type so that
                // the user can string together method calls
                self.insert_trait_implementation_for_type(engines, method.return_type);
                return ok(decl_id, warnings, errors);
            }
        }

        if !args_buf
            .get(0)
            .map(|x| type_engine.look_up_type_id(x.return_type))
            .eq(&Some(TypeInfo::ErrorRecovery), type_engine)
        {
            errors.push(CompileError::MethodNotFound {
                method_name: method_name.clone(),
                type_name: type_engine.help_out(type_id).to_string(),
                span: method_name.span(),
            });
        }
        err(warnings, errors)
    }

    /// Short-hand for performing a [Module::star_import] with `mod_path` as the destination.
    pub(crate) fn star_import(&mut self, src: &Path, engines: Engines<'_>) -> CompileResult<()> {
        self.root.star_import(src, &self.mod_path, engines)
    }

    /// Short-hand for performing a [Module::self_import] with `mod_path` as the destination.
    pub(crate) fn self_import(
        &mut self,
        engines: Engines<'_>,
        src: &Path,
        alias: Option<Ident>,
    ) -> CompileResult<()> {
        self.root.self_import(engines, src, &self.mod_path, alias)
    }

    /// Short-hand for performing a [Module::item_import] with `mod_path` as the destination.
    pub(crate) fn item_import(
        &mut self,
        engines: Engines<'_>,
        src: &Path,
        item: &Ident,
        alias: Option<Ident>,
    ) -> CompileResult<()> {
        self.root
            .item_import(engines, src, item, &self.mod_path, alias)
    }

    /// "Enter" the submodule at the given path by returning a new [SubmoduleNamespace].
    ///
    /// Here we temporarily change `mod_path` to the given `dep_mod_path` and wrap `self` in a
    /// [SubmoduleNamespace] type. When dropped, the [SubmoduleNamespace] resets the `mod_path`
    /// back to the original path so that we can continue type-checking the current module after
    /// finishing with the dependency.
    pub(crate) fn enter_submodule(&mut self, dep_name: Ident) -> SubmoduleNamespace {
        let init = self.init.clone();
        self.submodules.entry(dep_name.to_string()).or_insert(init);
        let submod_path: Vec<_> = self
            .mod_path
            .iter()
            .cloned()
            .chain(Some(dep_name))
            .collect();
        let parent_mod_path = std::mem::replace(&mut self.mod_path, submod_path);
        SubmoduleNamespace {
            namespace: self,
            parent_mod_path,
        }
    }
}

impl std::ops::Deref for Namespace {
    type Target = Module;
    fn deref(&self) -> &Self::Target {
        self.module()
    }
}

impl std::ops::DerefMut for Namespace {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.module_mut()
    }
}
