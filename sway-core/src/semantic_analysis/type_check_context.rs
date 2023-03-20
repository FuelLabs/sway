use crate::{
    decl_engine::{DeclEngine, DeclId, DeclRef},
    engine_threading::*,
    language::{parsed::TreeType, Purity},
    namespace::Path,
    semantic_analysis::{ast_node::Mode, Namespace},
    type_system::*,
    CompileResult, CompileWarning,
};
use sway_error::error::CompileError;
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

    /// The type engine storing types.
    pub(crate) type_engine: &'a TypeEngine,

    /// The declaration engine holds declarations.
    pub(crate) decl_engine: &'a DeclEngine,

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
    pub fn from_root(root_namespace: &'a mut Namespace, engines: Engines<'a>) -> Self {
        Self::from_module_namespace(root_namespace, engines)
    }

    fn from_module_namespace(namespace: &'a mut Namespace, engines: Engines<'a>) -> Self {
        let (type_engine, decl_engine) = engines.unwrap();
        Self {
            namespace,
            type_engine,
            decl_engine,
            type_annotation: type_engine.insert(decl_engine, TypeInfo::Unknown),
            help_text: "",
            // TODO: Contract? Should this be passed in based on program kind (aka TreeType)?
            self_type: type_engine.insert(decl_engine, TypeInfo::Contract),
            mode: Mode::NonAbi,
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
            mode: self.mode,
            help_text: self.help_text,
            purity: self.purity,
            kind: self.kind.clone(),
            type_engine: self.type_engine,
            decl_engine: self.decl_engine,
            disallow_functions: self.disallow_functions,
        }
    }

    /// Scope the `TypeCheckContext` with the given `Namespace`.
    pub fn scoped(self, namespace: &'a mut Namespace) -> TypeCheckContext<'a> {
        TypeCheckContext {
            namespace,
            type_annotation: self.type_annotation,
            self_type: self.self_type,
            mode: self.mode,
            help_text: self.help_text,
            purity: self.purity,
            kind: self.kind,
            type_engine: self.type_engine,
            decl_engine: self.decl_engine,
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
        module_span: Span,
        with_submod_ctx: impl FnOnce(TypeCheckContext) -> T,
    ) -> T {
        // We're checking a submodule, so no need to pass through anything other than the
        // namespace. However, we will likely want to pass through the type engine and declaration
        // engine here once they're added.
        let Self { namespace, .. } = self;
        let mut submod_ns = namespace.enter_submodule(mod_name, module_span);
        let submod_ctx = TypeCheckContext::from_module_namespace(
            &mut submod_ns,
            Engines::new(self.type_engine, self.decl_engine),
        );
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

    pub(crate) fn mode(&self) -> Mode {
        self.mode
    }

    pub(crate) fn purity(&self) -> Purity {
        self.purity
    }

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

    pub(crate) fn combine_subst_list_and_args<T>(
        &mut self,
        decl_ref: &mut DeclRef<DeclId<T>>,
        type_args: &mut [TypeArgument],
        enforce_type_args: EnforceTypeArguments,
        call_site_span: &Span,
    ) -> CompileResult<()> {
        let mod_path = self.namespace.mod_path.clone();
        self.type_engine.combine_subst_list_and_args(
            self.namespace,
            self.decl_engine,
            &mod_path,
            decl_ref,
            type_args,
            enforce_type_args,
            call_site_span,
        )
    }

    /// Short-hand for calling [Namespace::resolve_type].
    pub(crate) fn resolve_type(
        &mut self,
        type_id: TypeId,
        span: &Span,
        enforce_type_args: EnforceTypeArguments,
        type_info_prefix: Option<&Path>,
    ) -> CompileResult<TypeId> {
        self.namespace.resolve_type(
            self.engines(),
            type_id,
            span,
            enforce_type_args,
            type_info_prefix,
        )
    }

    /// Short-hand around `type_system::unify`, where the
    /// `TypeCheckContext` provides the type annotation and help text.
    pub(crate) fn unify_with_type_annotation(
        &self,
        ty: TypeId,
        span: &Span,
    ) -> (Vec<CompileWarning>, Vec<CompileError>) {
        self.type_engine.unify(
            self.decl_engine,
            ty,
            self.type_annotation(),
            &self.namespace.type_subst_stack_top(),
            span,
            self.help_text(),
            None,
        )
    }

    /// Get the engines needed for engine threading.
    pub(crate) fn engines(&self) -> Engines<'a> {
        Engines::new(self.type_engine, self.decl_engine)
    }
}
