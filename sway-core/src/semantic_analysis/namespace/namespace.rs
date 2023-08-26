use crate::{
    decl_engine::{DeclRefConstant, DeclRefFunction},
    engine_threading::*,
    language::{ty, CallPath, Visibility},
    type_system::*,
    Ident,
};

use super::{module::Module, root::Root, submodule_namespace::SubmoduleNamespace, Path, PathBuf};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{span::Span, Spanned};

use std::collections::{HashMap, VecDeque};

/// Enum used to pass a value asking for insertion of type into trait map when an implementation
/// of the trait cannot be found.
pub enum TryInsertingTraitImplOnFailure {
    Yes,
    No,
}

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
    pub(crate) fn resolve_symbol(
        &self,
        handler: &Handler,
        symbol: &Ident,
    ) -> Result<&ty::TyDecl, ErrorEmitted> {
        self.root.resolve_symbol(handler, &self.mod_path, symbol)
    }

    /// Short-hand for calling [Root::resolve_call_path] on `root` with the `mod_path`.
    pub(crate) fn resolve_call_path(
        &self,
        handler: &Handler,
        call_path: &CallPath,
    ) -> Result<&ty::TyDecl, ErrorEmitted> {
        self.root
            .resolve_call_path(handler, &self.mod_path, call_path)
    }

    /// Short-hand for calling [Root::resolve_call_path_with_visibility_check] on `root` with the `mod_path`.
    pub(crate) fn resolve_call_path_with_visibility_check(
        &self,
        handler: &Handler,
        engines: &Engines,
        call_path: &CallPath,
    ) -> Result<&ty::TyDecl, ErrorEmitted> {
        self.root.resolve_call_path_with_visibility_check(
            handler,
            engines,
            &self.mod_path,
            call_path,
        )
    }

    /// Short-hand for calling [Root::resolve_type] on `root` with the `mod_path`.
    #[allow(clippy::too_many_arguments)] // TODO: remove lint bypass once private modules are no longer experimental
    pub(crate) fn resolve_type(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        type_id: TypeId,
        span: &Span,
        enforce_type_arguments: EnforceTypeArguments,
        type_info_prefix: Option<&Path>,
    ) -> Result<TypeId, ErrorEmitted> {
        let mod_path = self.mod_path.clone();
        engines.te().resolve(
            handler,
            engines,
            type_id,
            span,
            enforce_type_arguments,
            type_info_prefix,
            self,
            &mod_path,
        )
    }

    /// Given a name and a type (plus a `self_type` to potentially
    /// resolve it), find items matching in the namespace.
    pub(crate) fn find_items_for_type(
        &mut self,
        handler: &Handler,
        type_id: TypeId,
        item_prefix: &Path,
        item_name: &Ident,
        engines: &Engines,
    ) -> Result<Vec<ty::TyTraitItem>, ErrorEmitted> {
        let type_engine = engines.te();
        let _decl_engine = engines.de();

        // If the type that we are looking for is the error recovery type, then
        // we want to return the error case without creating a new error
        // message.
        if let TypeInfo::ErrorRecovery(err) = type_engine.get(type_id) {
            return Err(err);
        }

        // grab the local module
        let local_module = self.root().check_submodule(handler, &self.mod_path)?;

        // grab the local items from the local module
        let local_items = local_module.get_items_for_type(engines, type_id);

        // grab the module where the type itself is declared
        let type_module = self.root().check_submodule(handler, item_prefix)?;

        // grab the items from where the type is declared
        let mut type_items = type_module.get_items_for_type(engines, type_id);

        let mut items = local_items;
        items.append(&mut type_items);

        let mut matching_item_decl_refs: Vec<ty::TyTraitItem> = vec![];

        for item in items.into_iter() {
            match &item {
                ty::TyTraitItem::Fn(decl_ref) => {
                    if decl_ref.name() == item_name {
                        matching_item_decl_refs.push(item.clone());
                    }
                }
                ty::TyTraitItem::Constant(decl_ref) => {
                    if decl_ref.name() == item_name {
                        matching_item_decl_refs.push(item.clone());
                    }
                }
            }
        }

        Ok(matching_item_decl_refs)
    }

    /// Given a name and a type, find that method in the namespace. Requires `args_buf`
    /// because of some special casing for the standard library where we pull
    /// the type from the arguments buffer.
    ///
    /// This function will generate a missing method error if the method is not
    /// found.
    #[allow(clippy::too_many_arguments)] // TODO: remove lint bypass once private modules are no longer experimental
    pub(crate) fn find_method_for_type(
        &mut self,
        handler: &Handler,
        type_id: TypeId,
        method_prefix: &Path,
        method_name: &Ident,
        annotation_type: TypeId,
        args_buf: &VecDeque<ty::TyExpression>,
        as_trait: Option<TypeInfo>,
        engines: &Engines,
        try_inserting_trait_impl_on_failure: TryInsertingTraitImplOnFailure,
    ) -> Result<DeclRefFunction, ErrorEmitted> {
        let decl_engine = engines.de();
        let type_engine = engines.te();

        let eq_check = UnifyCheck::non_dynamic_equality(engines);
        let coercion_check = UnifyCheck::coercion(engines);

        // default numeric types to u64
        if type_engine.contains_numeric(decl_engine, type_id) {
            type_engine.decay_numeric(handler, engines, type_id, &method_name.span())?;
        }

        let matching_item_decl_refs = self.find_items_for_type(
            handler,
            type_id,
            method_prefix,
            method_name,
            engines,
        )?;

        let matching_method_decl_refs = matching_item_decl_refs
            .into_iter()
            .flat_map(|item| match item {
                ty::TyTraitItem::Fn(decl_ref) => Some(decl_ref),
                ty::TyTraitItem::Constant(_) => None,
            })
            .collect::<Vec<_>>();

        let mut qualified_call_path = None;
        let matching_method_decl_ref = {
            // Case where multiple methods exist with the same name
            // This is the case of https://github.com/FuelLabs/sway/issues/3633
            // where multiple generic trait impls use the same method name but with different parameter types
            let mut maybe_method_decl_refs: Vec<DeclRefFunction> = vec![];
            for decl_ref in matching_method_decl_refs.clone().into_iter() {
                let method = decl_engine.get_function(&decl_ref);
                if method.parameters.len() == args_buf.len()
                    && method
                        .parameters
                        .iter()
                        .zip(args_buf.iter())
                        .all(|(p, a)| coercion_check.check(p.type_argument.type_id, a.return_type))
                    && (matches!(type_engine.get(annotation_type), TypeInfo::Unknown)
                        || coercion_check.check(annotation_type, method.return_type.type_id))
                {
                    maybe_method_decl_refs.push(decl_ref);
                }
            }

            if !maybe_method_decl_refs.is_empty() {
                let mut trait_methods =
                    HashMap::<(CallPath, Vec<WithEngines<TypeArgument>>), DeclRefFunction>::new();
                let mut impl_self_method = None;
                for method_ref in maybe_method_decl_refs.clone() {
                    let method = decl_engine.get_function(&method_ref);
                    if let Some(ty::TyDecl::ImplTrait(impl_trait)) =
                        method.implementing_type.clone()
                    {
                        let trait_decl = decl_engine.get_impl_trait(&impl_trait.decl_id);
                        if let Some(TypeInfo::Custom {
                            call_path,
                            type_arguments,
                        }) = as_trait.clone()
                        {
                            qualified_call_path = Some(call_path.clone());
                            // When `<S as Trait<T>>::method()` is used we only add methods to `trait_methods` that
                            // originate from the qualified trait.
                            if trait_decl.trait_name == call_path {
                                let mut params_equal = true;
                                if let Some(params) = type_arguments {
                                    if params.len() != trait_decl.trait_type_arguments.len() {
                                        params_equal = false;
                                    } else {
                                        for (p1, p2) in params
                                            .iter()
                                            .zip(trait_decl.trait_type_arguments.clone())
                                        {
                                            let p1_type_id = self.resolve_type(
                                                handler, engines, p1.type_id, &p1.span, EnforceTypeArguments::Yes, None,
                                            )?;
                                            let p2_type_id = self.resolve_type(
                                                handler, engines, p2.type_id, &p2.span, EnforceTypeArguments::Yes, None,
                                            )?;
                                            if !eq_check.check(p1_type_id, p2_type_id) {
                                                params_equal = false;
                                                break;
                                            }
                                        }
                                    }
                                }
                                if params_equal {
                                    trait_methods.insert(
                                        (
                                            trait_decl.trait_name,
                                            trait_decl
                                                .trait_type_arguments
                                                .iter()
                                                .cloned()
                                                .map(|a| engines.help_out(a))
                                                .collect::<Vec<_>>(),
                                        ),
                                        method_ref.clone(),
                                    );
                                }
                            }
                        } else {
                            trait_methods.insert(
                                (
                                    trait_decl.trait_name,
                                    trait_decl
                                        .trait_type_arguments
                                        .iter()
                                        .cloned()
                                        .map(|a| engines.help_out(a))
                                        .collect::<Vec<_>>(),
                                ),
                                method_ref.clone(),
                            );
                        }
                        if trait_decl.trait_decl_ref.is_none() {
                            impl_self_method = Some(method_ref);
                        }
                    }
                }

                if trait_methods.len() == 1 {
                    trait_methods.values().next().cloned()
                } else if trait_methods.len() > 1 {
                    if impl_self_method.is_some() {
                        // In case we have trait methods and a impl self method we use the impl self method.
                        impl_self_method
                    } else {
                        fn to_string(
                            trait_name: CallPath,
                            trait_type_args: Vec<WithEngines<TypeArgument>>,
                        ) -> String {
                            format!(
                                "{}{}",
                                trait_name.suffix,
                                if trait_type_args.is_empty() {
                                    String::new()
                                } else {
                                    format!(
                                        "<{}>",
                                        trait_type_args
                                            .iter()
                                            .map(|type_arg| type_arg.to_string())
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    )
                                }
                            )
                        }
                        let mut trait_strings = trait_methods
                            .keys()
                            .map(|t| to_string(t.0.clone(), t.1.clone()))
                            .collect::<Vec<String>>();
                        // Sort so the output of the error is always the same.
                        trait_strings.sort();
                        return Err(handler.emit_err(
                            CompileError::MultipleApplicableItemsInScope {
                                method_name: method_name.as_str().to_string(),
                                type_name: engines.help_out(type_id).to_string(),
                                as_traits: trait_strings,
                                span: method_name.span(),
                            },
                        ));
                    }
                } else if qualified_call_path.is_some() {
                    // When we use a qualified path the expected method should be in trait_methods.
                    None
                } else {
                    maybe_method_decl_refs.get(0).cloned()
                }
            } else {
                // When we can't match any method with parameter types we still return the first method found
                // This was the behavior before introducing the parameter type matching
                matching_method_decl_refs.get(0).cloned()
            }
        };

        if let Some(method_decl_ref) = matching_method_decl_ref {
            return Ok(method_decl_ref);
        }

        if let Some(TypeInfo::ErrorRecovery(err)) =
            args_buf.get(0).map(|x| type_engine.get(x.return_type))
        {
            Err(err)
        } else {
            if matches!(
                try_inserting_trait_impl_on_failure,
                TryInsertingTraitImplOnFailure::Yes
            ) {
                // Retrieve the implemented traits for the type and insert them in the namespace.
                // insert_trait_implementation_for_type is already called when we do type check of structs, enums, arrays and tuples.
                // In cases such as blanket trait implementation and usage of builtin types a method may not be found because
                // insert_trait_implementation_for_type has yet to be called for that type.
                self.insert_trait_implementation_for_type(engines, type_id);

                return self.find_method_for_type(
                    handler,
                    type_id,
                    method_prefix,
                    method_name,
                    annotation_type,
                    args_buf,
                    as_trait,
                    engines,
                    TryInsertingTraitImplOnFailure::No,
                );
            }
            let type_name = if let Some(call_path) = qualified_call_path {
                format!("{} as {}", engines.help_out(type_id), call_path)
            } else {
                engines.help_out(type_id).to_string()
            };
            Err(handler.emit_err(CompileError::MethodNotFound {
                method_name: method_name.clone(),
                type_name,
                span: method_name.span(),
            }))
        }
    }

    /// Given a name and a type (plus a `self_type` to potentially
    /// resolve it), find that method in the namespace. Requires `args_buf`
    /// because of some special casing for the standard library where we pull
    /// the type from the arguments buffer.
    ///
    /// This function will generate a missing method error if the method is not
    /// found.
    pub(crate) fn find_constant_for_type(
        &mut self,
        handler: &Handler,
        type_id: TypeId,
        item_name: &Ident,
        engines: &Engines,
    ) -> Result<Option<DeclRefConstant>, ErrorEmitted> {
        let matching_item_decl_refs = self.find_items_for_type(
            handler,
            type_id,
            &Vec::<Ident>::new(),
            item_name,
            engines,
        )?;

        let matching_constant_decl_refs = matching_item_decl_refs
            .into_iter()
            .flat_map(|item| match item {
                ty::TyTraitItem::Fn(_decl_ref) => None,
                ty::TyTraitItem::Constant(decl_ref) => Some(decl_ref),
            })
            .collect::<Vec<_>>();

        Ok(matching_constant_decl_refs.first().cloned())
    }

    /// Short-hand for performing a [Module::star_import] with `mod_path` as the destination.
    pub(crate) fn star_import(
        &mut self,
        handler: &Handler,
        src: &Path,
        engines: &Engines,
        is_absolute: bool,
    ) -> Result<(), ErrorEmitted> {
        self.root
            .star_import(handler, src, &self.mod_path, engines, is_absolute)
    }

    /// Short-hand for performing a [Module::variant_star_import] with `mod_path` as the destination.
    pub(crate) fn variant_star_import(
        &mut self,
        handler: &Handler,
        src: &Path,
        engines: &Engines,
        enum_name: &Ident,
        is_absolute: bool,
    ) -> Result<(), ErrorEmitted> {
        self.root.variant_star_import(
            handler,
            src,
            &self.mod_path,
            engines,
            enum_name,
            is_absolute,
        )
    }

    /// Short-hand for performing a [Module::self_import] with `mod_path` as the destination.
    pub(crate) fn self_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &Path,
        alias: Option<Ident>,
        is_absolute: bool,
    ) -> Result<(), ErrorEmitted> {
        self.root
            .self_import(handler, engines, src, &self.mod_path, alias, is_absolute)
    }

    /// Short-hand for performing a [Module::item_import] with `mod_path` as the destination.
    pub(crate) fn item_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &Path,
        item: &Ident,
        alias: Option<Ident>,
        is_absolute: bool,
    ) -> Result<(), ErrorEmitted> {
        self.root.item_import(
            handler,
            engines,
            src,
            item,
            &self.mod_path,
            alias,
            is_absolute,
        )
    }

    /// Short-hand for performing a [Module::variant_import] with `mod_path` as the destination.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn variant_import(
        &mut self,
        handler: &Handler,
        engines: &Engines,
        src: &Path,
        enum_name: &Ident,
        variant_name: &Ident,
        alias: Option<Ident>,
        is_absolute: bool,
    ) -> Result<(), ErrorEmitted> {
        self.root.variant_import(
            handler,
            engines,
            src,
            enum_name,
            variant_name,
            &self.mod_path,
            alias,
            is_absolute,
        )
    }

    /// "Enter" the submodule at the given path by returning a new [SubmoduleNamespace].
    ///
    /// Here we temporarily change `mod_path` to the given `dep_mod_path` and wrap `self` in a
    /// [SubmoduleNamespace] type. When dropped, the [SubmoduleNamespace] resets the `mod_path`
    /// back to the original path so that we can continue type-checking the current module after
    /// finishing with the dependency.
    pub(crate) fn enter_submodule(
        &mut self,
        mod_name: Ident,
        visibility: Visibility,
        module_span: Span,
    ) -> SubmoduleNamespace {
        let init = self.init.clone();
        self.submodules.entry(mod_name.to_string()).or_insert(init);
        let submod_path: Vec<_> = self
            .mod_path
            .iter()
            .cloned()
            .chain(Some(mod_name.clone()))
            .collect();
        let parent_mod_path = std::mem::replace(&mut self.mod_path, submod_path);
        self.name = Some(mod_name);
        self.span = Some(module_span);
        self.visibility = visibility;
        self.is_external = false;
        SubmoduleNamespace {
            namespace: self,
            parent_mod_path,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn insert_trait_implementation(
        &mut self,
        handler: &Handler,
        trait_name: CallPath,
        trait_type_args: Vec<TypeArgument>,
        type_id: TypeId,
        items: &[ty::TyImplItem],
        impl_span: &Span,
        trait_decl_span: Option<Span>,
        is_impl_self: bool,
        engines: &Engines,
    ) -> Result<(), ErrorEmitted> {
        // Use trait name with full path, improves consistency between
        // this inserting and getting in `get_methods_for_type_and_trait_name`.
        let full_trait_name = trait_name.to_fullpath(self);

        self.implemented_traits.insert(
            handler,
            full_trait_name,
            trait_type_args,
            type_id,
            items,
            impl_span,
            trait_decl_span,
            is_impl_self,
            engines,
        )
    }

    pub(crate) fn get_items_for_type_and_trait_name(
        &self,
        engines: &Engines,
        type_id: TypeId,
        trait_name: &CallPath,
    ) -> Vec<ty::TyTraitItem> {
        // Use trait name with full path, improves consistency between
        // this get and inserting in `insert_trait_implementation`.
        let trait_name = trait_name.to_fullpath(self);

        self.implemented_traits
            .get_items_for_type_and_trait_name(engines, type_id, &trait_name)
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
