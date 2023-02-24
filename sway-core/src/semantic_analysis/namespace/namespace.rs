use crate::{
    decl_engine::DeclRef,
    engine_threading::*,
    error::*,
    language::{ty, CallPath},
    type_system::*,
    CompileResult, Ident,
};

use super::{module::Module, root::Root, submodule_namespace::SubmoduleNamespace, Path, PathBuf};

use hashbrown::{hash_map::RawEntryMut, HashMap};
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
        engines.te().resolve_with_self(
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
        engines.te().resolve(
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
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn find_method_for_type(
        &mut self,
        mut type_id: TypeId,
        method_prefix: &Path,
        method_name: &Ident,
        self_type: TypeId,
        method_may_have_self_type: bool,
        args_buf: &VecDeque<ty::TyExpression>,
        engines: Engines<'_>,
    ) -> CompileResult<DeclRef> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = engines.te();
        let decl_engine = engines.de();

        // If the type that we are looking for is the error recovery type, then
        // we want to return the error case without creating a new error
        // message.
        if let TypeInfo::ErrorRecovery = type_engine.get(type_id) {
            return err(warnings, errors);
        }

        // Replace with the "self" type if necessary.
        type_id.replace_self_type(engines, self_type);

        // resolve the type
        let type_id = check!(
            type_engine.resolve(
                decl_engine,
                type_id,
                &method_name.span(),
                EnforceTypeArguments::No,
                None,
                self,
                method_prefix
            ),
            type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
            warnings,
            errors
        );

        // grab the local module
        let local_module = check!(
            self.root().check_submodule(&self.mod_path),
            return err(warnings, errors),
            warnings,
            errors
        );

        // grab the local methods from the local module
        let methods = dedup_methods(engines, local_module.get_methods_for_type(engines, type_id));

        // println!("# methods: {}", methods.len());

        // Filter the list of methods into a list of possible methods.
        let unify_checker = UnifyCheck::new(engines);
        let possible_methods: Vec<DeclRef> = methods
            .into_iter()
            .filter(|decl_ref| {
                let method = check!(
                    CompileResult::from(decl_engine.get_function(decl_ref, &decl_ref.span())),
                    return false,
                    warnings,
                    errors
                );
                let is_first_param_self = method
                    .parameters
                    .get(0)
                    .map(|f| f.is_self())
                    .unwrap_or(false);
                let args_buf_to_check = if method_may_have_self_type && !is_first_param_self {
                    args_buf.clone().split_off(1)
                } else {
                    args_buf.clone()
                };
                // println!(
                //     "{}, [{}]",
                //     method.name,
                //     method
                //         .parameters
                //         .iter()
                //         .map(|p| {
                //             format!(
                //                 "{}: {}",
                //                 p.name,
                //                 engines.help_out(type_engine.get(p.type_argument.type_id))
                //             )
                //         })
                //         .collect::<Vec<_>>()
                //         .join(", ")
                // );
                &method.name == method_name
                    && args_buf_to_check.len() == method.parameters.len()
                    && args_buf_to_check
                        .iter()
                        .zip(method.parameters.iter())
                        .all(|(a, p)| unify_checker.check(a.return_type, p.type_argument.type_id))
            })
            .collect();

        // println!("# possible_methods: {}", possible_methods.len());
        // println!(
        //     "args_buf: {}",
        //     args_buf
        //         .iter()
        //         .map(|p| engines.help_out(type_engine.get(p.return_type)).to_string())
        //         .collect::<Vec<_>>()
        //         .join(", ")
        // );

        // Given the list of possible methods, determine if there is one, zero,
        // or multiple matches.
        let matching_method = match possible_methods.get(0) {
            Some(matching_method) if possible_methods.len() == 1 => matching_method,
            Some(_) => {
                // Case where multiple methods exist with the same name.
                // This is the case of https://github.com/FuelLabs/sway/issues/3633
                // where multiple generic trait impls use the same method name
                // but with different parameter types.
                errors.push(CompileError::MultipleMethodsPossible {
                    method_name: method_name.clone(),
                    span: method_name.span(),
                });
                return err(warnings, errors);
            }
            None => {
                errors.push(CompileError::MethodNotFound {
                    method_name: method_name.clone(),
                    type_name: engines.help_out(type_id).to_string(),
                    span: method_name.span(),
                });
                return err(warnings, errors);
            }
        };

        if errors.is_empty() {
            ok(matching_method.clone(), warnings, errors)
        } else {
            err(warnings, errors)
        }
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
            .chain(Some(dep_name.clone()))
            .collect();
        let parent_mod_path = std::mem::replace(&mut self.mod_path, submod_path);
        self.name = Some(dep_name);
        SubmoduleNamespace {
            namespace: self,
            parent_mod_path,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn insert_trait_implementation(
        &mut self,
        trait_name: CallPath,
        trait_type_args: Vec<TypeArgument>,
        type_id: TypeId,
        items: &[ty::TyImplItem],
        impl_span: &Span,
        is_impl_self: bool,
        engines: Engines<'_>,
    ) -> CompileResult<()> {
        // Use trait name with full path, improves consistency between
        // this inserting and getting in `get_methods_for_type_and_trait_name`.
        let full_trait_name = trait_name.to_fullpath(self);

        self.implemented_traits.insert(
            full_trait_name,
            trait_type_args,
            type_id,
            items,
            impl_span,
            is_impl_self,
            engines,
        )
    }

    pub(crate) fn get_methods_for_type_and_trait_name(
        &mut self,
        engines: Engines<'_>,
        type_id: TypeId,
        trait_name: &CallPath,
    ) -> Vec<DeclRef> {
        // Use trait name with full path, improves consistency between
        // this get and inserting in `insert_trait_implementation`.
        let trait_name = trait_name.to_fullpath(self);

        self.implemented_traits
            .get_methods_for_type_and_trait_name(engines, type_id, &trait_name)
    }
}

fn dedup_methods(engines: Engines<'_>, methods: Vec<DeclRef>) -> Vec<DeclRef> {
    let mut hashed_methods: HashMap<DeclRef, bool> = HashMap::new();
    for decl_ref in methods.into_iter() {
        let hash_builder = hashed_methods.hasher().clone();
        let hasher = make_hasher(&hash_builder, engines)(&decl_ref);
        match hashed_methods
            .raw_entry_mut()
            .from_hash(hasher, |x| x.eq(&decl_ref, engines))
        {
            RawEntryMut::Occupied(_) => {}
            RawEntryMut::Vacant(v) => {
                v.insert_with_hasher(hasher, decl_ref, true, make_hasher(&hash_builder, engines));
            }
        }
    }
    hashed_methods.keys().cloned().into_iter().collect()
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
