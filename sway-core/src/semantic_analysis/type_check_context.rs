use crate::{
    engine_threading::*,
    language::{parsed::TreeType, ty::TyDecl, Purity, Visibility},
    namespace::Path,
    semantic_analysis::{
        ast_node::{AbiMode, ConstShadowingMode},
        Namespace,
    },
    type_system::{
        EnforceTypeArguments, MonomorphizeHelper, SubstTypes, TypeArgument, TypeId, TypeInfo,
    },
};
use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{span::Span, Ident};

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

    /// Short-hand for calling [Namespace::resolve_type]
    pub(crate) fn resolve_type(
        &mut self,
        handler: &Handler,
        type_id: TypeId,
        span: &Span,
        enforce_type_args: EnforceTypeArguments,
        type_info_prefix: Option<&Path>,
    ) -> Result<TypeId, ErrorEmitted> {
        self.namespace.resolve_type(
            handler,
            self.engines(),
            type_id,
            span,
            enforce_type_args,
            type_info_prefix,
        )
    }

    /// Short-hand around `type_system::unify_`, where the `TypeCheckContext`
    /// provides the type annotation and help text.
    pub(crate) fn unify_with_type_annotation(&self, handler: &Handler, ty: TypeId, span: &Span) {
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
}
