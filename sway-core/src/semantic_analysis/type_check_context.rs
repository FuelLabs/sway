#![allow(clippy::mutable_key_type)]
use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    decl_engine::{DeclEngineGet, DeclRefFunction},
    engine_threading::*,
    language::{
        parsed::TreeType,
        ty::{self, TyDecl},
        CallPath, QualifiedCallPath, Visibility,
    },
    monomorphization::{monomorphize_with_modpath, MonomorphizeHelper},
    namespace::{
        IsExtendingExistingImpl, IsImplSelf, ModulePath, ResolvedDeclaration,
        ResolvedTraitImplItem, TraitMap, TryInsertingTraitImplOnFailure,
    },
    semantic_analysis::{
        ast_node::{AbiMode, ConstShadowingMode},
        Namespace,
    },
    type_system::{SubstTypes, TypeArgument, TypeId, TypeInfo},
    EnforceTypeArguments, SubstTypesContext, TraitConstraint, TypeSubstMap, UnifyCheck,
};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_features::ExperimentalFeatures;
use sway_types::{span::Span, Ident, Spanned};

use super::{
    symbol_collection_context::SymbolCollectionContext,
    type_resolve::{resolve_call_path, resolve_qualified_call_path, resolve_type},
    GenericShadowingMode,
};

/// Contextual state tracked and accumulated throughout type-checking.
pub struct TypeCheckContext<'a> {
    /// The namespace context accumulated throughout type-checking.
    ///
    /// Internally, this includes:
    ///
    /// - The `root` module from which all other modules maybe be accessed using absolute paths.
    /// - The `init` module used to initialize submodule namespaces.
    /// - A `mod_path` that represents the current module being type-checked. This is automatically
    ///   updated upon entering/exiting submodules via the `enter_submodule` method.
    pub(crate) namespace: &'a mut Namespace,

    pub(crate) engines: &'a Engines,

    /// Set of experimental flags.
    pub(crate) experimental: ExperimentalFeatures,

    /// Keeps the accumulated symbols previously collected.
    pub(crate) collection_ctx: &'a mut SymbolCollectionContext,

    // The following set of fields are intentionally private. When a `TypeCheckContext` is passed
    // into a new node during type checking, these fields should be updated using the `with_*`
    // methods which provides a new `TypeCheckContext`, ensuring we don't leak our changes into
    // the parent nodes.
    /// While type-checking an expression, this indicates the expected type.
    ///
    /// Assists type inference.
    type_annotation: TypeId,
    /// Assists type inference.
    function_type_annotation: TypeId,
    /// When true unify_with_type_annotation will use unify_with_generic instead of the default unify.
    /// This ensures that expected generic types are unified to more specific received types.
    unify_generic: bool,
    /// While type-checking an `impl` (whether inherent or for a `trait`/`abi`) this represents the
    /// type for which we are implementing. For example in `impl Foo {}` or `impl Trait for Foo
    /// {}`, this represents the type ID of `Foo`.
    self_type: Option<TypeId>,
    /// While type-checking an expression, this indicates the types to be substituted when a
    /// type is resolved. This is required is to replace associated types, namely TypeInfo::TraitType.
    type_subst: TypeSubstMap,
    /// Whether or not we're within an `abi` implementation.
    ///
    /// This is `ImplAbiFn` while checking `abi` implementations whether at their original impl
    /// declaration or within an abi cast expression.
    abi_mode: AbiMode,
    /// Whether or not a const declaration shadows previous const declarations sequentially.
    ///
    /// This is `Sequential` while checking const declarations in functions, otherwise `ItemStyle`.
    const_shadowing_mode: ConstShadowingMode,
    /// Whether or not a generic type parameters shadows previous generic type parameters.
    ///
    /// This is `Disallow` everywhere except while checking type parameters bounds in struct instantiation.
    generic_shadowing_mode: GenericShadowingMode,
    /// Provides "help text" to `TypeError`s during unification.
    // TODO: We probably shouldn't carry this through the `Context`, but instead pass it directly
    // to `unify` as necessary?
    help_text: &'static str,
    /// Provides the kind of the module.
    /// This is useful for example to throw an error when while loops are present in predicates.
    kind: TreeType,

    /// Indicates when semantic analysis should disallow functions. (i.e.
    /// disallowing functions from being defined inside of another function
    /// body).
    disallow_functions: bool,

    /// Indicates when semantic analysis is type checking storage declaration.
    storage_declaration: bool,

    // Indicates when we are collecting unifications.
    collecting_unifications: bool,

    // Indicates when we are doing the first pass of the code block type checking.
    // In some nested places of the first pass we want to disable the first pass optimizations
    // To disable those optimizations we can set this to false.
    code_block_first_pass: bool,
}

