use crate::{
    engine_threading::*,
    language::{CallPath, Visibility},
    namespace::ResolvedDeclaration,
    semantic_analysis::{ast_node::ConstShadowingMode, Namespace},
    type_system::TypeId,
};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{span::Span, Ident};

use super::{
    symbol_collection_context::SymbolCollectionContext,
    type_resolve::{resolve_call_path, VisibilityCheck},
    GenericShadowingMode,
};

/// Contextual state tracked and accumulated throughout symbol resolving.
pub struct SymbolResolveContext<'a> {
    /// The namespace context accumulated throughout symbol resolving.
    ///
    /// Internally, this includes:
    ///
    /// - The `root` module from which all other modules maybe be accessed using absolute paths.
    /// - The `init` module used to initialize submodule namespaces.
    /// - A `mod_path` that represents the current module being type-checked. This is automatically
    ///   updated upon entering/exiting submodules via the `enter_submodule` method.
    pub(crate) engines: &'a Engines,
    pub(crate) symbol_collection_ctx: &'a mut SymbolCollectionContext,

    // The following set of fields are intentionally private. When a `SymbolResolveContext` is passed
    // into a new node during symbol resolving, these fields should be updated using the `with_*`
    // methods which provides a new `SymbolResolveContext`, ensuring we don't leak our changes into
    // the parent nodes.
    /// While symbol resolving an `impl` (whether inherent or for a `trait`/`abi`) this represents the
    /// type for which we are implementing. For example in `impl Foo {}` or `impl Trait for Foo
    /// {}`, this represents the type ID of `Foo`.
    self_type: Option<TypeId>,
    /// Whether or not a const declaration shadows previous const declarations sequentially.
    ///
    /// This is `Sequential` while checking const declarations in functions, otherwise `ItemStyle`.
    const_shadowing_mode: ConstShadowingMode,
    /// Whether or not a generic type parameters shadows previous generic type parameters.
    ///
    /// This is `Disallow` everywhere except while checking type parameters bounds in struct instantiation.
    generic_shadowing_mode: GenericShadowingMode,
}

impl<'a> SymbolResolveContext<'a> {
    /// Initialize a symbol resolving context with a namespace.
    pub fn new(
        engines: &'a Engines,
        symbol_collection_ctx: &'a mut SymbolCollectionContext,
    ) -> Self {
        Self {
            engines,
            symbol_collection_ctx,
            self_type: None,
            const_shadowing_mode: ConstShadowingMode::ItemStyle,
            generic_shadowing_mode: GenericShadowingMode::Disallow,
        }
    }

    /// Create a new context that mutably borrows the inner `namespace` with a lifetime bound by
    /// `self`.
    ///
    /// This is particularly useful when symbol resolving a node that has more than one child node
    /// (very often the case). By taking the context with the namespace lifetime bound to `self`
    /// rather than the original namespace reference, we instead restrict the returned context to
    /// the local scope and avoid consuming the original context when providing context to the
    /// first visited child node.
    pub fn by_ref(&mut self) -> SymbolResolveContext<'_> {
        SymbolResolveContext {
            engines: self.engines,
            symbol_collection_ctx: self.symbol_collection_ctx,
            self_type: self.self_type,
            const_shadowing_mode: self.const_shadowing_mode,
            generic_shadowing_mode: self.generic_shadowing_mode,
        }
    }

    /// Scope the `SymbolResolveContext` with a new namespace lexical scope.
    pub fn enter_lexical_scope<T>(
        self,
        handler: &Handler,
        span: Span,
        with_scoped_ctx: impl FnOnce(SymbolResolveContext) -> Result<T, ErrorEmitted>,
    ) -> Result<T, ErrorEmitted> {
        let engines = self.engines;
        self.symbol_collection_ctx.enter_lexical_scope(
            handler,
            engines,
            span,
            |sub_scope_collect_ctx| {
                let sub_scope_resolve_ctx =
                    SymbolResolveContext::new(engines, sub_scope_collect_ctx);
                with_scoped_ctx(sub_scope_resolve_ctx)
            },
        )
    }

    /// Enter the submodule with the given name and a symbol resolve context ready for
    /// symbol resolving its content.
    ///
    /// Returns the result of the given `with_submod_ctx` function.
    pub fn enter_submodule<T>(
        self,
	handler: &Handler,
        mod_name: Ident,
        visibility: Visibility,
        module_span: Span,
        with_submod_ctx: impl FnOnce(SymbolResolveContext) -> T,
    ) -> Result<T, ErrorEmitted> {
        let engines = self.engines;
        self.symbol_collection_ctx.enter_submodule(
	    handler,
            engines,
            mod_name,
            visibility,
            module_span,
            |submod_collect_ctx| {
                let submod_ctx = SymbolResolveContext::new(engines, submod_collect_ctx);
                with_submod_ctx(submod_ctx)
            },
        )
    }

    /// Returns a mutable reference to the current namespace.
    pub fn namespace_mut(&mut self) -> &mut Namespace {
        &mut self.symbol_collection_ctx.namespace
    }

    /// Returns a reference to the current namespace.
    pub fn namespace(&self) -> &Namespace {
        &self.symbol_collection_ctx.namespace
    }

    /// Map this `SymbolResolveContext` instance to a new one with the given const shadowing `mode`.
    #[allow(unused)]
    pub(crate) fn with_const_shadowing_mode(
        self,
        const_shadowing_mode: ConstShadowingMode,
    ) -> Self {
        Self {
            const_shadowing_mode,
            ..self
        }
    }

    /// Map this `SymbolResolveContext` instance to a new one with the given generic shadowing `mode`.
    #[allow(unused)]
    pub(crate) fn with_generic_shadowing_mode(
        self,
        generic_shadowing_mode: GenericShadowingMode,
    ) -> Self {
        Self {
            generic_shadowing_mode,
            ..self
        }
    }

    // A set of accessor methods. We do this rather than making the fields `pub` in order to ensure
    // that these are only updated via the `with_*` methods that produce a new `SymbolResolveContext`.
    #[allow(unused)]
    pub(crate) fn self_type(&self) -> Option<TypeId> {
        self.self_type
    }

    #[allow(unused)]
    pub(crate) fn const_shadowing_mode(&self) -> ConstShadowingMode {
        self.const_shadowing_mode
    }

    #[allow(unused)]
    pub(crate) fn generic_shadowing_mode(&self) -> GenericShadowingMode {
        self.generic_shadowing_mode
    }

    /// Get the engines needed for engine threading.
    pub(crate) fn engines(&self) -> &'a Engines {
        self.engines
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
            &self.namespace().current_mod_path,
            call_path,
            self.self_type(),
            VisibilityCheck::Yes,
        )
    }
}
