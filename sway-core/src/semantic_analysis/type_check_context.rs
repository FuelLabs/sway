use std::collections::{HashMap, VecDeque};

use crate::{
    decl_engine::{DeclRefConstant, DeclRefFunction},
    engine_threading::*,
    language::{
        parsed::TreeType,
        ty::{self, TyDecl},
        CallPath, Purity, Visibility,
    },
    namespace::{Path, TryInsertingTraitImplOnFailure},
    semantic_analysis::{
        ast_node::{AbiMode, ConstShadowingMode},
        Namespace,
    },
    type_system::{
        EnforceTypeArguments, MonomorphizeHelper, SubstTypes, TypeArgument, TypeId, TypeInfo,
    },
    ReplaceSelfType, UnifyCheck,
};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{span::Span, Ident, Spanned};

/// Contextual state tracked and accumulated throughout type-checking.
pub struct TypeCheckContext<'a> {
    /// The namespace context accumulated throughout type-checking.
    ///
    /// Internally, this includes:
    ///
    /// - The `root` module from which all other modules maybe be accessed using absolute paths.
    /// - The `init` module used to initialise submodule namespaces.
    /// - A `mod_path` that represents the current module being type-checked. This is automatically
    ///   updated upon entering/exiting submodules via the `enter_submodule` method.
    pub(crate) namespace: &'a mut Namespace,

    pub(crate) engines: &'a Engines,

    // The following set of fields are intentionally private. When a `TypeCheckContext` is passed
    // into a new node during type checking, these fields should be updated using the `with_*`
    // methods which provides a new `TypeCheckContext`, ensuring we don't leak our changes into
    // the parent nodes.
    /// While type-checking an `impl` (whether inherent or for a `trait`/`abi`) this represents the
    /// type for which we are implementing. For example in `impl Foo {}` or `impl Trait for Foo
    /// {}`, this represents the type ID of `Foo`.
    self_type: TypeId,
    /// While type-checking an expression, this indicates the expected type.
    ///
    /// Assists type inference.
    type_annotation: TypeId,
    /// Whether or not we're within an `abi` implementation.
    ///
    /// This is `ImplAbiFn` while checking `abi` implementations whether at their original impl
    /// declaration or within an abi cast expression.
    abi_mode: AbiMode,
    /// Whether or not a const declaration shadows previous const declarations sequentially.
    ///
    /// This is `Sequential` while checking const declarations in functions, otherwise `ItemStyle`.
    const_shadowing_mode: ConstShadowingMode,
    /// Provides "help text" to `TypeError`s during unification.
    // TODO: We probably shouldn't carry this through the `Context`, but instead pass it directly
    // to `unify` as necessary?
    help_text: &'static str,
    /// Tracks the purity of the context, e.g. whether or not we should be allowed to write to
    /// storage.
    purity: Purity,
    /// Provides the kind of the module.
    /// This is useful for example to throw an error when while loops are present in predicates.
    kind: TreeType,

    /// Indicates when semantic analysis should disallow functions. (i.e.
    /// disallowing functions from being defined inside of another function
    /// body).
    disallow_functions: bool,
}

impl<'a> TypeCheckContext<'a> {
    /// Initialise a context at the top-level of a module with its namespace.
    ///
    /// Initializes with:
    ///
    /// - type_annotation: unknown
    /// - mode: NoneAbi
    /// - help_text: ""
    /// - purity: Pure
    pub fn from_root(root_namespace: &'a mut Namespace, engines: &'a Engines) -> Self {
        Self::from_module_namespace(root_namespace, engines)
    }

    fn from_module_namespace(namespace: &'a mut Namespace, engines: &'a Engines) -> Self {
        Self {
            namespace,
            engines,
            type_annotation: engines.te().insert(engines, TypeInfo::Unknown),
            help_text: "",
            // TODO: Contract? Should this be passed in based on program kind (aka TreeType)?
            self_type: engines.te().insert(engines, TypeInfo::Contract),
            abi_mode: AbiMode::NonAbi,
            const_shadowing_mode: ConstShadowingMode::ItemStyle,
            purity: Purity::default(),
            kind: TreeType::Contract,
            disallow_functions: false,
        }
    }

