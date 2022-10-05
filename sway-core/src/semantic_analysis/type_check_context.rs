use crate::{
    language::parsed::declaration::Purity,
    namespace::Path,
    semantic_analysis::{ast_node::Mode, Namespace},
    type_system::{
        insert_type, monomorphize, unify_with_self, CopyTypes, EnforceTypeArguments,
        MonomorphizeHelper, TypeArgument, TypeId, TypeInfo,
    },
    CompileError, CompileResult, CompileWarning,
};
use sway_types::{span::Span, Ident};

/// Contextual state tracked and accumulated throughout type-checking.
pub struct TypeCheckContext<'ns> {
    /// The namespace context accumulated throughout type-checking.
    ///
    /// Internally, this includes:
    ///
    /// - The `root` module from which all other modules maybe be accessed using absolute paths.
    /// - The `init` module used to initialise submodule namespaces.
    /// - A `mod_path` that represents the current module being type-checked. This is automatically
    ///   updated upon entering/exiting submodules via the `enter_submodule` method.
    pub(crate) namespace: &'ns mut Namespace,

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
    mode: Mode,
    /// Provides "help text" to `TypeError`s during unification.
    // TODO: We probably shouldn't carry this through the `Context`, but instead pass it directly
    // to `unify` as necessary?
    help_text: &'static str,
    /// Tracks the purity of the context, e.g. whether or not we should be allowed to write to
    /// storage.
    purity: Purity,
}

impl<'ns> TypeCheckContext<'ns> {
    /// Initialise a context at the top-level of a module with its namespace.
    ///
    /// Initializes with:
    ///
    /// - type_annotation: unknown
    /// - mode: NoneAbi
    /// - help_text: ""
    /// - purity: Pure
    pub fn from_root(root_namespace: &'ns mut Namespace) -> Self {
        Self::from_module_namespace(root_namespace)
    }

    fn from_module_namespace(namespace: &'ns mut Namespace) -> Self {
        Self {
            namespace,
            type_annotation: insert_type(TypeInfo::Unknown),
            help_text: "",
            // TODO: Contract? Should this be passed in based on program kind (aka TreeType)?
            self_type: insert_type(TypeInfo::Contract),
            mode: Mode::NonAbi,
            purity: Purity::default(),
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
    pub fn by_ref(&mut self) -> TypeCheckContext {
        TypeCheckContext {
            namespace: self.namespace,
            type_annotation: self.type_annotation,
            self_type: self.self_type,
            mode: self.mode,
            help_text: self.help_text,
            purity: self.purity,
        }
    }

    /// Scope the `TypeCheckContext` with the given `Namespace`.
    pub fn scoped(self, namespace: &mut Namespace) -> TypeCheckContext {
        TypeCheckContext {
            namespace,
            type_annotation: self.type_annotation,
            self_type: self.self_type,
            mode: self.mode,
            help_text: self.help_text,
            purity: self.purity,
        }
    }

    /// Enter the submodule with the given name and produce a type-check context ready for
    /// type-checking its content.
    ///
    /// Returns the result of the given `with_submod_ctx` function.
    pub fn enter_submodule<F, T>(self, dep_name: Ident, with_submod_ctx: F) -> T
    where
        F: FnOnce(TypeCheckContext) -> T,
    {
        // We're checking a submodule, so no need to pass through anything other than the
        // namespace. However, we will likely want to pass through the type engine and declaration
        // engine here once they're added.
        let Self { namespace, .. } = self;
        let mut submod_ns = namespace.enter_submodule(dep_name);
        let submod_ctx = TypeCheckContext::from_module_namespace(&mut submod_ns);
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
    pub(crate) fn with_mode(self, mode: Mode) -> Self {
        Self { mode, ..self }
    }

    /// Map this `TypeCheckContext` instance to a new one with the given purity.
    pub(crate) fn with_purity(self, purity: Purity) -> Self {
        Self { purity, ..self }
    }

    /// Map this `TypeCheckContext` instance to a new one with the given purity.
    pub(crate) fn with_self_type(self, self_type: TypeId) -> Self {
        Self { self_type, ..self }
    }

    // A set of accessor methods. We do this rather than making the fields `pub` in order to ensure
    // that these are only updated via the `with_*` methods that produce a new `TypeCheckContext`.

    pub(crate) fn help_text(&self) -> &'static str {
        self.help_text
    }

    pub(crate) fn type_annotation(&self) -> TypeId {
        self.type_annotation
    }

    pub(crate) fn mode(&self) -> Mode {
        self.mode
    }

    pub(crate) fn purity(&self) -> Purity {
        self.purity
    }

    pub(crate) fn self_type(&self) -> TypeId {
        self.self_type
    }

    // Provide some convenience functions around the inner context.

    /// Short-hand for calling the `monomorphize` function in the type engine
    pub(crate) fn monomorphize<T>(
        &self,
        value: &mut T,
        type_arguments: &mut [TypeArgument],
        enforce_type_arguments: EnforceTypeArguments,
        call_site_span: &Span,
    ) -> CompileResult<()>
    where
        T: MonomorphizeHelper + CopyTypes,
    {
        monomorphize(
            value,
            type_arguments,
            enforce_type_arguments,
            call_site_span,
            &self.namespace.root,
            &self.namespace.mod_path,
        )
    }

    /// Short-hand for calling [Namespace::resolve_type_with_self] with the `self_type` provided by
    /// the `TypeCheckContext`.
    pub(crate) fn resolve_type_with_self(
        &mut self,
        type_id: TypeId,
        span: &Span,
        enforce_type_args: EnforceTypeArguments,
        type_info_prefix: Option<&Path>,
    ) -> CompileResult<TypeId> {
        self.namespace.resolve_type_with_self(
            type_id,
            self.self_type,
            span,
            enforce_type_args,
            type_info_prefix,
        )
    }

    /// Short-hand for calling [Namespace::resolve_type_without_self]
    pub(crate) fn resolve_type_without_self(
        &mut self,
        type_id: TypeId,
        span: &Span,
        type_info_prefix: Option<&Path>,
    ) -> CompileResult<TypeId> {
        self.namespace
            .resolve_type_without_self(type_id, span, type_info_prefix)
    }

    /// Short-hand around `type_system::unify_with_self`, where the `TypeCheckContext` provides the
    /// type annotation, self type and help text.
    pub(crate) fn unify_with_self(
        &self,
        ty: TypeId,
        span: &Span,
    ) -> (Vec<CompileWarning>, Vec<CompileError>) {
        unify_with_self(
            ty,
            self.type_annotation(),
            self.self_type(),
            span,
            self.help_text(),
        )
    }
}
