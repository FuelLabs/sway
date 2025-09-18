#![allow(clippy::mutable_key_type)]
use std::collections::{BTreeMap, HashSet};

use crate::{
    ast_elements::type_argument::GenericTypeArgument,
    decl_engine::{DeclEngineGet, DeclRefFunction, MaterializeConstGenerics},
    engine_threading::*,
    language::{
        parsed::{MethodName, TreeType},
        ty::{self, TyDecl, TyExpression},
        CallPath, QualifiedCallPath, Visibility,
    },
    monomorphization::{monomorphize_with_modpath, MonomorphizeHelper},
    namespace::{
        IsExtendingExistingImpl, IsImplSelf, Module, ModulePath, ResolvedDeclaration,
        ResolvedTraitImplItem, TraitKey, TraitMap, TraitSuffix,
    },
    semantic_analysis::{
        ast_node::{AbiMode, ConstShadowingMode},
        Namespace,
    },
    type_system::{GenericArgument, SubstTypes, TypeId, TypeInfo},
    EnforceTypeArguments, SubstTypesContext, TraitConstraint, TypeParameter, TypeSubstMap,
    UnifyCheck,
};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_features::ExperimentalFeatures;
use sway_types::{span::Span, Ident, Spanned};

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

struct MethodCandidate {
    decl_ref: DeclRefFunction,
    params: Vec<TypeId>,
    ret: TypeId,
    is_contract_call: bool,
}

enum MatchScore {
    Exact,
    Coercible,
    Incompatible,
}

#[derive(Clone)]
pub(crate) struct CandidateTraitItem {
    item: ty::TyTraitItem,
    trait_key: TraitKey,
    original_type_id: TypeId,
    resolved_type_id: TypeId,
}

fn trait_paths_equivalent(
    allowed: &CallPath<TraitSuffix>,
    other: &CallPath<TraitSuffix>,
    unify_check: &UnifyCheck,
) -> bool {
    if allowed
        .prefixes
        .iter()
        .zip(other.prefixes.iter())
        .any(|(a, b)| a != b)
    {
        return false;
    }
    if allowed.suffix.name != other.suffix.name {
        return false;
    }
    if allowed.suffix.args.len() != other.suffix.args.len() {
        return false;
    }
    allowed
        .suffix
        .args
        .iter()
        .zip(other.suffix.args.iter())
        .all(|(a, b)| unify_check.check(a.type_id(), b.type_id()))
}

type TraitImplId = crate::decl_engine::DeclId<ty::TyImplSelfOrTrait>;
type GroupingKey = (TraitImplId, Option<TypeId>);

struct GroupingResult {
    trait_methods: BTreeMap<GroupingKey, DeclRefFunction>,
    impl_self_method: Option<DeclRefFunction>,
    qualified_call_path: Option<QualifiedCallPath>,
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