    /// Create a new context that mutably borrows the inner `namespace` with a lifetime bound by
    /// `self`.
    ///
    /// This is particularly useful when type-checking a node that has more than one child node
    /// (very often the case). By taking the context with the namespace lifetime bound to `self`
    /// rather than the original namespace reference, we instead restrict the returned context to
    /// the local scope and avoid consuming the original context when providing context to the
    /// first visited child node.
    pub fn by_ref(&mut self) -> TypeCheckContext<'_> {
        TypeCheckContext {
            namespace: self.namespace,
            type_annotation: self.type_annotation,
            self_type: self.self_type,
            abi_mode: self.abi_mode.clone(),
            const_shadowing_mode: self.const_shadowing_mode,
            help_text: self.help_text,
            purity: self.purity,
            kind: self.kind.clone(),
            engines: self.engines,
            disallow_functions: self.disallow_functions,
        }
    }

    /// Scope the `TypeCheckContext` with the given `Namespace`.
    pub fn scoped(self, namespace: &'a mut Namespace) -> TypeCheckContext<'a> {
        TypeCheckContext {
            namespace,
            type_annotation: self.type_annotation,
            self_type: self.self_type,
            abi_mode: self.abi_mode,
            const_shadowing_mode: self.const_shadowing_mode,
            help_text: self.help_text,
            purity: self.purity,
            kind: self.kind,
            engines: self.engines,
            disallow_functions: self.disallow_functions,
        }
    }

    /// Enter the submodule with the given name and produce a type-check context ready for
    /// type-checking its content.
    ///
    /// Returns the result of the given `with_submod_ctx` function.
    pub fn enter_submodule<T>(
        self,
        mod_name: Ident,
        visibility: Visibility,
        module_span: Span,
        with_submod_ctx: impl FnOnce(TypeCheckContext) -> T,
    ) -> T {
        // We're checking a submodule, so no need to pass through anything other than the
        // namespace. However, we will likely want to pass through the type engine and declaration
        // engine here once they're added.
        let Self { namespace, .. } = self;
        let mut submod_ns = namespace.enter_submodule(mod_name, visibility, module_span);
        let submod_ctx = TypeCheckContext::from_module_namespace(&mut submod_ns, self.engines);
        with_submod_ctx(submod_ctx)
    }

    /// Map this `TypeCheckContext` instance to a new one with the given `help_text`.
    pub(crate) fn with_help_text(self, help_text: &'static str) -> Self {
        Self { help_text, ..self }
    }

    /// Map this `TypeCheckContext` instance to a new one with the given type annotation.
    pub(crate) fn with_type_annotation(self, type_annotation: TypeId) -> Self {
        Self {
            type_annotation,
            ..self
        }
    }

    /// Map this `TypeCheckContext` instance to a new one with the given ABI `mode`.
    pub(crate) fn with_abi_mode(self, abi_mode: AbiMode) -> Self {
        Self { abi_mode, ..self }
    }

    /// Map this `TypeCheckContext` instance to a new one with the given const shadowing `mode`.
    pub(crate) fn with_const_shadowing_mode(
        self,
        const_shadowing_mode: ConstShadowingMode,
    ) -> Self {
        Self {
            const_shadowing_mode,
            ..self
        }
    }

    /// Map this `TypeCheckContext` instance to a new one with the given purity.
    pub(crate) fn with_purity(self, purity: Purity) -> Self {
        Self { purity, ..self }
    }

    /// Map this `TypeCheckContext` instance to a new one with the given module kind.
    pub(crate) fn with_kind(self, kind: TreeType) -> Self {
        Self { kind, ..self }
    }

    /// Map this `TypeCheckContext` instance to a new one with the given purity.
    pub(crate) fn with_self_type(self, self_type: TypeId) -> Self {
        Self { self_type, ..self }
    }

    /// Map this `TypeCheckContext` instance to a new one with
    /// `disallow_functions` set to `true`.
    pub(crate) fn disallow_functions(self) -> Self {
        Self {
            disallow_functions: true,
            ..self
        }
    }

    /// Map this `TypeCheckContext` instance to a new one with
    /// `disallow_functions` set to `false`.
    pub(crate) fn allow_functions(self) -> Self {
        Self {
            disallow_functions: false,
            ..self
        }
    }

    // A set of accessor methods. We do this rather than making the fields `pub` in order to ensure
    // that these are only updated via the `with_*` methods that produce a new `TypeCheckContext`.

    pub(crate) fn help_text(&self) -> &'static str {
        self.help_text
    }

    pub(crate) fn type_annotation(&self) -> TypeId {
        self.type_annotation
    }

    pub(crate) fn abi_mode(&self) -> AbiMode {
        self.abi_mode.clone()
    }

    pub(crate) fn const_shadowing_mode(&self) -> ConstShadowingMode {
        self.const_shadowing_mode
    }

    pub(crate) fn purity(&self) -> Purity {
        self.purity
    }

    #[allow(dead_code)]
    pub(crate) fn kind(&self) -> TreeType {
        self.kind.clone()
    }

    pub(crate) fn self_type(&self) -> TypeId {
        self.self_type
    }

    pub(crate) fn functions_disallowed(&self) -> bool {
        self.disallow_functions
    }

    // Provide some convenience functions around the inner context.

    /// Short-hand for calling the `monomorphize` function in the type engine
    pub(crate) fn monomorphize<T>(
        &mut self,
        handler: &Handler,
        value: &mut T,
        type_arguments: &mut [TypeArgument],
        enforce_type_arguments: EnforceTypeArguments,
        call_site_span: &Span,
    ) -> Result<(), ErrorEmitted>
    where
        T: MonomorphizeHelper + SubstTypes,
    {
        let mod_path = self.namespace.mod_path.clone();
        self.engines.te().monomorphize(
            handler,
            self.engines(),
            value,
            type_arguments,
            enforce_type_arguments,
            call_site_span,
            self.namespace,
            &mod_path,
        )
    }

    /// Short-hand around `type_system::unify_with_self`, where the `TypeCheckContext` provides the
    /// type annotation, self type and help text.
    pub(crate) fn unify_with_self(&self, handler: &Handler, ty: TypeId, span: &Span) {
        self.engines.te().unify_with_self(
            handler,
            self.engines(),
            ty,
            self.type_annotation(),
            self.self_type(),
            span,
            self.help_text(),
            None,
        )
    }

    /// Short-hand for calling [Namespace::insert_symbol] with the `const_shadowing_mode` provided by
    /// the `TypeCheckContext`.
    pub(crate) fn insert_symbol(
        &mut self,
        handler: &Handler,
        name: Ident,
        item: TyDecl,
    ) -> Result<(), ErrorEmitted> {
        self.namespace
            .insert_symbol(handler, name, item, self.const_shadowing_mode)
    }

    /// Get the engines needed for engine threading.
    pub(crate) fn engines(&self) -> &'a Engines {
        self.engines
    }

    /// Short-hand for calling [Root::resolve_type_with_self] on `root` with the `mod_path`.
    #[allow(clippy::too_many_arguments)] // TODO: remove lint bypass once private modules are no longer experimental
    pub(crate) fn resolve_type_with_self(
        &mut self,
        handler: &Handler,
        type_id: TypeId,
        self_type: TypeId,
        span: &Span,
        enforce_type_arguments: EnforceTypeArguments,
        type_info_prefix: Option<&Path>,
    ) -> Result<TypeId, ErrorEmitted> {
        let mod_path = self.namespace.mod_path.clone();
        self.engines.te().resolve_with_self(
            handler,
            self.engines,
            type_id,
            self_type,
            span,
            enforce_type_arguments,
            type_info_prefix,
            self.namespace,
            &mod_path,
        )
    }

    /// Short-hand for calling [Root::resolve_type_without_self] on `root` and with the `mod_path`.
    pub(crate) fn resolve_type_without_self(
        &mut self,
        handler: &Handler,
        type_id: TypeId,
        span: &Span,
        type_info_prefix: Option<&Path>,
    ) -> Result<TypeId, ErrorEmitted> {
        let mod_path = self.namespace.mod_path.clone();
        self.engines.te().resolve(
            handler,
            self.engines,
            type_id,
            span,
            EnforceTypeArguments::Yes,
            type_info_prefix,
            self.namespace,
            &mod_path,
        )
    }

    /// Short-hand for calling [Root::resolve_call_path_with_visibility_check] on `root` with the `mod_path`.
    pub(crate) fn resolve_call_path_with_visibility_check(
        &self,
        handler: &Handler,
        call_path: &CallPath,
    ) -> Result<&ty::TyDecl, ErrorEmitted> {
        self.namespace.root.resolve_call_path_with_visibility_check(
            handler,
            self.engines,
            &self.namespace.mod_path,
            call_path,
        )
    }

    /// Given a name and a type (plus a `self_type` to potentially
    /// resolve it), find items matching in the namespace.
    pub(crate) fn find_items_for_type(
        &mut self,
        handler: &Handler,
        mut type_id: TypeId,
        item_prefix: &Path,
        item_name: &Ident,
        self_type: TypeId,
    ) -> Result<Vec<ty::TyTraitItem>, ErrorEmitted> {
        let type_engine = self.engines.te();
        let _decl_engine = self.engines.de();

        // If the type that we are looking for is the error recovery type, then
        // we want to return the error case without creating a new error
        // message.
        if let TypeInfo::ErrorRecovery(err) = type_engine.get(type_id) {
            return Err(err);
        }

        // grab the local module
        let local_module = self
            .namespace
            .root()
            .check_submodule(handler, &self.namespace.mod_path)?;

        // grab the local items from the local module
        let local_items = local_module.get_items_for_type(self.engines, type_id);

        type_id.replace_self_type(self.engines, self_type);

        // resolve the type
        let type_id = type_engine
            .resolve(
                handler,
                self.engines,
                type_id,
                &item_name.span(),
                EnforceTypeArguments::No,
                None,
                self.namespace,
                item_prefix,
            )
            .unwrap_or_else(|err| type_engine.insert(self.engines, TypeInfo::ErrorRecovery(err)));

        // grab the module where the type itself is declared
        let type_module = self
            .namespace
            .root()
            .check_submodule(handler, item_prefix)?;

        // grab the items from where the type is declared
        let mut type_items = type_module.get_items_for_type(self.engines, type_id);

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

    /// Given a name and a type (plus a `self_type` to potentially
    /// resolve it), find that method in the namespace. Requires `args_buf`
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
        self_type: TypeId,
        annotation_type: TypeId,
        args_buf: &VecDeque<ty::TyExpression>,
        as_trait: Option<TypeInfo>,
        try_inserting_trait_impl_on_failure: TryInsertingTraitImplOnFailure,
    ) -> Result<DeclRefFunction, ErrorEmitted> {
        let decl_engine = self.engines.de();
        let type_engine = self.engines.te();

        let eq_check = UnifyCheck::non_dynamic_equality(self.engines);
        let coercion_check = UnifyCheck::coercion(self.engines);

        // default numeric types to u64
        if type_engine.contains_numeric(decl_engine, type_id) {
            type_engine.decay_numeric(handler, self.engines, type_id, &method_name.span())?;
        }

        let matching_item_decl_refs =
            self.find_items_for_type(handler, type_id, method_prefix, method_name, self_type)?;

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
                                            let p1_type_id = self.resolve_type_without_self(
                                                handler, p1.type_id, &p1.span, None,
                                            )?;
                                            let p2_type_id = self.resolve_type_without_self(
                                                handler, p2.type_id, &p2.span, None,
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
                                                .map(|a| self.engines.help_out(a))
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
                                        .map(|a| self.engines.help_out(a))
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
                                type_name: self.engines.help_out(type_id).to_string(),
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
                self.namespace
                    .insert_trait_implementation_for_type(self.engines, type_id);

                return self.find_method_for_type(
                    handler,
                    type_id,
                    method_prefix,
                    method_name,
                    self_type,
                    annotation_type,
                    args_buf,
                    as_trait,
                    TryInsertingTraitImplOnFailure::No,
                );
            }
            let type_name = if let Some(call_path) = qualified_call_path {
                format!("{} as {}", self.engines.help_out(type_id), call_path)
            } else {
                self.engines.help_out(type_id).to_string()
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
        self_type: TypeId,
    ) -> Result<Option<DeclRefConstant>, ErrorEmitted> {
        let matching_item_decl_refs =
            self.find_items_for_type(handler, type_id, &Vec::<Ident>::new(), item_name, self_type)?;

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
        is_absolute: bool,
    ) -> Result<(), ErrorEmitted> {
        self.namespace.root.star_import(
            handler,
            src,
            &self.namespace.mod_path,
            self.engines,
            is_absolute,
        )
    }

    /// Short-hand for performing a [Module::variant_star_import] with `mod_path` as the destination.
    pub(crate) fn variant_star_import(
        &mut self,
        handler: &Handler,
        src: &Path,
        enum_name: &Ident,
        is_absolute: bool,
    ) -> Result<(), ErrorEmitted> {
        self.namespace.root.variant_star_import(
            handler,
            src,
            &self.namespace.mod_path,
            self.engines,
            enum_name,
            is_absolute,
        )
    }

    /// Short-hand for performing a [Module::self_import] with `mod_path` as the destination.
    pub(crate) fn self_import(
        &mut self,
        handler: &Handler,
        src: &Path,
        alias: Option<Ident>,
        is_absolute: bool,
    ) -> Result<(), ErrorEmitted> {
        self.namespace.root.self_import(
            handler,
            self.engines,
            src,
            &self.namespace.mod_path,
            alias,
            is_absolute,
        )
    }

    /// Short-hand for performing a [Module::item_import] with `mod_path` as the destination.
    pub(crate) fn item_import(
        &mut self,
        handler: &Handler,
        src: &Path,
        item: &Ident,
        alias: Option<Ident>,
        is_absolute: bool,
    ) -> Result<(), ErrorEmitted> {
        self.namespace.root.item_import(
            handler,
            self.engines,
            src,
            item,
            &self.namespace.mod_path,
            alias,
            is_absolute,
        )
    }

    /// Short-hand for performing a [Module::variant_import] with `mod_path` as the destination.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn variant_import(
        &mut self,
        handler: &Handler,
        src: &Path,
        enum_name: &Ident,
        variant_name: &Ident,
        alias: Option<Ident>,
        is_absolute: bool,
    ) -> Result<(), ErrorEmitted> {
        self.namespace.root.variant_import(
            handler,
            self.engines,
            src,
            enum_name,
            variant_name,
            &self.namespace.mod_path,
            alias,
            is_absolute,
        )
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
    ) -> Result<(), ErrorEmitted> {
        // Use trait name with full path, improves consistency between
        // this inserting and getting in `get_methods_for_type_and_trait_name`.
        let full_trait_name = trait_name.to_fullpath(self.namespace);

        self.namespace.implemented_traits.insert(
            handler,
            full_trait_name,
            trait_type_args,
            type_id,
            items,
            impl_span,
            trait_decl_span,
            is_impl_self,
            self.engines,
        )
    }

    pub(crate) fn get_items_for_type_and_trait_name(
        &self,
        type_id: TypeId,
        trait_name: &CallPath,
    ) -> Vec<ty::TyTraitItem> {
        // Use trait name with full path, improves consistency between
        // this get and inserting in `insert_trait_implementation`.
        let trait_name = trait_name.to_fullpath(self.namespace);

        self.namespace
            .implemented_traits
            .get_items_for_type_and_trait_name(self.engines, type_id, &trait_name)
    }
}