impl<'a> TypeCheckContext<'a> {
    /// Initialize a type-checking context with a namespace.
    pub fn from_namespace(
        namespace: &'a mut Namespace,
        collection_ctx: &'a mut SymbolCollectionContext,
        engines: &'a Engines,
        experimental: ExperimentalFeatures,
    ) -> Self {
        Self {
            namespace,
            engines,
            collection_ctx,
            type_annotation: engines.te().insert(engines, TypeInfo::Unknown, None),
            function_type_annotation: engines.te().insert(engines, TypeInfo::Unknown, None),
            unify_generic: false,
            self_type: None,
            type_subst: TypeSubstMap::new(),
            help_text: "",
            abi_mode: AbiMode::NonAbi,
            const_shadowing_mode: ConstShadowingMode::ItemStyle,
            generic_shadowing_mode: GenericShadowingMode::Disallow,
            kind: TreeType::Contract,
            disallow_functions: false,
            storage_declaration: false,
            experimental,
            collecting_unifications: false,
            code_block_first_pass: false,
        }
    }

    /// Initialize a context at the top-level of a module with its namespace.
    ///
    /// Initializes with:
    ///
    /// - type_annotation: unknown
    /// - mode: NoneAbi
    /// - help_text: ""
    pub fn from_root(
        root_namespace: &'a mut Namespace,
        collection_ctx: &'a mut SymbolCollectionContext,
        engines: &'a Engines,
        experimental: ExperimentalFeatures,
    ) -> Self {
        Self::from_module_namespace(root_namespace, collection_ctx, engines, experimental)
    }

    fn from_module_namespace(
        namespace: &'a mut Namespace,
        collection_ctx: &'a mut SymbolCollectionContext,
        engines: &'a Engines,
        experimental: ExperimentalFeatures,
    ) -> Self {
        Self {
            collection_ctx,
            namespace,
            engines,
            type_annotation: engines.te().insert(engines, TypeInfo::Unknown, None),
            function_type_annotation: engines.te().insert(engines, TypeInfo::Unknown, None),
            unify_generic: false,
            self_type: None,
            type_subst: TypeSubstMap::new(),
            help_text: "",
            abi_mode: AbiMode::NonAbi,
            const_shadowing_mode: ConstShadowingMode::ItemStyle,
            generic_shadowing_mode: GenericShadowingMode::Disallow,
            kind: TreeType::Contract,
            disallow_functions: false,
            storage_declaration: false,
            experimental,
            collecting_unifications: false,
            code_block_first_pass: false,
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
            collection_ctx: self.collection_ctx,
            type_annotation: self.type_annotation,
            function_type_annotation: self.function_type_annotation,
            unify_generic: self.unify_generic,
            self_type: self.self_type,
            type_subst: self.type_subst.clone(),
            abi_mode: self.abi_mode.clone(),
            const_shadowing_mode: self.const_shadowing_mode,
            generic_shadowing_mode: self.generic_shadowing_mode,
            help_text: self.help_text,
            kind: self.kind,
            engines: self.engines,
            disallow_functions: self.disallow_functions,
            storage_declaration: self.storage_declaration,
            experimental: self.experimental,
            collecting_unifications: self.collecting_unifications,
            code_block_first_pass: self.code_block_first_pass,
        }
    }

    /// Scope the `TypeCheckContext` with a new namespace, and set up the collection context
    /// so it enters the lexical scope corresponding to the given span.
    pub fn scoped<T>(
        self,
        handler: &Handler,
        span: Option<Span>,
        with_scoped_ctx: impl FnOnce(TypeCheckContext) -> Result<T, ErrorEmitted>,
    ) -> Result<T, ErrorEmitted> {
        let mut namespace = self.namespace.clone();
        if let Some(span) = span {
            self.collection_ctx.enter_lexical_scope(
                handler,
                self.engines,
                span,
                |scoped_collection_ctx| {
                    let ctx = TypeCheckContext {
                        namespace: &mut namespace,
                        collection_ctx: scoped_collection_ctx,
                        type_annotation: self.type_annotation,
                        function_type_annotation: self.function_type_annotation,
                        unify_generic: self.unify_generic,
                        self_type: self.self_type,
                        type_subst: self.type_subst,
                        abi_mode: self.abi_mode,
                        const_shadowing_mode: self.const_shadowing_mode,
                        generic_shadowing_mode: self.generic_shadowing_mode,
                        help_text: self.help_text,
                        kind: self.kind,
                        engines: self.engines,
                        disallow_functions: self.disallow_functions,
                        storage_declaration: self.storage_declaration,
                        experimental: self.experimental,
                        collecting_unifications: self.collecting_unifications,
                        code_block_first_pass: self.code_block_first_pass,
                    };
                    with_scoped_ctx(ctx)
                },
            )
        } else {
            let ctx = TypeCheckContext {
                collection_ctx: self.collection_ctx,
                namespace: &mut namespace,
                type_annotation: self.type_annotation,
                function_type_annotation: self.function_type_annotation,
                unify_generic: self.unify_generic,
                self_type: self.self_type,
                type_subst: self.type_subst,
                abi_mode: self.abi_mode,
                const_shadowing_mode: self.const_shadowing_mode,
                generic_shadowing_mode: self.generic_shadowing_mode,
                help_text: self.help_text,
                kind: self.kind,
                engines: self.engines,
                disallow_functions: self.disallow_functions,
                storage_declaration: self.storage_declaration,
                experimental: self.experimental,
                collecting_unifications: self.collecting_unifications,
                code_block_first_pass: self.code_block_first_pass,
            };
            with_scoped_ctx(ctx)
        }
    }

