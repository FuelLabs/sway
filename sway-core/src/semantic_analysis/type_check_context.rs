#![allow(clippy::mutable_key_type)]
use std::collections::BTreeMap;

use crate::{
    decl_engine::{DeclEngineGet, MaterializeConstGenerics},
    engine_threading::*,
    language::{
        parsed::TreeType,
        ty::{self, TyDecl, TyExpression},
        CallPath, QualifiedCallPath, Visibility,
    },
    monomorphization::{monomorphize_with_modpath, MonomorphizeHelper},
    namespace::{
        IsExtendingExistingImpl, IsImplSelf, ModulePath, ResolvedDeclaration,
        ResolvedTraitImplItem, TraitMap,
    },
    semantic_analysis::{
        ast_node::{AbiMode, ConstShadowingMode},
        Namespace,
    },
    type_system::{GenericArgument, SubstTypes, TypeId, TypeInfo},
    EnforceTypeArguments, SubstTypesContext, TraitConstraint, TypeParameter, TypeSubstMap,
};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_features::ExperimentalFeatures;
use sway_types::{span::Span, Ident};

use super::{
    namespace::{IsImplInterfaceSurface, Items, LexicalScopeId},
    symbol_collection_context::SymbolCollectionContext,
    type_resolve::{resolve_call_path, resolve_qualified_call_path, resolve_type, VisibilityCheck},
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

    pub engines: &'a Engines,

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
    pub(crate) const_shadowing_mode: ConstShadowingMode,
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
            type_annotation: engines.te().new_unknown(),
            function_type_annotation: engines.te().new_unknown(),
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
            type_annotation: engines.te().new_unknown(),
            function_type_annotation: engines.te().new_unknown(),
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

    /// Scope the `TypeCheckContext` with a new lexical scope, and set up the collection context
    /// so it enters the lexical scope corresponding to the given span.
    pub fn scoped<T>(
        &mut self,
        handler: &Handler,
        span: Option<Span>,
        with_scoped_ctx: impl FnOnce(&mut TypeCheckContext) -> Result<T, ErrorEmitted>,
    ) -> Result<T, ErrorEmitted> {
        self.scoped_and_lexical_scope_id(handler, span, with_scoped_ctx)
            .0
    }

    /// Scope the `TypeCheckContext` with a new lexical scope, and set up the collection context
    /// so it enters the lexical scope corresponding to the given span.
    pub fn scoped_and_lexical_scope_id<T>(
        &mut self,
        handler: &Handler,
        span: Option<Span>,
        with_scoped_ctx: impl FnOnce(&mut TypeCheckContext) -> Result<T, ErrorEmitted>,
    ) -> (Result<T, ErrorEmitted>, LexicalScopeId) {
        let engines = self.engines;
        if let Some(span) = span {
            self.namespace_scoped(engines, |ctx| {
                ctx.collection_ctx.enter_lexical_scope(
                    handler,
                    ctx.engines,
                    span,
                    |scoped_collection_ctx| {
                        let mut ctx = TypeCheckContext {
                            collection_ctx: scoped_collection_ctx,
                            namespace: ctx.namespace,
                            type_annotation: ctx.type_annotation,
                            function_type_annotation: ctx.function_type_annotation,
                            unify_generic: ctx.unify_generic,
                            self_type: ctx.self_type,
                            type_subst: ctx.type_subst.clone(),
                            abi_mode: ctx.abi_mode.clone(),
                            const_shadowing_mode: ctx.const_shadowing_mode,
                            generic_shadowing_mode: ctx.generic_shadowing_mode,
                            help_text: ctx.help_text,
                            kind: ctx.kind,
                            engines: ctx.engines,
                            disallow_functions: ctx.disallow_functions,
                            storage_declaration: ctx.storage_declaration,
                            experimental: ctx.experimental,
                            collecting_unifications: ctx.collecting_unifications,
                            code_block_first_pass: ctx.code_block_first_pass,
                        };
                        with_scoped_ctx(&mut ctx)
                    },
                )
            })
        } else {
            self.namespace_scoped(engines, |ctx| with_scoped_ctx(ctx))
        }
    }

    /// Scope the `CollectionContext` with a new lexical scope.
    pub fn namespace_scoped<T>(
        &mut self,
        engines: &Engines,
        with_scoped_ctx: impl FnOnce(&mut TypeCheckContext) -> Result<T, ErrorEmitted>,
    ) -> (Result<T, ErrorEmitted>, LexicalScopeId) {
        let lexical_scope_id: LexicalScopeId = self
            .namespace
            .current_module_mut()
            .write(engines, |m| m.push_new_lexical_scope(Span::dummy(), None));
        let ret = with_scoped_ctx(self);
        self.namespace
            .current_module_mut()
            .write(engines, |m| m.pop_lexical_scope());
        (ret, lexical_scope_id)
    }

    /// Enter the submodule with the given name and produce a type-check context ready for
    /// type-checking its content.
    ///
    /// Returns the result of the given `with_submod_ctx` function.
    pub fn enter_submodule<T>(
        &mut self,
        handler: &Handler,
        mod_name: Ident,
        visibility: Visibility,
        module_span: Span,
        with_submod_ctx: impl FnOnce(TypeCheckContext) -> T,
    ) -> Result<T, ErrorEmitted> {
        let experimental = self.experimental;

        // We're checking a submodule, so no need to pass through anything other than the
        // namespace and the engines.
        let engines = self.engines;
        self.namespace.enter_submodule(
            handler,
            engines,
            mod_name.clone(),
            visibility,
            module_span.clone(),
            true,
        )?;

        self.collection_ctx.enter_submodule(
            handler,
            engines,
            mod_name,
            visibility,
            module_span,
            |submod_collection_ctx| {
                let submod_ctx = TypeCheckContext::from_namespace(
                    self.namespace,
                    submod_collection_ctx,
                    engines,
                    experimental,
                );
                let ret = with_submod_ctx(submod_ctx);
                self.namespace.pop_submodule();
                ret
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

    pub(crate) fn subst_ctx(&self) -> SubstTypesContext<'_, '_> {
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
        type_arguments: &mut [GenericArgument],
        const_generics: BTreeMap<String, TyExpression>,
        enforce_type_arguments: EnforceTypeArguments,
        call_site_span: &Span,
    ) -> Result<(), ErrorEmitted>
    where
        T: MonomorphizeHelper + SubstTypes + MaterializeConstGenerics,
    {
        let mod_path = self.namespace().current_mod_path().clone();
        monomorphize_with_modpath(
            handler,
            self.engines(),
            self.namespace(),
            value,
            type_arguments,
            const_generics,
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
                || None,
            )
        } else {
            self.engines.te().unify(
                handler,
                self.engines(),
                ty,
                self.type_annotation(),
                span,
                self.help_text(),
                || None,
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

        Items::insert_symbol(
            handler,
            engines,
            self.namespace_mut().current_module_mut(),
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
        let mod_path = self.namespace().current_mod_path().clone();
        resolve_type(
            handler,
            self.engines(),
            self.namespace(),
            &mod_path,
            type_id,
            span,
            enforce_type_arguments,
            type_info_prefix,
            self.self_type(),
            &self.subst_ctx(),
            VisibilityCheck::Yes,
        )
    }

    pub(crate) fn resolve_qualified_call_path(
        &mut self,
        handler: &Handler,
        qualified_call_path: &QualifiedCallPath,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        resolve_qualified_call_path(
            handler,
            self.engines(),
            self.namespace(),
            &self.namespace().current_mod_path().clone(),
            qualified_call_path,
            self.self_type(),
            &self.subst_ctx(),
            VisibilityCheck::Yes,
        )
        .map(|d| d.expect_typed())
    }

    /// Short-hand for calling [Root::resolve_symbol] on `root` with the `mod_path`.
    pub(crate) fn resolve_symbol(
        &self,
        handler: &Handler,
        symbol: &Ident,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        resolve_call_path(
            handler,
            self.engines(),
            self.namespace(),
            self.namespace().current_mod_path(),
            &symbol.clone().into(),
            self.self_type(),
            VisibilityCheck::No,
        )
        .map(|d| d.expect_typed())
    }

    /// Short-hand for calling [Root::resolve_call_path_with_visibility_check] on `root` with the `mod_path`.
    pub(crate) fn resolve_call_path_with_visibility_check(
        &self,
        handler: &Handler,
        call_path: &CallPath,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        resolve_call_path(
            handler,
            self.engines(),
            self.namespace(),
            self.namespace().current_mod_path(),
            call_path,
            self.self_type(),
            VisibilityCheck::Yes,
        )
        .map(|d| d.expect_typed())
    }

    /// Short-hand for calling [Root::resolve_call_path] on `root` with the `mod_path`.
    pub(crate) fn resolve_call_path(
        &self,
        handler: &Handler,
        call_path: &CallPath,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        resolve_call_path(
            handler,
            self.engines(),
            self.namespace(),
            self.namespace().current_mod_path(),
            call_path,
            self.self_type(),
            VisibilityCheck::No,
        )
        .map(|d| d.expect_typed())
    }

    /// Short-hand for performing a [Module::star_import] with `mod_path` as the destination.
    pub(crate) fn star_import(
        &mut self,
        handler: &Handler,
        src: &ModulePath,
        visibility: Visibility,
    ) -> Result<(), ErrorEmitted> {
        let engines = self.engines;
        self.namespace_mut()
            .star_import_to_current_module(handler, engines, src, visibility)
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
        self.namespace_mut()
            .variant_star_import_to_current_module(handler, engines, src, enum_name, visibility)
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
        self.namespace_mut()
            .self_import_to_current_module(handler, engines, src, alias, visibility)
    }

    // Import all impls for a struct/enum. Do nothing for other types.
    pub(crate) fn impls_import(&mut self, engines: &Engines, type_id: TypeId) {
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

        let Some(src_mod) = &self
            .namespace()
            .module_from_absolute_path(&decl_call_path.prefixes)
        else {
            return;
        };

        let _ = src_mod.walk_scope_chain_early_return(|lexical_scope| {
            impls_to_insert.extend(
                lexical_scope
                    .items
                    .implemented_traits
                    .filter_by_type_item_import(type_id, engines),
                engines,
            );
            Ok(None::<()>)
        });

        let dst_mod = self.namespace_mut().current_module_mut();
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
        self.namespace_mut()
            .item_import_to_current_module(handler, engines, src, item, alias, visibility)
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
        self.namespace_mut().variant_import_to_current_module(
            handler,
            engines,
            src,
            enum_name,
            variant_name,
            alias,
            visibility,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn insert_trait_implementation(
        &mut self,
        handler: &Handler,
        trait_name: CallPath,
        trait_type_args: Vec<GenericArgument>,
        type_id: TypeId,
        mut impl_type_parameters: Vec<TypeParameter>,
        items: &[ty::TyImplItem],
        impl_span: &Span,
        trait_decl_span: Option<Span>,
        is_impl_self: IsImplSelf,
        is_extending_existing_impl: IsExtendingExistingImpl,
        is_impl_interface_surface: IsImplInterfaceSurface,
    ) -> Result<(), ErrorEmitted> {
        let engines = self.engines;

        // Use trait name with full path, improves consistency between
        // this inserting and getting in `get_methods_for_type_and_trait_name`.
        for tc in impl_type_parameters
            .iter_mut()
            .filter_map(|x| x.as_type_parameter_mut())
            .flat_map(|x| x.trait_constraints.iter_mut())
        {
            tc.trait_name = tc.trait_name.to_fullpath(self.engines(), self.namespace())
        }

        let impl_type_parameters_ids = impl_type_parameters
            .iter()
            .map(|type_parameter| engines.te().new_type_param(type_parameter.clone()))
            .collect::<Vec<_>>();

        // CallPath::to_fullpath gives a resolvable path, but is not guaranteed to provide the path
        // to the actual trait declaration. Since the path of the trait declaration is used as a key
        // in the trait map, we need to find the actual declaration path.
        let canonical_trait_path = trait_name.to_canonical_path(self.engines(), self.namespace());

        let items = items
            .iter()
            .map(|item| ResolvedTraitImplItem::Typed(item.clone()))
            .collect::<Vec<_>>();
        self.namespace_mut()
            .current_module_mut()
            .current_items_mut()
            .implemented_traits
            .insert(
                handler,
                canonical_trait_path,
                trait_type_args,
                type_id,
                impl_type_parameters_ids,
                &items,
                impl_span,
                trait_decl_span,
                is_impl_self,
                is_extending_existing_impl,
                is_impl_interface_surface,
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
        trait_type_args: &[GenericArgument],
    ) -> Vec<ty::TyTraitItem> {
        // CallPath::to_fullpath gives a resolvable path, but is not guaranteed to provide the path
        // to the actual trait declaration. Since the path of the trait declaration is used as a key
        // in the trait map, we need to find the actual declaration path.
        let canonical_trait_path = trait_name.to_canonical_path(self.engines(), self.namespace());

        TraitMap::get_items_for_type_and_trait_name_and_trait_type_arguments_typed(
            self.namespace().current_module(),
            self.engines,
            type_id,
            &canonical_trait_path,
            trait_type_args,
        )
    }

    pub fn check_type_impls_traits(
        &mut self,
        type_id: TypeId,
        constraints: &[TraitConstraint],
    ) -> bool {
        let handler = Handler::default();
        let engines = self.engines;
        TraitMap::check_if_trait_constraints_are_satisfied_for_type(
            &handler,
            self.namespace_mut().current_module_mut(),
            type_id,
            constraints,
            &Span::dummy(),
            engines,
        )
        .is_ok()
    }
}
