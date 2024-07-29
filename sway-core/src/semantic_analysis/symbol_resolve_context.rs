use crate::{
    engine_threading::*,
    language::{CallPath, QualifiedCallPath, Visibility},
    namespace::{ModulePath, ResolvedDeclaration},
    semantic_analysis::{ast_node::ConstShadowingMode, Namespace},
    type_system::{TypeArgument, TypeId, TypeInfo},
    TraitConstraint,
};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{span::Span, Ident, Spanned};
use sway_utils::iter_prefixes;

use super::{symbol_collection_context::SymbolCollectionContext, GenericShadowingMode};

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
    pub fn scoped<T>(
        self,
        with_scoped_ctx: impl FnOnce(SymbolResolveContext) -> Result<T, ErrorEmitted>,
    ) -> Result<T, ErrorEmitted> {
        let engines = self.engines;
        self.symbol_collection_ctx
            .enter_lexical_scope(engines, |sub_scope_collect_ctx| {
                let sub_scope_resolve_ctx =
                    SymbolResolveContext::new(engines, sub_scope_collect_ctx);
                with_scoped_ctx(sub_scope_resolve_ctx)
            })
    }

    /// Enter the submodule with the given name and a symbol resolve context ready for
    /// symbol resolving its content.
    ///
    /// Returns the result of the given `with_submod_ctx` function.
    pub fn enter_submodule<T>(
        self,
        mod_name: Ident,
        visibility: Visibility,
        module_span: Span,
        with_submod_ctx: impl FnOnce(SymbolResolveContext) -> T,
    ) -> T {
        let engines = self.engines;
        self.symbol_collection_ctx.enter_submodule(
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
        self.resolve_call_path_with_visibility_check_and_modpath(
            handler,
            &self.namespace().mod_path,
            call_path,
        )
    }

    /// Resolve a symbol that is potentially prefixed with some path, e.g. `foo::bar::symbol`.
    ///
    /// This will concatenate the `mod_path` with the `call_path`'s prefixes and
    /// then calling `resolve_symbol` with the resulting path and call_path's suffix.
    ///
    /// The `mod_path` is significant here as we assume the resolution is done within the
    /// context of the module pointed to by `mod_path` and will only check the call path prefixes
    /// and the symbol's own visibility.
    pub(crate) fn resolve_call_path_with_visibility_check_and_modpath(
        &self,
        handler: &Handler,
        mod_path: &ModulePath,
        call_path: &CallPath,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        let (decl, mod_path) = self.namespace().root.resolve_call_path_and_mod_path(
            handler,
            self.engines,
            mod_path,
            call_path,
            self.self_type,
        )?;

        // In case there is no mod path we don't need to check visibility
        if mod_path.is_empty() {
            return Ok(decl);
        }

        // In case there are no prefixes we don't need to check visibility
        if call_path.prefixes.is_empty() {
            return Ok(decl);
        }

        // check the visibility of the call path elements
        // we don't check the first prefix because direct children are always accessible
        for prefix in iter_prefixes(&call_path.prefixes).skip(1) {
            let module = self.namespace().lookup_submodule_from_absolute_path(
                handler,
                self.engines(),
                prefix,
            )?;
            if module.visibility().is_private() {
                let prefix_last = prefix[prefix.len() - 1].clone();
                handler.emit_err(CompileError::ImportPrivateModule {
                    span: prefix_last.span(),
                    name: prefix_last,
                });
            }
        }

        // check the visibility of the symbol itself
        if !decl.visibility(self.engines).is_public() {
            handler.emit_err(CompileError::ImportPrivateSymbol {
                name: call_path.suffix.clone(),
                span: call_path.suffix.span(),
            });
        }

        Ok(decl)
    }

    #[allow(unused)]
    pub(crate) fn resolve_qualified_call_path_with_visibility_check(
        &mut self,
        handler: &Handler,
        qualified_call_path: &QualifiedCallPath,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        self.resolve_qualified_call_path_with_visibility_check_and_modpath(
            handler,
            &self.namespace().mod_path.clone(),
            qualified_call_path,
        )
    }

    pub(crate) fn resolve_qualified_call_path_with_visibility_check_and_modpath(
        &mut self,
        handler: &Handler,
        mod_path: &ModulePath,
        qualified_call_path: &QualifiedCallPath,
    ) -> Result<ResolvedDeclaration, ErrorEmitted> {
        let engines = self.engines();
        let type_engine = self.engines().te();
        if let Some(qualified_path_root) = qualified_call_path.clone().qualified_path_root {
            let root_type_id = match &&*type_engine.get(qualified_path_root.ty.type_id) {
                TypeInfo::Custom {
                    qualified_call_path: call_path,
                    type_arguments,
                    ..
                } => {
                    let type_decl = self.resolve_call_path_with_visibility_check_and_modpath(
                        handler,
                        mod_path,
                        &call_path.clone().to_call_path(handler)?,
                    )?;
                    self.type_decl_opt_to_type_id(
                        handler,
                        Some(type_decl),
                        call_path.clone(),
                        &qualified_path_root.ty.span(),
                        EnforceTypeArguments::No,
                        mod_path,
                        type_arguments.clone(),
                    )?
                }
                _ => qualified_path_root.ty.type_id,
            };

            let as_trait_opt = match &&*type_engine.get(qualified_path_root.as_trait) {
                TypeInfo::Custom {
                    qualified_call_path: call_path,
                    ..
                } => Some(
                    call_path
                        .clone()
                        .to_call_path(handler)?
                        .to_fullpath(engines, self.namespace()),
                ),
                _ => None,
            };

            self.namespace().root.resolve_call_path_and_root_type_id(
                handler,
                engines,
                self.namespace().module(engines),
                root_type_id,
                as_trait_opt,
                &qualified_call_path.call_path,
                self.self_type(),
            )
        } else {
            self.resolve_call_path_with_visibility_check_and_modpath(
                handler,
                mod_path,
                &qualified_call_path.call_path,
            )
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(unused)]
    fn type_decl_opt_to_type_id(
        &mut self,
        handler: &Handler,
        type_decl_opt: Option<ResolvedDeclaration>,
        call_path: QualifiedCallPath,
        span: &Span,
        enforce_type_arguments: EnforceTypeArguments,
        mod_path: &ModulePath,
        type_arguments: Option<Vec<TypeArgument>>,
    ) -> Result<TypeId, ErrorEmitted> {
        todo!();
        // TODO/tritao
        // let decl_engine = self.engines.de();
        // let type_engine = self.engines.te();
        // Ok(match type_decl_opt {
        //     Some(ty::TyDecl::StructDecl(ty::StructDecl {
        //         decl_id: original_id,
        //         ..
        //     })) => {
        //         // get the copy from the declaration engine
        //         let mut new_copy = (*decl_engine.get_struct(&original_id)).clone();

        //         // monomorphize the copy, in place
        //         self.monomorphize_with_modpath(
        //             handler,
        //             &mut new_copy,
        //             &mut type_arguments.unwrap_or_default(),
        //             enforce_type_arguments,
        //             span,
        //             mod_path,
        //         )?;

        //         // insert the new copy in the decl engine
        //         let new_decl_ref = decl_engine.insert(new_copy);

        //         // create the type id from the copy
        //         type_engine.insert(
        //             self.engines,
        //             TypeInfo::Struct(new_decl_ref.clone()),
        //             new_decl_ref.span().source_id(),
        //         )
        //     }
        //     Some(ty::TyDecl::EnumDecl(ty::EnumDecl {
        //         decl_id: original_id,
        //         ..
        //     })) => {
        //         // get the copy from the declaration engine
        //         let mut new_copy = (*decl_engine.get_enum(&original_id)).clone();

        //         // monomorphize the copy, in place
        //         self.monomorphize_with_modpath(
        //             handler,
        //             &mut new_copy,
        //             &mut type_arguments.unwrap_or_default(),
        //             enforce_type_arguments,
        //             span,
        //             mod_path,
        //         )?;

        //         // insert the new copy in the decl engine
        //         let new_decl_ref = decl_engine.insert(new_copy);

        //         // create the type id from the copy
        //         type_engine.insert(
        //             self.engines,
        //             TypeInfo::Enum(new_decl_ref.clone()),
        //             new_decl_ref.span().source_id(),
        //         )
        //     }
        //     Some(ty::TyDecl::TypeAliasDecl(ty::TypeAliasDecl {
        //         decl_id: original_id,
        //         ..
        //     })) => {
        //         let new_copy = decl_engine.get_type_alias(&original_id);

        //         // TODO: monomorphize the copy, in place, when generic type aliases are
        //         // supported

        //         new_copy.create_type_id(self.engines)
        //     }
        //     Some(ty::TyDecl::GenericTypeForFunctionScope(ty::GenericTypeForFunctionScope {
        //         type_id,
        //         ..
        //     })) => type_id,
        //     Some(ty::TyDecl::TraitTypeDecl(ty::TraitTypeDecl {
        //         decl_id,
        //         name,
        //         decl_span: _,
        //     })) => {
        //         let decl_type = decl_engine.get_type(&decl_id);

        //         if let Some(ty) = &decl_type.ty {
        //             ty.type_id
        //         } else if let Some(implementing_type) = self.self_type() {
        //             type_engine.insert(
        //                 self.engines,
        //                 TypeInfo::TraitType {
        //                     name: name.clone(),
        //                     trait_type_id: implementing_type,
        //                 },
        //                 name.span().source_id(),
        //             )
        //         } else {
        //             return Err(handler.emit_err(CompileError::Internal(
        //                 "Self type not provided.",
        //                 span.clone(),
        //             )));
        //         }
        //     }
        //     _ => {
        //         let err = handler.emit_err(CompileError::UnknownTypeName {
        //             name: call_path.call_path.to_string(),
        //             span: call_path.call_path.span(),
        //         });
        //         type_engine.insert(self.engines, TypeInfo::ErrorRecovery(err), None)
        //     }
        // })
    }

    pub fn check_type_impls_traits(
        &mut self,
        type_id: TypeId,
        constraints: &[TraitConstraint],
    ) -> bool {
        let handler = Handler::default();
        let engines = self.engines;

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
            )
            .is_ok()
    }
}

/// This type is used to denote if, during monomorphization, the compiler
/// should enforce that type arguments be provided. An example of that
/// might be this:
///
/// ```ignore
/// struct Point<T> {
///   x: u64,
///   y: u64
/// }
///
/// fn add<T>(p1: Point<T>, p2: Point<T>) -> Point<T> {
///   Point {
///     x: p1.x + p2.x,
///     y: p1.y + p2.y
///   }
/// }
/// ```
///
/// `EnforceTypeArguments` would require that the type annotations
/// for `p1` and `p2` contain `<...>`. This is to avoid ambiguous definitions:
///
/// ```ignore
/// fn add(p1: Point, p2: Point) -> Point {
///   Point {
///     x: p1.x + p2.x,
///     y: p1.y + p2.y
///   }
/// }
/// ```
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub(crate) enum EnforceTypeArguments {
    Yes,
    No,
}