    /// Scope the `TypeCheckContext` with a new namespace and returns it in case of success.
    /// Also sets up the collection context so it enters the lexical scope corresponding to
    /// the given span.
    pub fn scoped_and_namespace<T>(
        self,
        handler: &Handler,
        span: Option<Span>,
        with_scoped_ctx: impl FnOnce(TypeCheckContext) -> Result<T, ErrorEmitted>,
    ) -> Result<(T, Namespace), ErrorEmitted> {
        let mut namespace = self.namespace.clone();
        if let Some(span) = span {
            self.collection_ctx.enter_lexical_scope(
                handler,
                self.engines,
                span,
                |scoped_collection_ctx| {
                    let ctx = TypeCheckContext {
                        collection_ctx: scoped_collection_ctx,
                        namespace: &mut namespace,
                        type_annotation: self.type_annotation,
                        function_type_annotation: self.function_type_annotation,
                        unify_generic: self.unify_generic,
                        self_type: self.self_type,
                        type_subst: self.type_subst,
                        abi_mode: self.abi_mode,
                        const_shadowing_mode: self.const_shadowing_mode,
                        generic_shadowing_mode: self.generic_shadowing_mode,
                        help_text: self.help_text,
                        kind: self.kind,
                        engines: self.engines,
                        disallow_functions: self.disallow_functions,
                        storage_declaration: self.storage_declaration,
                        experimental: self.experimental,
                        collecting_unifications: self.collecting_unifications,
                        code_block_first_pass: self.code_block_first_pass,
                    };
                    Ok((with_scoped_ctx(ctx)?, namespace))
                },
            )
        } else {
            let ctx = TypeCheckContext {
                collection_ctx: self.collection_ctx,
                namespace: &mut namespace,
                type_annotation: self.type_annotation,
                function_type_annotation: self.function_type_annotation,
                unify_generic: self.unify_generic,
                self_type: self.self_type,
                type_subst: self.type_subst,
                abi_mode: self.abi_mode,
                const_shadowing_mode: self.const_shadowing_mode,
                generic_shadowing_mode: self.generic_shadowing_mode,
                help_text: self.help_text,
                kind: self.kind,
                engines: self.engines,
                disallow_functions: self.disallow_functions,
                storage_declaration: self.storage_declaration,
                experimental: self.experimental,
                collecting_unifications: self.collecting_unifications,
                code_block_first_pass: self.code_block_first_pass,
            };
            Ok((with_scoped_ctx(ctx)?, namespace))
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
        let experimental = self.experimental;

        // We're checking a submodule, so no need to pass through anything other than the
        // namespace and the engines.
        let engines = self.engines;
        let mut submod_ns = self.namespace.enter_submodule(
            engines,
            mod_name.clone(),
            visibility,
            module_span.clone(),
        );

        self.collection_ctx.enter_submodule(
            engines,
            mod_name,
            visibility,
            module_span,
            |submod_collection_ctx| {
                let submod_ctx = TypeCheckContext::from_namespace(
                    &mut submod_ns,
                    submod_collection_ctx,
                    engines,
                    experimental,
                );
                with_submod_ctx(submod_ctx)
            },
        )
    }

    /// Returns a mutable reference to the current namespace.
    pub fn namespace_mut(&mut self) -> &mut Namespace {
        self.namespace
    }

    /// Returns a reference to the current namespace.
    pub fn namespace(&self) -> &Namespace {
        self.namespace
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

    /// Map this `TypeCheckContext` instance to a new one with the given type annotation.
    pub(crate) fn with_function_type_annotation(self, function_type_annotation: TypeId) -> Self {
        Self {
            function_type_annotation,
            ..self
        }
    }

    /// Map this `TypeCheckContext` instance to a new one with the given type annotation.
    pub(crate) fn with_unify_generic(self, unify_generic: bool) -> Self {
        Self {
            unify_generic,
            ..self
        }
    }

    /// Map this `TypeCheckContext` instance to a new one with the given type subst.
    pub(crate) fn with_type_subst(self, type_subst: &TypeSubstMap) -> Self {
        Self {
            type_subst: type_subst.clone(),
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

    /// Map this `TypeCheckContext` instance to a new one with the given generic shadowing `mode`.
    pub(crate) fn with_generic_shadowing_mode(
        self,
        generic_shadowing_mode: GenericShadowingMode,
    ) -> Self {
        Self {
            generic_shadowing_mode,
            ..self
        }
    }

    /// Map this `TypeCheckContext` instance to a new one with the given module kind.
    pub(crate) fn with_kind(self, kind: TreeType) -> Self {
        Self { kind, ..self }
    }

    /// Map this `TypeCheckContext` instance to a new one with the given self type.
    pub(crate) fn with_self_type(self, self_type: Option<TypeId>) -> Self {
        Self { self_type, ..self }
    }

    pub(crate) fn with_collecting_unifications(self) -> Self {
        Self {
            collecting_unifications: true,
            ..self
        }
    }

    pub(crate) fn with_code_block_first_pass(self, value: bool) -> Self {
        Self {
            code_block_first_pass: value,
            ..self
        }
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

    /// Map this `TypeCheckContext` instance to a new one with
    /// `storage_declaration` set to `true`.
    pub(crate) fn with_storage_declaration(self) -> Self {
        Self {
            storage_declaration: true,
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

    pub(crate) fn function_type_annotation(&self) -> TypeId {
        self.function_type_annotation
    }

    pub(crate) fn unify_generic(&self) -> bool {
        self.unify_generic
    }

    pub(crate) fn self_type(&self) -> Option<TypeId> {
        self.self_type
    }

    pub(crate) fn subst_ctx(&self) -> SubstTypesContext {
        SubstTypesContext::new(
            self.engines(),
            &self.type_subst,
            !self.code_block_first_pass(),
        )
    }

    pub(crate) fn abi_mode(&self) -> AbiMode {
        self.abi_mode.clone()
    }

    #[allow(dead_code)]
    pub(crate) fn kind(&self) -> TreeType {
        self.kind
    }

    pub(crate) fn functions_disallowed(&self) -> bool {
        self.disallow_functions
    }

    pub(crate) fn storage_declaration(&self) -> bool {
        self.storage_declaration
    }

    pub(crate) fn collecting_unifications(&self) -> bool {
        self.collecting_unifications
    }

    pub(crate) fn code_block_first_pass(&self) -> bool {
        self.code_block_first_pass
    }

    /// Get the engines needed for engine threading.
    pub(crate) fn engines(&self) -> &'a Engines {
        self.engines
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
        let mod_path = self.namespace().mod_path.clone();
        monomorphize_with_modpath(
            handler,
            self.engines(),
            self.namespace(),
            value,
            type_arguments,
            enforce_type_arguments,
            call_site_span,
            &mod_path,
            self.self_type(),
            &self.subst_ctx(),
        )
    }

    /// Short-hand around `type_system::unify_`, where the `TypeCheckContext`
    /// provides the type annotation and help text.
    pub(crate) fn unify_with_type_annotation(&self, handler: &Handler, ty: TypeId, span: &Span) {
        if self.unify_generic() {
            self.engines.te().unify_with_generic(
                handler,
                self.engines(),
                ty,
                self.type_annotation(),
                span,
                self.help_text(),
                None,
            )
        } else {
            self.engines.te().unify(
                handler,
                self.engines(),
                ty,
                self.type_annotation(),
                span,
                self.help_text(),
                None,
            )
        }
    }

    /// Short-hand for calling [Namespace::insert_symbol] with the `const_shadowing_mode` provided by
    /// the `TypeCheckContext`.
    pub(crate) fn insert_symbol(
        &mut self,
        handler: &Handler,
        name: Ident,
        item: TyDecl,
    ) -> Result<(), ErrorEmitted> {
        let const_shadowing_mode = self.const_shadowing_mode;
        let generic_shadowing_mode = self.generic_shadowing_mode;
        let collecting_unifications = self.collecting_unifications;
        let engines = self.engines();
        self.namespace_mut()
            .module_mut(engines)
            .current_items_mut()
            .insert_symbol(
                handler,
                engines,
                name,
                ResolvedDeclaration::Typed(item),
                const_shadowing_mode,
                generic_shadowing_mode,
                collecting_unifications,
            )
    }

    /// Short-hand for calling [resolve_type] on `root` with the `mod_path`.
    pub(crate) fn resolve_type(
        &self,
        handler: &Handler,
        type_id: TypeId,
        span: &Span,
        enforce_type_arguments: EnforceTypeArguments,
        type_info_prefix: Option<&ModulePath>,
    ) -> Result<TypeId, ErrorEmitted> {
        resolve_type(
            handler,
            self.engines(),
            self.namespace(),
            &self.namespace().mod_path,
            type_id,
            span,
            enforce_type_arguments,
            type_info_prefix,
            self.self_type(),
            &self.subst_ctx(),
        )
    }

    /// Short-hand for calling [Root::resolve_call_path_with_visibility_check] on `root` with the `mod_path`.
    pub(crate) fn resolve_call_path_with_visibility_check(
        &self,
        handler: &Handler,
        call_path: &CallPath,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        resolve_call_path(
            handler,
            self.engines(),
            self.namespace(),
            &self.namespace().mod_path,
            call_path,
            self.self_type(),
        )
    }

    pub(crate) fn resolve_qualified_call_path_with_visibility_check(
        &mut self,
        handler: &Handler,
        qualified_call_path: &QualifiedCallPath,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        resolve_qualified_call_path(
            handler,
            self.engines(),
            self.namespace(),
            &self.namespace().mod_path.clone(),
            qualified_call_path,
            self.self_type(),
            &self.subst_ctx(),
        )
    }

    /// Given a name and a type (plus a `self_type` to potentially
    /// resolve it), find items matching in the namespace.
    pub(crate) fn find_items_for_type(
        &mut self,
        handler: &Handler,
        type_id: TypeId,
        item_prefix: &ModulePath,
        item_name: &Ident,
    ) -> Result<Vec<ty::TyTraitItem>, ErrorEmitted> {
        let type_engine = self.engines.te();
        let _decl_engine = self.engines.de();

        // If the type that we are looking for is the error recovery type, then
        // we want to return the error case without creating a new error
        // message.
        if let TypeInfo::ErrorRecovery(err) = &*type_engine.get(type_id) {
            return Err(*err);
        }

        // grab the local module
        let local_module = self.namespace().lookup_submodule_from_absolute_path(
            handler,
            self.engines(),
            &self.namespace().mod_path,
        )?;

        // grab the local items from the local module
        let local_items = local_module
            .current_items()
            .get_items_for_type(self.engines, type_id);

        // resolve the type
        let type_id = resolve_type(
            handler,
            self.engines(),
            self.namespace(),
            item_prefix,
            type_id,
            &item_name.span(),
            EnforceTypeArguments::No,
            None,
            self.self_type(),
            &self.subst_ctx(),
        )
        .unwrap_or_else(|err| type_engine.insert(self.engines, TypeInfo::ErrorRecovery(err), None));

        // grab the module where the type itself is declared
        let type_module = self.namespace().lookup_submodule_from_absolute_path(
            handler,
            self.engines(),
            item_prefix,
        )?;

        // grab the items from where the type is declared
        let mut type_items = type_module
            .current_items()
            .get_items_for_type(self.engines, type_id);

        let mut items = local_items;
        items.append(&mut type_items);

        let mut matching_item_decl_refs: Vec<ty::TyTraitItem> = vec![];

        for item in items.into_iter() {
            match &item {
                ResolvedTraitImplItem::Parsed(_) => todo!(),
                ResolvedTraitImplItem::Typed(item) => match item {
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
                    ty::TyTraitItem::Type(decl_ref) => {
                        if decl_ref.name() == item_name {
                            matching_item_decl_refs.push(item.clone());
                        }
                    }
                },
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
        method_prefix: &ModulePath,
        method_name: &Ident,
        annotation_type: TypeId,
        arguments_types: &VecDeque<TypeId>,
        as_trait: Option<TypeId>,
        try_inserting_trait_impl_on_failure: TryInsertingTraitImplOnFailure,
    ) -> Result<DeclRefFunction, ErrorEmitted> {
        let decl_engine = self.engines.de();
        let type_engine = self.engines.te();

        let eq_check = UnifyCheck::non_dynamic_equality(self.engines);
        let coercion_check = UnifyCheck::coercion(self.engines);

        // default numeric types to u64
        if type_engine.contains_numeric(self.engines, type_id) {
            // While collecting unification we don't decay numeric and will ignore this error.
            if self.collecting_unifications {
                return Err(handler.emit_err(CompileError::MethodNotFound {
                    method: method_name.clone().as_str().to_string(),
                    type_name: self.engines.help_out(type_id).to_string(),
                    matching_method_strings: vec![],
                    span: method_name.span(),
                }));
            }
            type_engine.decay_numeric(handler, self.engines, type_id, &method_name.span())?;
        }

        let mut matching_item_decl_refs =
            self.find_items_for_type(handler, type_id, method_prefix, method_name)?;

        // This code path tries to get items of specific implementation of the annotation type that is a superset of type id.
        // This allows the following associated method to be found:
        // let _: Option<MyStruct<u64>> = MyStruct::try_from(my_u64);
        if !matches!(&*type_engine.get(annotation_type), TypeInfo::Unknown)
            && !type_id.is_concrete(self.engines, crate::TreatNumericAs::Concrete)
        {
            let inner_types =
                annotation_type.extract_inner_types(self.engines, crate::IncludeSelf::Yes);
            for inner_type_id in inner_types {
                if coercion_check.check(inner_type_id, type_id) {
                    matching_item_decl_refs.extend(self.find_items_for_type(
                        handler,
                        inner_type_id,
                        method_prefix,
                        method_name,
                    )?);
                }
            }
        }

        let mut matching_method_strings = HashSet::<String>::new();

        let matching_method_decl_refs = matching_item_decl_refs
            .into_iter()
            .flat_map(|item| match item {
                ty::TyTraitItem::Fn(decl_ref) => Some(decl_ref),
                ty::TyTraitItem::Constant(_) => None,
                ty::TyTraitItem::Type(_) => None,
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
                // Contract call methods don't have self parameter.
                let args_len_diff = if method.is_contract_call && !arguments_types.is_empty() {
                    1
                } else {
                    0
                };
                if method.parameters.len() == arguments_types.len() - args_len_diff
                    && method
                        .parameters
                        .iter()
                        .zip(arguments_types.iter().skip(args_len_diff))
                        .all(|(p, a)| coercion_check.check(*a, p.type_argument.type_id))
                    && (matches!(&*type_engine.get(annotation_type), TypeInfo::Unknown)
                        || matches!(
                            &*type_engine.get(method.return_type.type_id),
                            TypeInfo::Never
                        )
                        || coercion_check
                            .with_ignore_generic_names(true)
                            .check(annotation_type, method.return_type.type_id))
                {
                    maybe_method_decl_refs.push(decl_ref);
                }
            }

            if !maybe_method_decl_refs.is_empty() {
                let mut trait_methods = HashMap::<
                    (
                        CallPath,
                        Vec<WithEngines<TypeArgument>>,
                        Option<WithEngines<TypeInfo>>,
                    ),
                    DeclRefFunction,
                >::new();
                let mut impl_self_method = None;
                for method_ref in maybe_method_decl_refs.clone() {
                    let method = decl_engine.get_function(&method_ref);
                    if let Some(ty::TyDecl::ImplSelfOrTrait(impl_trait)) =
                        method.implementing_type.clone()
                    {
                        let trait_decl = decl_engine.get_impl_self_or_trait(&impl_trait.decl_id);
                        let mut skip_insert = false;
                        if let Some(as_trait) = as_trait {
                            if let TypeInfo::Custom {
                                qualified_call_path: call_path,
                                type_arguments,
                            } = &*type_engine.get(as_trait)
                            {
                                qualified_call_path = Some(call_path.clone());
                                // When `<S as Trait<T>>::method()` is used we only add methods to `trait_methods` that
                                // originate from the qualified trait.
                                if trait_decl.trait_name
                                    == call_path.clone().to_call_path(handler)?
                                {
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
                                                    handler,
                                                    p1.type_id,
                                                    &p1.span,
                                                    EnforceTypeArguments::Yes,
                                                    None,
                                                )?;
                                                let p2_type_id = self.resolve_type(
                                                    handler,
                                                    p2.type_id,
                                                    &p2.span,
                                                    EnforceTypeArguments::Yes,
                                                    None,
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
                                                trait_decl.trait_name.clone(),
                                                trait_decl
                                                    .trait_type_arguments
                                                    .iter()
                                                    .cloned()
                                                    .map(|a| self.engines.help_out(a))
                                                    .collect::<Vec<_>>(),
                                                method.implementing_for_typeid.map(|t| {
                                                    self.engines.help_out(
                                                        (*self.engines.te().get(t)).clone(),
                                                    )
                                                }),
                                            ),
                                            method_ref.clone(),
                                        );
                                    }
                                }
                                skip_insert = true;
                            }
                        }

                        let trait_methods_key = (
                            trait_decl.trait_name.clone(),
                            trait_decl
                                .trait_type_arguments
                                .iter()
                                .cloned()
                                .map(|a| self.engines.help_out(a))
                                .collect::<Vec<_>>(),
                            method.implementing_for_typeid.map(|t| {
                                self.engines.help_out((*self.engines.te().get(t)).clone())
                            }),
                        );

                        // If we have: impl<T> FromBytes for T
                        // and: impl FromBytes for DataPoint
                        // We pick the second implementation.
                        if let Some(existing_value) = trait_methods.get(&trait_methods_key) {
                            let existing_method = decl_engine.get_function(existing_value);
                            if let Some(ty::TyDecl::ImplSelfOrTrait(existing_impl_trait)) =
                                existing_method.implementing_type.clone()
                            {
                                let existing_trait_decl = decl_engine
                                    .get_impl_self_or_trait(&existing_impl_trait.decl_id);
                                if existing_trait_decl.impl_type_parameters.is_empty() {
                                    // We already have an impl without type parameters so we skip the others.
                                    skip_insert = true;
                                }
                            }
                        }

                        if !skip_insert {
                            trait_methods.insert(trait_methods_key, method_ref.clone());
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
                                },
                            )
                        }
                        let mut trait_strings = trait_methods
                            .keys()
                            .map(|t| {
                                (
                                    to_string(t.0.clone(), t.1.clone()),
                                    t.2.clone()
                                        .map(|t| t.to_string())
                                        .or_else(|| {
                                            Some(self.engines().help_out(type_id).to_string())
                                        })
                                        .unwrap(),
                                )
                            })
                            .collect::<Vec<(String, String)>>();
                        // Sort so the output of the error is always the same.
                        trait_strings.sort();
                        return Err(handler.emit_err(
                            CompileError::MultipleApplicableItemsInScope {
                                item_name: method_name.as_str().to_string(),
                                item_kind: "function".to_string(),
                                as_traits: trait_strings,
                                span: method_name.span(),
                            },
                        ));
                    }
                } else if qualified_call_path.is_some() {
                    // When we use a qualified path the expected method should be in trait_methods.
                    None
                } else {
                    maybe_method_decl_refs.first().cloned()
                }
            } else {
                for decl_ref in matching_method_decl_refs.clone().into_iter() {
                    let method = decl_engine.get_function(&decl_ref);
                    matching_method_strings.insert(format!(
                        "{}({}) -> {}{}",
                        method.name.as_str(),
                        method
                            .parameters
                            .iter()
                            .map(|p| self.engines.help_out(p.type_argument.type_id).to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                        self.engines.help_out(method.return_type.type_id),
                        if let Some(implementing_for_type_id) = method.implementing_for_typeid {
                            format!(" in {}", self.engines.help_out(implementing_for_type_id))
                        } else {
                            "".to_string()
                        }
                    ));
                }

                // When we can't match any method with parameter types we will throw an error.
                None
            }
        };

        if let Some(method_decl_ref) = matching_method_decl_ref {
            return Ok(method_decl_ref);
        }

        if let Some(TypeInfo::ErrorRecovery(err)) = arguments_types
            .front()
            .map(|x| (*type_engine.get(*x)).clone())
        {
            Err(err)
        } else {
            if matches!(
                try_inserting_trait_impl_on_failure,
                TryInsertingTraitImplOnFailure::Yes
            ) {
                // Retrieve the implemented traits for the type and insert them in the namespace.
                // insert_trait_implementation_for_type is done lazily only when required because of a failure.
                self.insert_trait_implementation_for_type(type_id);

                return self.find_method_for_type(
                    handler,
                    type_id,
                    method_prefix,
                    method_name,
                    annotation_type,
                    arguments_types,
                    as_trait,
                    TryInsertingTraitImplOnFailure::No,
                );
            }

            let type_name = if let Some(call_path) = qualified_call_path {
                format!(
                    "{} as {}",
                    self.engines.help_out(type_id),
                    call_path.call_path
                )
            } else {
                self.engines.help_out(type_id).to_string()
            };

            Err(handler.emit_err(CompileError::MethodNotFound {
                method: format!(
                    "{}({}){}",
                    method_name.clone(),
                    arguments_types
                        .iter()
                        .map(|a| self.engines.help_out(a).to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    if matches!(
                        *self.engines.te().get(self.type_annotation),
                        TypeInfo::Unknown
                    ) {
                        "".to_string()
                    } else {
                        format!(" -> {}", self.engines.help_out(self.type_annotation))
                    }
                ),
                type_name,
                matching_method_strings: matching_method_strings
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>(),
                span: method_name.span(),
            }))
        }
    }

    /// Short-hand for performing a [Module::star_import] with `mod_path` as the destination.
    pub(crate) fn star_import(
        &mut self,
        handler: &Handler,
        src: &ModulePath,
        visibility: Visibility,
    ) -> Result<(), ErrorEmitted> {
        let engines = self.engines;
        let mod_path = self.namespace().mod_path.clone();
        self.namespace_mut()
            .root
            .star_import(handler, engines, src, &mod_path, visibility)
    }

    /// Short-hand for performing a [Module::variant_star_import] with `mod_path` as the destination.
    pub(crate) fn variant_star_import(
        &mut self,
        handler: &Handler,
        src: &ModulePath,
        enum_name: &Ident,
        visibility: Visibility,
    ) -> Result<(), ErrorEmitted> {
        let engines = self.engines;
        let mod_path = self.namespace().mod_path.clone();
        self.namespace_mut()
            .root
            .variant_star_import(handler, engines, src, &mod_path, enum_name, visibility)
    }

    /// Short-hand for performing a [Module::self_import] with `mod_path` as the destination.
    pub(crate) fn self_import(
        &mut self,
        handler: &Handler,
        src: &ModulePath,
        alias: Option<Ident>,
        visibility: Visibility,
    ) -> Result<(), ErrorEmitted> {
        let engines = self.engines;
        let mod_path = self.namespace().mod_path.clone();
        self.namespace_mut()
            .root
            .self_import(handler, engines, src, &mod_path, alias, visibility)
    }

    // Import all impls for a struct/enum. Do nothing for other types.
    pub(crate) fn impls_import(&mut self, handler: &Handler, engines: &Engines, type_id: TypeId) {
        let type_info = engines.te().get(type_id);

        let decl_call_path = match &*type_info {
            TypeInfo::Enum(decl_id) => {
                let decl = engines.de().get(decl_id);
                decl.call_path.clone()
            }
            TypeInfo::Struct(decl_id) => {
                let decl = engines.de().get(decl_id);
                decl.call_path.clone()
            }
            _ => return,
        };

        let mut impls_to_insert = TraitMap::default();

        let root_mod = &self.namespace().root().module;
        let Ok(src_mod) = root_mod.lookup_submodule(handler, engines, &decl_call_path.prefixes)
        else {
            return;
        };

        impls_to_insert.extend(
            src_mod
                .current_items()
                .implemented_traits
                .filter_by_type_item_import(type_id, engines, self.code_block_first_pass().into()),
            engines,
        );

        let dst_mod = self.namespace_mut().module_mut(engines);
        dst_mod
            .current_items_mut()
            .implemented_traits
            .extend(impls_to_insert, engines);
    }

    /// Short-hand for performing a [Module::item_import] with `mod_path` as the destination.
    pub(crate) fn item_import(
        &mut self,
        handler: &Handler,
        src: &ModulePath,
        item: &Ident,
        alias: Option<Ident>,
        visibility: Visibility,
    ) -> Result<(), ErrorEmitted> {
        let engines = self.engines;
        let mod_path = self.namespace().mod_path.clone();
        self.namespace_mut()
            .root
            .item_import(handler, engines, src, item, &mod_path, alias, visibility)
    }

    /// Short-hand for performing a [Module::variant_import] with `mod_path` as the destination.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn variant_import(
        &mut self,
        handler: &Handler,
        src: &ModulePath,
        enum_name: &Ident,
        variant_name: &Ident,
        alias: Option<Ident>,
        visibility: Visibility,
    ) -> Result<(), ErrorEmitted> {
        let engines = self.engines;
        let mod_path = self.namespace().mod_path.clone();
        self.namespace_mut().root.variant_import(
            handler,
            engines,
            src,
            enum_name,
            variant_name,
            &mod_path,
            alias,
            visibility,
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
        is_impl_self: IsImplSelf,
        is_extending_existing_impl: IsExtendingExistingImpl,
    ) -> Result<(), ErrorEmitted> {
        // Use trait name with full path, improves consistency between
        // this inserting and getting in `get_methods_for_type_and_trait_name`.
        let full_trait_name = trait_name.to_fullpath(self.engines(), self.namespace());
        let engines = self.engines;
        let items = items
            .iter()
            .map(|item| ResolvedTraitImplItem::Typed(item.clone()))
            .collect::<Vec<_>>();
        self.namespace_mut()
            .module_mut(engines)
            .current_items_mut()
            .implemented_traits
            .insert(
                handler,
                full_trait_name,
                trait_type_args,
                type_id,
                &items,
                impl_span,
                trait_decl_span,
                is_impl_self,
                is_extending_existing_impl,
                engines,
            )
    }

    pub(crate) fn get_items_for_type_and_trait_name(
        &self,
        type_id: TypeId,
        trait_name: &CallPath,
    ) -> Vec<ty::TyTraitItem> {
        self.get_items_for_type_and_trait_name_and_trait_type_arguments(type_id, trait_name, &[])
    }

    pub(crate) fn get_items_for_type_and_trait_name_and_trait_type_arguments(
        &self,
        type_id: TypeId,
        trait_name: &CallPath,
        trait_type_args: &[TypeArgument],
    ) -> Vec<ty::TyTraitItem> {
        // Use trait name with full path, improves consistency between
        // this get and inserting in `insert_trait_implementation`.
        let trait_name = trait_name.to_fullpath(self.engines(), self.namespace());

        self.namespace()
            .module(self.engines())
            .current_items()
            .implemented_traits
            .get_items_for_type_and_trait_name_and_trait_type_arguments_typed(
                self.engines,
                type_id,
                &trait_name,
                trait_type_args,
            )
    }

    pub(crate) fn insert_trait_implementation_for_type(&mut self, type_id: TypeId) {
        let engines = self.engines;
        let code_block_first_pass = self.code_block_first_pass();
        self.namespace_mut()
            .module_mut(engines)
            .current_items_mut()
            .implemented_traits
            .insert_for_type(engines, type_id, code_block_first_pass.into());
    }

    pub fn check_type_impls_traits(
        &mut self,
        type_id: TypeId,
        constraints: &[TraitConstraint],
    ) -> bool {
        let handler = Handler::default();
        let engines = self.engines;
        let code_block_first_pass = self.code_block_first_pass();
        self.namespace_mut()
            .module_mut(engines)
            .current_items_mut()
            .implemented_traits
            .check_if_trait_constraints_are_satisfied_for_type(
                &handler,
                type_id,
                constraints,
                &Span::dummy(),
                engines,
                crate::namespace::TryInsertingTraitImplOnFailure::Yes,
                code_block_first_pass.into(),
            )
            .is_ok()
    }
}