    /// Given a name and a type (plus a `self_type` to potentially
    /// resolve it), find items matching in the namespace.
    pub(crate) fn find_items_for_type(
        &self,
        handler: &Handler,
        type_id: TypeId,
        item_prefix: &ModulePath,
        item_name: &Ident,
        method_name: &Option<&MethodName>,
    ) -> Result<Vec<CandidateTraitItem>, ErrorEmitted> {
        let type_engine = self.engines.te();
        let original_type_id = type_id;

        // If the type that we are looking for is the error recovery type, then
        // we want to return the error case without creating a new error
        // message.
        if let TypeInfo::ErrorRecovery(err) = &*type_engine.get(type_id) {
            return Err(*err);
        }

        // resolve the type
        let resolved_type_id = resolve_type(
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
            VisibilityCheck::Yes,
        )
        .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));

        // grab the local module
        let local_module = self
            .namespace()
            .require_module_from_absolute_path(handler, &self.namespace().current_mod_path)?;

        // grab the local items from the local module
        let mut matching_items = vec![];
        let mut filter_item = |item: ResolvedTraitImplItem, trait_key: TraitKey| match &item {
            ResolvedTraitImplItem::Parsed(_) => todo!(),
            ResolvedTraitImplItem::Typed(ty_item) => match ty_item {
                ty::TyTraitItem::Fn(decl_ref) if decl_ref.name() == item_name => {
                    matching_items.push(CandidateTraitItem {
                        item: ty_item.clone(),
                        trait_key: trait_key.clone(),
                        original_type_id,
                        resolved_type_id,
                    });
                }
                ty::TyTraitItem::Constant(decl_ref) if decl_ref.name() == item_name => {
                    matching_items.push(CandidateTraitItem {
                        item: ty_item.clone(),
                        trait_key: trait_key.clone(),
                        original_type_id,
                        resolved_type_id,
                    });
                }
                ty::TyTraitItem::Type(decl_ref) if decl_ref.name() == item_name => {
                    matching_items.push(CandidateTraitItem {
                        item: ty_item.clone(),
                        trait_key: trait_key.clone(),
                        original_type_id,
                        resolved_type_id,
                    });
                }
                _ => {}
            },
        };

        TraitMap::find_items_and_trait_key_for_type(
            local_module,
            self.engines,
            resolved_type_id,
            &mut filter_item,
        );

        // grab the items from where the argument type is declared
        if let Some(MethodName::FromTrait { .. }) = method_name {
            let type_module = self.get_namespace_module_from_type_id(resolved_type_id);
            if let Ok(type_module) = type_module {
                TraitMap::find_items_and_trait_key_for_type(
                    type_module,
                    self.engines,
                    resolved_type_id,
                    &mut filter_item,
                );
            }
        }

        if item_prefix != self.namespace().current_mod_path.as_slice() {
            // grab the module where the type itself is declared
            let type_module = self
                .namespace()
                .require_module_from_absolute_path(handler, item_prefix)?;

            // grab the items from where the type is declared
            TraitMap::find_items_and_trait_key_for_type(
                type_module,
                self.engines,
                resolved_type_id,
                &mut filter_item,
            );
        }

        Ok(matching_items)
    }

    fn get_namespace_module_from_type_id(&self, type_id: TypeId) -> Result<&Module, ErrorEmitted> {
        let type_info = self.engines().te().get(type_id);
        if type_info.is_alias() {
            if let TypeInfo::Alias { ty, .. } = &*type_info {
                if let Some(GenericTypeArgument { type_id, .. }) = ty.as_type_argument() {
                    return self.get_namespace_module_from_type_id(*type_id);
                }
            }
        }

        let handler = Handler::default();
        let call_path = match *type_info {
            TypeInfo::Enum(decl_id) => self.engines().de().get_enum(&decl_id).call_path.clone(),
            TypeInfo::Struct(decl_id) => self.engines().de().get_struct(&decl_id).call_path.clone(),
            _ => {
                return Err(handler.emit_err(CompileError::Internal(
                    "No call path for type id",
                    Span::dummy(),
                )))
            }
        };

        let call_path = call_path.rshift();
        self.namespace()
            .require_module_from_absolute_path(&handler, &call_path.as_vec_ident())
    }

    #[inline]
    fn default_numeric_if_needed(
        &self,
        handler: &Handler,
        type_id: TypeId,
        method_name: &Ident,
    ) -> Result<(), ErrorEmitted> {
        let type_engine = self.engines.te();

        // Default numeric types to u64
        if type_engine.contains_numeric(self.engines, type_id) {
            // While collecting unifications we don't decay numeric and will ignore this error.
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

        Ok(())
    }

    /// Collect all candidate trait items that might provide the method:
    /// - Items directly available for `type_id`
    /// - Plus items from any annotation-type inner that can coerce to `type_id`
    fn collect_candidate_items(
        &self,
        handler: &Handler,
        type_id: TypeId,
        method_prefix: &ModulePath,
        method_ident: &Ident,
        annotation_type: TypeId,
        method_name: &Option<&MethodName>,
    ) -> Result<Vec<ty::TyTraitItem>, ErrorEmitted> {
        let type_engine = self.engines.te();

        // Start with items for the concrete type.
        let mut items =
            self.find_items_for_type(handler, type_id, method_prefix, method_ident, method_name)?;

        if method_name.is_none() {
            return Ok(items.into_iter().map(|candidate| candidate.item).collect());
        }

        // Consider items from supersets indicated by the annotation return type.
        if !matches!(&*type_engine.get(annotation_type), TypeInfo::Unknown)
            && !type_id.is_concrete(self.engines, crate::TreatNumericAs::Concrete)
        {
            let coercion_check = UnifyCheck::coercion(self.engines).with_ignore_generic_names(true);

            let inner_types =
                annotation_type.extract_inner_types(self.engines, crate::IncludeSelf::Yes);

            for inner in inner_types {
                if coercion_check.check(inner, type_id) {
                    items.extend(self.find_items_for_type(
                        handler,
                        inner,
                        method_prefix,
                        method_ident,
                        method_name,
                    )?);
                }
            }
        }

        if let Some(method_name) = method_name {
            let method_constraints = self.trait_constraints_from_method_name(handler, method_name);
            self.filter_items_by_trait_access(&mut items, method_constraints.as_ref());
        }

        Ok(items.into_iter().map(|candidate| candidate.item).collect())
    }

    fn trait_constraints_from_method_name(
        &self,
        handler: &Handler,
        method_name: &MethodName,
    ) -> Option<(TypeId, Vec<TraitConstraint>)> {
        let _ = handler;
        let MethodName::FromType {
            call_path_binding, ..
        } = method_name
        else {
            return None;
        };

        let (_, type_ident) = &call_path_binding.inner.suffix;
        let resolve_handler = Handler::default();
        let Ok((resolved_decl, _)) = self.namespace().current_module().resolve_symbol(
            &resolve_handler,
            self.engines(),
            type_ident,
        ) else {
            return None;
        };

        match resolved_decl.expect_typed() {
            ty::TyDecl::GenericTypeForFunctionScope(ty::GenericTypeForFunctionScope {
                type_id,
                ..
            }) => {
                let mut constraints = Vec::new();
                let mut visited = HashSet::new();
                self.collect_trait_constraints_recursive(type_id, &mut constraints, &mut visited);
                Some((type_id, constraints))
            }
            _ => None,
        }
    }

    /// Filter the candidate trait items so that only methods whose traits satisfy the relevant
    /// trait bounds remain. Groups corresponding to other generic parameters are left untouched.
    fn filter_items_by_trait_access(
        &self,
        items: &mut Vec<CandidateTraitItem>,
        method_constraints: Option<&(TypeId, Vec<TraitConstraint>)>,
    ) {
        if items.is_empty() {
            return;
        }

        // Group candidates by the (possibly still generic) type they originated from so we can
        // later apply trait bounds per generic parameter.
        let mut grouped: BTreeMap<TypeId, Vec<CandidateTraitItem>> = BTreeMap::new();
        for item in items.drain(..) {
            grouped
                .entry(
                    self.engines
                        .te()
                        .get_unaliased_type_id(item.resolved_type_id),
                )
                .or_default()
                .push(item);
        }

        // If this lookup is resolving a concrete method name, pre-compute the generic type whose
        // bounds we collected so we only apply those bounds to matching groups.
        let method_constraint_info = method_constraints.map(|(type_id, constraints)| {
            (
                self.engines.te().get_unaliased_type_id(*type_id),
                constraints.as_slice(),
            )
        });

        let type_engine = self.engines.te();
        let mut filtered = Vec::new();

        for (type_id, group) in grouped {
            let type_info = type_engine.get(type_id);
            if !matches!(
                *type_info,
                TypeInfo::UnknownGeneric { .. } | TypeInfo::Placeholder(_)
            ) {
                filtered.extend(group);
                continue;
            }

            let (interface_items, impl_items): (Vec<_>, Vec<_>) =
                group.into_iter().partition(|item| {
                    matches!(
                        item.trait_key.is_impl_interface_surface,
                        IsImplInterfaceSurface::Yes
                    )
                });

            // Only groups born from the same generic parameter as the method call need to honour
            // the method's trait bounds. Other generic parameters can pass through untouched.
            let extra_constraints =
                method_constraint_info.and_then(|(constraint_type_id, constraints)| {
                    let applies_to_group =
                        interface_items.iter().chain(impl_items.iter()).any(|item| {
                            self.engines
                                .te()
                                .get_unaliased_type_id(item.original_type_id)
                                == constraint_type_id
                        });

                    applies_to_group.then_some(constraints)
                });

            let allowed_traits =
                self.allowed_traits_for_type(type_id, &interface_items, extra_constraints);

            if allowed_traits.is_empty() {
                filtered.extend(interface_items);
                filtered.extend(impl_items);
                continue;
            }

            if !impl_items.is_empty() {
                let mut retained_impls = Vec::new();
                for item in impl_items {
                    if self.trait_key_matches_allowed(&item.trait_key, &allowed_traits) {
                        retained_impls.push(item);
                    }
                }

                if !retained_impls.is_empty() {
                    filtered.extend(retained_impls);
                    filtered.extend(interface_items);
                    continue;
                }
            }

            // No impl methods matched the bounds, so fall back to the interface placeholders.
            filtered.extend(interface_items);
        }

        *items = filtered;
    }

    /// Build the list of trait paths that should remain visible for the given `type_id` when
    /// resolving a method. This includes traits that supplied the interface surface entries as well
    /// as any traits required by bounds on the generic parameter.
    fn allowed_traits_for_type(
        &self,
        type_id: TypeId,
        interface_items: &[CandidateTraitItem],
        extra_constraints: Option<&[TraitConstraint]>,
    ) -> Vec<CallPath<TraitSuffix>> {
        // Seed the allow-list with the traits that provided the interface items. They act as
        // fallbacks whenever no concrete implementation matches the bounds.
        let mut allowed: Vec<CallPath<TraitSuffix>> = interface_items
            .iter()
            .map(|item| item.trait_key.name.as_ref().clone())
            .collect();

        // Add trait bounds declared on the type parameter itself (recursively following inherited
        // bounds) so they can participate in disambiguation.
        let mut constraints = Vec::new();
        let mut visited = HashSet::new();
        self.collect_trait_constraints_recursive(type_id, &mut constraints, &mut visited);

        for constraint in constraints {
            let canonical = constraint
                .trait_name
                .to_canonical_path(self.engines(), self.namespace());
            allowed.push(CallPath {
                prefixes: canonical.prefixes,
                suffix: TraitSuffix {
                    name: canonical.suffix,
                    args: constraint.type_arguments.clone(),
                },
                callpath_type: canonical.callpath_type,
            });
        }

        // Method-specific bounds (for example from `fn foo<T: Trait>()`) are supplied separately,
        // include them so only the permitted traits remain candidates after filtering.
        if let Some(extra) = extra_constraints {
            for constraint in extra {
                let canonical = constraint
                    .trait_name
                    .to_canonical_path(self.engines(), self.namespace());
                allowed.push(CallPath {
                    prefixes: canonical.prefixes,
                    suffix: TraitSuffix {
                        name: canonical.suffix,
                        args: constraint.type_arguments.clone(),
                    },
                    callpath_type: canonical.callpath_type,
                });
            }
        }

        self.dedup_allowed_traits(allowed)
    }

    fn dedup_allowed_traits(
        &self,
        allowed: Vec<CallPath<TraitSuffix>>,
    ) -> Vec<CallPath<TraitSuffix>> {
        let mut deduped = Vec::new();
        let unify_check = UnifyCheck::constraint_subset(self.engines);

        for entry in allowed.into_iter() {
            if deduped
                .iter()
                .any(|existing| trait_paths_equivalent(existing, &entry, &unify_check))
            {
                continue;
            }
            deduped.push(entry);
        }

        deduped
    }

    /// Recursively collect trait constraints that apply to `type_id`, following aliases,
    /// placeholders, and chains of generic parameters.
    fn collect_trait_constraints_recursive(
        &self,
        type_id: TypeId,
        acc: &mut Vec<TraitConstraint>,
        visited: &mut HashSet<TypeId>,
    ) {
        let type_engine = self.engines.te();
        let type_id = type_engine.get_unaliased_type_id(type_id);
        if !visited.insert(type_id) {
            return;
        }

        match &*type_engine.get(type_id) {
            TypeInfo::UnknownGeneric {
                trait_constraints,
                parent,
                ..
            } => {
                acc.extend(trait_constraints.iter().cloned());
                if let Some(parent_id) = parent {
                    self.collect_trait_constraints_recursive(*parent_id, acc, visited);
                }
            }
            TypeInfo::Placeholder(TypeParameter::Type(generic)) => {
                acc.extend(generic.trait_constraints.iter().cloned());
                self.collect_trait_constraints_recursive(generic.type_id, acc, visited);
            }
            TypeInfo::Alias { ty, .. } => {
                if let Some(GenericTypeArgument { type_id: inner, .. }) = ty.as_type_argument() {
                    self.collect_trait_constraints_recursive(*inner, acc, visited);
                }
            }
            _ => {}
        }
    }

    fn retain_trait_methods_matching_constraints(
        &self,
        trait_methods: &mut BTreeMap<GroupingKey, DeclRefFunction>,
        constraints: &[TraitConstraint],
    ) {
        if constraints.is_empty() {
            return;
        }

        let allowed_traits = constraints
            .iter()
            .map(|constraint| {
                let canonical = constraint
                    .trait_name
                    .to_canonical_path(self.engines(), self.namespace());
                CallPath {
                    prefixes: canonical.prefixes,
                    suffix: TraitSuffix {
                        name: canonical.suffix,
                        args: constraint.type_arguments.clone(),
                    },
                    callpath_type: canonical.callpath_type,
                }
            })
            .collect::<Vec<_>>();

        if allowed_traits.is_empty() {
            return;
        }

        let mut filtered = trait_methods.clone();
        let unify_check = UnifyCheck::constraint_subset(self.engines);

        filtered.retain(|(_impl_id, _), decl_ref| {
            let method = self.engines.de().get_function(decl_ref);
            let Some(ty::TyDecl::ImplSelfOrTrait(impl_ref)) = method.implementing_type.as_ref()
            else {
                return true;
            };

            let impl_decl = self.engines.de().get_impl_self_or_trait(&impl_ref.decl_id);

            // Inherent impls have no trait declaration, keep them untouched.
            if impl_decl.trait_decl_ref.is_none() {
                return true;
            }

            // Build the canonical trait path and check whether it matches any of the trait bounds
            // collected for this lookup. Only methods provided by traits that satisfy the bounds
            // remain candidates for disambiguation.
            let canonical = impl_decl
                .trait_name
                .to_canonical_path(self.engines(), self.namespace());
            let candidate = CallPath {
                prefixes: canonical.prefixes,
                suffix: TraitSuffix {
                    name: canonical.suffix,
                    args: impl_decl.trait_type_arguments.clone(),
                },
                callpath_type: canonical.callpath_type,
            };

            allowed_traits
                .iter()
                .any(|allowed| trait_paths_equivalent(allowed, &candidate, &unify_check))
        });

        if !filtered.is_empty() {
            *trait_methods = filtered;
        }
    }

    fn trait_key_matches_allowed(
        &self,
        trait_key: &TraitKey,
        allowed_traits: &[CallPath<TraitSuffix>],
    ) -> bool {
        if allowed_traits.is_empty() {
            return false;
        }
        let call_path = trait_key.name.as_ref();
        let unify_check = UnifyCheck::constraint_subset(self.engines);

        allowed_traits
            .iter()
            .any(|allowed| trait_paths_equivalent(allowed, call_path, &unify_check))
    }

    /// Convert collected items to just the method decl refs we care about.
    #[inline]
    fn items_to_method_refs(&self, items: Vec<ty::TyTraitItem>) -> Vec<DeclRefFunction> {
        items
            .into_iter()
            .filter_map(|item| match item {
                ty::TyTraitItem::Fn(decl_ref) => Some(decl_ref),
                _ => None,
            })
            .collect()
    }

    #[inline]
    fn to_method_candidate(&self, decl_ref: &DeclRefFunction) -> MethodCandidate {
        let decl_engine = self.engines.de();
        let fn_decl = decl_engine.get_function(decl_ref);
        MethodCandidate {
            decl_ref: decl_ref.clone(),
            params: fn_decl
                .parameters
                .iter()
                .map(|p| p.type_argument.type_id())
                .collect(),
            ret: fn_decl.return_type.type_id(),
            is_contract_call: fn_decl.is_contract_call,
        }
    }

    /// Decide whether `cand` matches the given argument and annotation types.
    fn score_method_candidate(
        &self,
        cand: &MethodCandidate,
        argument_types: &[TypeId],
        annotation_type: TypeId,
    ) -> MatchScore {
        let eq_check = UnifyCheck::constraint_subset(self.engines).with_unify_ref_mut(false);
        let coercion_check = UnifyCheck::coercion(self.engines).with_ignore_generic_names(true);

        // Handle "self" for contract calls.
        let args_len_diff = if cand.is_contract_call && !argument_types.is_empty() {
            1
        } else {
            0
        };

        // Parameter count must match.
        if cand.params.len() != argument_types.len().saturating_sub(args_len_diff) {
            return MatchScore::Incompatible;
        }

        // Param-by-param check.
        let mut all_exact = true;
        for (p, a) in cand
            .params
            .iter()
            .zip(argument_types.iter().skip(args_len_diff))
        {
            if eq_check.check(*a, *p) {
                continue;
            }
            if coercion_check.check(*a, *p) {
                all_exact = false;
                continue;
            }
            return MatchScore::Incompatible;
        }

        let type_engine = self.engines.te();
        let ann = &*type_engine.get(annotation_type);
        let ret_ok = matches!(ann, TypeInfo::Unknown)
            || matches!(&*type_engine.get(cand.ret), TypeInfo::Never)
            || coercion_check.check(annotation_type, cand.ret);

        if !ret_ok {
            return MatchScore::Incompatible;
        }

        if all_exact {
            MatchScore::Exact
        } else {
            MatchScore::Coercible
        }
    }

    /// Keep only compatible candidates (coercible or exact).
    fn filter_method_candidates_by_signature(
        &self,
        decl_refs: &Vec<DeclRefFunction>,
        argument_types: &[TypeId],
        annotation_type: TypeId,
    ) -> Vec<MethodCandidate> {
        let mut out = Vec::new();
        for r in decl_refs {
            let cand = self.to_method_candidate(r);
            match self.score_method_candidate(&cand, argument_types, annotation_type) {
                MatchScore::Exact | MatchScore::Coercible => out.push(cand),
                MatchScore::Incompatible => {}
            }
        }
        out
    }

    /// Group signature-compatible method decl refs by their originating impl block,
    /// optionally filtering by a qualified trait path.
    fn group_by_trait_impl(
        &self,
        handler: &Handler,
        method_name: &Option<&MethodName>,
        method_decl_refs: &[DeclRefFunction],
    ) -> Result<GroupingResult, ErrorEmitted> {
        let decl_engine = self.engines.de();
        let type_engine = self.engines.te();
        let eq_check = UnifyCheck::constraint_subset(self.engines);

        // Extract `<... as Trait::<Args>>::method` info, if present.
        let (qualified_call_path, trait_method_name_binding_type_args): (
            Option<QualifiedCallPath>,
            Option<Vec<_>>,
        ) = match method_name {
            Some(MethodName::FromQualifiedPathRoot { as_trait, .. }) => {
                match &*type_engine.get(*as_trait) {
                    TypeInfo::Custom {
                        qualified_call_path: cp,
                        type_arguments,
                    } => (Some(cp.clone()), type_arguments.clone()),
                    _ => (None, None),
                }
            }
            _ => (None, None),
        };

        // Helper: compare two type arguments after resolution.
        let types_equal = |a: (&GenericArgument, &GenericArgument)| -> Result<bool, ErrorEmitted> {
            let (p1, p2) = a;
            let p1_id = self.resolve_type(
                handler,
                p1.type_id(),
                &p1.span(),
                EnforceTypeArguments::Yes,
                None,
            )?;
            let p2_id = self.resolve_type(
                handler,
                p2.type_id(),
                &p2.span(),
                EnforceTypeArguments::Yes,
                None,
            )?;
            Ok(eq_check.check(p1_id, p2_id))
        };

        // Helper: check whether this impl matches the optional qualified trait filter.
        let matches_trait_filter =
            |trait_decl: &ty::TyImplSelfOrTrait| -> Result<bool, ErrorEmitted> {
                // If there's no qualified trait filter, accept everything.
                let Some(qcp) = &qualified_call_path else {
                    return Ok(true);
                };

                // Trait name must match the one from the qualified path.
                if trait_decl.trait_name != qcp.clone().to_call_path(handler)? {
                    return Ok(false);
                }

                // If the qualified path provided type arguments, they must match the impl's.
                if let Some(params) = &trait_method_name_binding_type_args {
                    if params.len() != trait_decl.trait_type_arguments.len() {
                        return Ok(false);
                    }
                    for pair in params.iter().zip(trait_decl.trait_type_arguments.iter()) {
                        if !types_equal(pair)? {
                            return Ok(false);
                        }
                    }
                }
                Ok(true)
            };

        let mut trait_methods: BTreeMap<GroupingKey, DeclRefFunction> = BTreeMap::new();
        let mut impl_self_method: Option<DeclRefFunction> = None;

        for method_ref in method_decl_refs {
            let method = decl_engine.get_function(method_ref);

            // Only keep methods from an impl block (trait or inherent).
            let Some(ty::TyDecl::ImplSelfOrTrait(impl_trait)) = method.implementing_type.as_ref()
            else {
                continue;
            };

            let trait_decl = decl_engine.get_impl_self_or_trait(&impl_trait.decl_id);
            if !matches_trait_filter(&trait_decl)? {
                continue;
            }

            let key: GroupingKey = (impl_trait.decl_id, method.implementing_for_typeid);

            // Prefer the method that is type-check finalized when conflicting.
            match trait_methods.get_mut(&key) {
                Some(existing_ref) => {
                    let existing = decl_engine.get_function(existing_ref);
                    if !existing.is_type_check_finalized || method.is_type_check_finalized {
                        *existing_ref = method_ref.clone();
                    }
                }
                None => {
                    trait_methods.insert(key, method_ref.clone());
                }
            }

            // Track presence of an inherent impl so we can prefer it later.
            if trait_decl.trait_decl_ref.is_none() {
                impl_self_method = Some(method_ref.clone());
            }
        }

        Ok(GroupingResult {
            trait_methods,
            impl_self_method,
            qualified_call_path,
        })
    }

    fn prefer_non_blanket_impls(&self, trait_methods: &mut BTreeMap<GroupingKey, DeclRefFunction>) {
        let decl_engine = self.engines.de();

        let non_blanket_impl_exists = {
            trait_methods.values().any(|v| {
                let m = decl_engine.get_function(v);
                !m.is_from_blanket_impl(self.engines)
            })
        };

        if non_blanket_impl_exists {
            trait_methods.retain(|_, v| {
                let m = decl_engine.get_function(v);
                !m.is_from_blanket_impl(self.engines)
            });
        }
    }

    #[inline]
    fn trait_sig_string(&self, impl_id: &TraitImplId) -> String {
        let de = self.engines.de();
        let trait_decl = de.get_impl_self_or_trait(impl_id);
        if trait_decl.trait_type_arguments.is_empty() {
            trait_decl.trait_name.suffix.to_string()
        } else {
            let args = trait_decl
                .trait_type_arguments
                .iter()
                .map(|ga| self.engines.help_out(ga).to_string())
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", trait_decl.trait_name.suffix, args)
        }
    }

    fn select_method_from_grouped(
        &self,
        handler: &Handler,
        method_name: &Ident,
        type_id: TypeId,
        trait_methods: &BTreeMap<GroupingKey, DeclRefFunction>,
        impl_self_method: &Option<DeclRefFunction>,
    ) -> Result<Option<DeclRefFunction>, ErrorEmitted> {
        let decl_engine = self.engines.de();
        let eq_check = UnifyCheck::constraint_subset(self.engines);

        match trait_methods.len() {
            0 => Ok(None),
            1 => Ok(trait_methods.values().next().cloned()),
            _ => {
                if let Some(impl_self) = impl_self_method {
                    // Prefer inherent impl when mixed with trait methods.
                    return Ok(Some(impl_self.clone()));
                }

                // Exact implementing type wins.
                let mut exact = vec![];
                for r in trait_methods.values() {
                    let m = decl_engine.get_function(r);
                    if let Some(impl_for) = m.implementing_for_typeid {
                        if eq_check.with_unify_ref_mut(false).check(impl_for, type_id) {
                            exact.push(r.clone());
                        }
                    }
                }
                if exact.len() == 1 {
                    return Ok(Some(exact.remove(0)));
                }

                // Ambiguity: rebuild strings from impl ids.
                let mut trait_strings = trait_methods
                    .keys()
                    .map(|(impl_id, implementing_for)| {
                        let trait_str = self.trait_sig_string(impl_id);
                        let impl_for_str = implementing_for
                            .map(|t| self.engines.help_out(t).to_string())
                            .unwrap_or_else(|| self.engines().help_out(type_id).to_string());
                        (trait_str, impl_for_str)
                    })
                    .collect::<Vec<(String, String)>>();
                trait_strings.sort();

                Err(
                    handler.emit_err(CompileError::MultipleApplicableItemsInScope {
                        item_name: method_name.as_str().to_string(),
                        item_kind: "function".to_string(),
                        as_traits: trait_strings,
                        span: method_name.span(),
                    }),
                )
            }
        }
    }

    #[inline]
    fn format_candidate_summaries_for_error(&self, decl_refs: &[DeclRefFunction]) -> Vec<String> {
        let de = self.engines.de();

        let mut out: Vec<String> = decl_refs
            .iter()
            .map(|r| {
                let m = de.get_function(r);
                let params = m
                    .parameters
                    .iter()
                    .map(|p| self.engines.help_out(p.type_argument.type_id()).to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                let ret = self.engines.help_out(m.return_type.type_id());
                let in_impl = if let Some(for_ty) = m.implementing_for_typeid {
                    format!(" in {}", self.engines.help_out(for_ty))
                } else {
                    String::new()
                };
                format!("{}({}) -> {}{}", m.name.as_str(), params, ret, in_impl)
            })
            .collect();

        out.sort();
        out
    }

    /// Given a `method_name` and a `type_id`, find that method on that type in the namespace.
    /// `annotation_type` is the expected method return type. Requires `argument_types` because:
    /// - standard operations like +, <=, etc. are called like "std::ops::<operation>" and the
    ///   actual self type of the trait implementation is determined by the passed argument type.
    /// - we can have several implementations of generic traits for different types, that can
    ///   result in a method of a same name, but with different type arguments.
    ///
    /// This function will emit a [CompileError::MethodNotFound] if the method is not found.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn find_method_for_type(
        &self,
        handler: &Handler,
        type_id: TypeId,
        method_prefix: &ModulePath,
        method_ident: &Ident,
        annotation_type: TypeId,
        arguments_types: &[TypeId],
        method_name: Option<&MethodName>,
    ) -> Result<DeclRefFunction, ErrorEmitted> {
        let type_engine = self.engines.te();

        self.default_numeric_if_needed(handler, type_id, method_ident)?;

        let matching_items = self.collect_candidate_items(
            handler,
            type_id,
            method_prefix,
            method_ident,
            annotation_type,
            &method_name,
        )?;

        let matching_method_decl_refs = self.items_to_method_refs(matching_items);

        let candidates = self.filter_method_candidates_by_signature(
            &matching_method_decl_refs,
            arguments_types,
            annotation_type,
        );

        let mut matching_method_strings = HashSet::<String>::new();

        let mut qualified_call_path: Option<QualifiedCallPath> = None;

        if !candidates.is_empty() {
            let maybe_method_decl_refs: Vec<DeclRefFunction> =
                candidates.iter().map(|c| c.decl_ref.clone()).collect();

            let GroupingResult {
                mut trait_methods,
                impl_self_method,
                qualified_call_path: qcp,
            } = self.group_by_trait_impl(handler, &method_name, &maybe_method_decl_refs)?;
            qualified_call_path = qcp;

            if let Some((_, constraints)) =
                method_name.and_then(|name| self.trait_constraints_from_method_name(handler, name))
            {
                self.retain_trait_methods_matching_constraints(&mut trait_methods, &constraints);
            }

            // Prefer non-blanket impls when any concrete impl exists.
            self.prefer_non_blanket_impls(&mut trait_methods);

            // Final selection / ambiguity handling.
            if let Some(pick) = self.select_method_from_grouped(
                handler,
                method_ident,
                type_id,
                &trait_methods,
                &impl_self_method,
            )? {
                return Ok(pick.get_method_safe_to_unify(self.engines, type_id));
            }

            if qualified_call_path.is_none() {
                if let Some(first) = maybe_method_decl_refs.first() {
                    return Ok(first.get_method_safe_to_unify(self.engines, type_id));
                }
            }
        } else {
            // No signature-compatible candidates.
            matching_method_strings
                .extend(self.format_candidate_summaries_for_error(&matching_method_decl_refs));
        }

        // Forward an ErrorRecovery from the first argument if present.
        if let Some(TypeInfo::ErrorRecovery(err)) = arguments_types
            .first()
            .map(|x| (*type_engine.get(*x)).clone())
        {
            return Err(err);
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

        // Final: MethodNotFound with formatted signature and candidates.
        Err(handler.emit_err(CompileError::MethodNotFound {
            method: format!(
                "{}({}){}",
                method_ident.clone(),
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
            matching_method_strings: matching_method_strings.iter().cloned().collect::<Vec<_>>(),
            span: method_ident.span(),
        }))
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
