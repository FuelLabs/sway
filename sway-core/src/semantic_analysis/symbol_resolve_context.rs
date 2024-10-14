use std::collections::{HashMap, VecDeque};

use crate::{
    decl_engine::parsed_id::ParsedDeclId,
    engine_threading::*,
    language::{
        parsed::{self, Declaration, FunctionDeclaration},
        CallPath, QualifiedCallPath, Visibility,
    },
    namespace::{
        ModulePath, ResolvedDeclaration, ResolvedTraitImplItem, TryInsertingTraitImplOnFailure,
    },
    semantic_analysis::{ast_node::ConstShadowingMode, Namespace},
    type_system::{TypeArgument, TypeId, TypeInfo},
    EnforceTypeArguments, TraitConstraint, UnifyCheck,
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

    /// Resolve the type of the given [TypeId], replacing any instances of
    /// [TypeInfo::Custom] with a reference to the declaration.
    #[allow(clippy::too_many_arguments)]
    #[allow(unused)]
    pub(crate) fn resolve(
        &mut self,
        handler: &Handler,
        type_id: TypeId,
        span: &Span,
        enforce_type_arguments: EnforceTypeArguments,
        type_info_prefix: Option<&ModulePath>,
        mod_path: &ModulePath,
    ) -> Result<TypeId, ErrorEmitted> {
        let engines = self.engines;
        let type_engine = engines.te();
        let module_path = type_info_prefix.unwrap_or(mod_path);
        let type_id = match (*type_engine.get(type_id)).clone() {
            TypeInfo::Custom {
                qualified_call_path,
                type_arguments,
                root_type_id,
            } => {
                let type_decl_opt = if let Some(root_type_id) = root_type_id {
                    self.namespace()
                        .root()
                        .resolve_call_path_and_root_type_id(
                            handler,
                            self.engines,
                            self.namespace().module(engines),
                            root_type_id,
                            None,
                            &qualified_call_path.clone().to_call_path(handler)?,
                            self.self_type(),
                        )
                        .ok()
                } else {
                    self.resolve_qualified_call_path_with_visibility_check_and_modpath(
                        handler,
                        module_path,
                        &qualified_call_path,
                    )
                    .ok()
                };
                self.type_decl_opt_to_type_id(
                    handler,
                    type_decl_opt,
                    qualified_call_path.clone(),
                    span,
                    enforce_type_arguments,
                    mod_path,
                    type_arguments.clone(),
                )?
            }
            TypeInfo::Array(mut elem_ty, n) => {
                elem_ty.type_id = self
                    .resolve(
                        handler,
                        elem_ty.type_id,
                        span,
                        enforce_type_arguments,
                        None,
                        mod_path,
                    )
                    .unwrap_or_else(|err| {
                        self.engines
                            .te()
                            .insert(self.engines, TypeInfo::ErrorRecovery(err), None)
                    });

                self.engines.te().insert(
                    self.engines,
                    TypeInfo::Array(elem_ty.clone(), n.clone()),
                    elem_ty.span.source_id(),
                )
            }
            TypeInfo::Tuple(mut type_arguments) => {
                for type_argument in type_arguments.iter_mut() {
                    type_argument.type_id = self
                        .resolve(
                            handler,
                            type_argument.type_id,
                            span,
                            enforce_type_arguments,
                            None,
                            mod_path,
                        )
                        .unwrap_or_else(|err| {
                            self.engines.te().insert(
                                self.engines,
                                TypeInfo::ErrorRecovery(err),
                                None,
                            )
                        });
                }

                self.engines.te().insert(
                    self.engines,
                    TypeInfo::Tuple(type_arguments),
                    span.source_id(),
                )
            }
            TypeInfo::TraitType {
                name,
                trait_type_id,
            } => {
                let item_ref = self.namespace().get_root_trait_item_for_type(
                    handler,
                    self.engines,
                    &name,
                    trait_type_id,
                    None,
                )?;
                if let ResolvedTraitImplItem::Parsed(parsed::ImplItem::Type(type_ref)) = item_ref {
                    let type_decl = self.engines.pe().get_trait_type(&type_ref);
                    // if let Some(ty) = &type_decl.ty {
                    // ty.type_id
                    // } else {
                    type_id
                    // }
                } else {
                    return Err(handler.emit_err(CompileError::Internal(
                        "Expecting associated type",
                        item_ref.span(self.engines),
                    )));
                }
            }
            TypeInfo::Ref {
                referenced_type: mut ty,
                to_mutable_value,
            } => {
                ty.type_id = self
                    .resolve(
                        handler,
                        ty.type_id,
                        span,
                        enforce_type_arguments,
                        None,
                        mod_path,
                    )
                    .unwrap_or_else(|err| {
                        self.engines
                            .te()
                            .insert(self.engines, TypeInfo::ErrorRecovery(err), None)
                    });

                self.engines.te().insert(
                    self.engines,
                    TypeInfo::Ref {
                        to_mutable_value,
                        referenced_type: ty.clone(),
                    },
                    None,
                )
            }
            _ => type_id,
        };

        // TODO/tritao
        //let mut type_id = type_id;
        //type_id.subst(&self.type_subst(), self.engines());

        Ok(type_id)
    }

    /// Short-hand for calling [Root::resolve_type_with_self] on `root` with the `mod_path`.
    #[allow(clippy::too_many_arguments)] // TODO: remove lint bypass once private modules are no longer experimental
    #[allow(unused)]
    pub(crate) fn resolve_type(
        &mut self,
        handler: &Handler,
        type_id: TypeId,
        span: &Span,
        enforce_type_arguments: EnforceTypeArguments,
        type_info_prefix: Option<&ModulePath>,
    ) -> Result<TypeId, ErrorEmitted> {
        let mod_path = self.namespace().mod_path.clone();
        self.resolve(
            handler,
            type_id,
            span,
            enforce_type_arguments,
            type_info_prefix,
            &mod_path,
        )
    }

    /// Short-hand for calling [Root::resolve_type_without_self] on `root` and with the `mod_path`.
    pub(crate) fn resolve_type_without_self(
        &mut self,
        handler: &Handler,
        type_id: TypeId,
        span: &Span,
        type_info_prefix: Option<&ModulePath>,
    ) -> Result<TypeId, ErrorEmitted> {
        let mod_path = self.namespace().mod_path.clone();
        self.resolve(
            handler,
            type_id,
            span,
            EnforceTypeArguments::Yes,
            type_info_prefix,
            &mod_path,
        )
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

    /// Given a name and a type (plus a `self_type` to potentially
    /// resolve it), find items matching in the namespace.
    #[allow(unused)]
    pub(crate) fn find_items_for_type(
        &mut self,
        handler: &Handler,
        type_id: TypeId,
        item_prefix: &ModulePath,
        item_name: &Ident,
    ) -> Result<Vec<parsed::ImplItem>, ErrorEmitted> {
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
        let type_id = self
            .resolve(
                handler,
                type_id,
                &item_name.span(),
                EnforceTypeArguments::No,
                None,
                item_prefix,
            )
            .unwrap_or_else(|err| {
                type_engine.insert(self.engines, TypeInfo::ErrorRecovery(err), None)
            });

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

        let mut matching_item_decl_refs: Vec<parsed::ImplItem> = vec![];

        let pe = self.engines.pe();
        for item in items.into_iter() {
            match &item {
                ResolvedTraitImplItem::Parsed(item) => match item {
                    parsed::ImplItem::Fn(decl_id) => {
                        if pe.get_function(decl_id).name == *item_name {
                            matching_item_decl_refs.push(item.clone());
                        }
                    }
                    parsed::ImplItem::Constant(decl_id) => {
                        if pe.get_constant(decl_id).name == *item_name {
                            matching_item_decl_refs.push(item.clone());
                        }
                    }
                    parsed::ImplItem::Type(decl_id) => {
                        if pe.get_trait_type(decl_id).name == *item_name {
                            matching_item_decl_refs.push(item.clone());
                        }
                    }
                },
                ResolvedTraitImplItem::Typed(_) => unreachable!(),
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
    #[allow(unused)]
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
    ) -> Result<ParsedDeclId<FunctionDeclaration>, ErrorEmitted> {
        let decl_engine = self.engines.de();
        let type_engine = self.engines.te();
        let parsed_decl_engine = self.engines.pe();

        let eq_check = UnifyCheck::non_dynamic_equality(self.engines);
        let coercion_check = UnifyCheck::coercion(self.engines);

        // default numeric types to u64
        if type_engine.contains_numeric(decl_engine, type_id) {
            type_engine.decay_numeric(handler, self.engines, type_id, &method_name.span())?;
        }

        let matching_item_decl_refs =
            self.find_items_for_type(handler, type_id, method_prefix, method_name)?;

        let matching_method_decl_refs = matching_item_decl_refs
            .into_iter()
            .flat_map(|item| match item {
                parsed::ImplItem::Fn(decl_id) => Some(decl_id),
                parsed::ImplItem::Constant(_) => None,
                parsed::ImplItem::Type(_) => None,
            })
            .collect::<Vec<_>>();

        let mut qualified_call_path = None;
        let matching_method_decl_ref = {
            // Case where multiple methods exist with the same name
            // This is the case of https://github.com/FuelLabs/sway/issues/3633
            // where multiple generic trait impls use the same method name but with different parameter types
            let mut maybe_method_decl_refs: Vec<ParsedDeclId<FunctionDeclaration>> = vec![];
            for decl_ref in matching_method_decl_refs.clone().into_iter() {
                let method = parsed_decl_engine.get_function(&decl_ref);
                if method.parameters.len() == arguments_types.len()
                    && method
                        .parameters
                        .iter()
                        .zip(arguments_types.iter())
                        .all(|(p, a)| coercion_check.check(p.type_argument.type_id, *a))
                    && (matches!(&*type_engine.get(annotation_type), TypeInfo::Unknown)
                        || coercion_check.check(annotation_type, method.return_type.type_id))
                {
                    maybe_method_decl_refs.push(decl_ref);
                }
            }

            if !maybe_method_decl_refs.is_empty() {
                let mut trait_methods = HashMap::<
                    (CallPath, Vec<WithEngines<TypeArgument>>),
                    ParsedDeclId<FunctionDeclaration>,
                >::new();
                let mut impl_self_method = None;
                for method_ref in maybe_method_decl_refs.clone() {
                    let method = parsed_decl_engine.get_function(&method_ref);
                    if let Some(Declaration::ImplSelfOrTrait(impl_trait)) =
                        method.implementing_type.clone()
                    {
                        let trait_decl = parsed_decl_engine.get_impl_self_or_trait(&impl_trait);
                        let mut skip_insert = false;
                        if let Some(as_trait) = as_trait {
                            if let TypeInfo::Custom {
                                qualified_call_path: call_path,
                                type_arguments,
                                root_type_id: _,
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
                                                trait_decl.trait_name.clone(),
                                                trait_decl
                                                    .trait_type_arguments
                                                    .iter()
                                                    .cloned()
                                                    .map(|a| self.engines.help_out(a))
                                                    .collect::<Vec<_>>(),
                                            ),
                                            method_ref,
                                        );
                                    }
                                }
                                skip_insert = true;
                            }
                        }

                        if !skip_insert {
                            trait_methods.insert(
                                (
                                    trait_decl.trait_name.clone(),
                                    trait_decl
                                        .trait_type_arguments
                                        .iter()
                                        .cloned()
                                        .map(|a| self.engines.help_out(a))
                                        .collect::<Vec<_>>(),
                                ),
                                method_ref,
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
                                item_name: method_name.as_str().to_string(),
                                item_kind: "function".to_string(),
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
                    maybe_method_decl_refs.first().cloned()
                }
            } else {
                // When we can't match any method with parameter types we still return the first method found
                // This was the behavior before introducing the parameter type matching
                matching_method_decl_refs.first().cloned()
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
                method_name: method_name.clone(),
                type_name,
                span: method_name.span(),
            }))
        }
    }

    #[allow(unused)]
    pub(crate) fn get_items_for_type_and_trait_name(
        &self,
        type_id: TypeId,
        trait_name: &CallPath,
    ) -> Vec<ResolvedTraitImplItem> {
        self.get_items_for_type_and_trait_name_and_trait_type_arguments(type_id, trait_name, &[])
    }

    #[allow(unused)]
    pub(crate) fn get_items_for_type_and_trait_name_and_trait_type_arguments(
        &self,
        type_id: TypeId,
        trait_name: &CallPath,
        trait_type_args: &[TypeArgument],
    ) -> Vec<ResolvedTraitImplItem> {
        // Use trait name with full path, improves consistency between
        // this get and inserting in `insert_trait_implementation`.
        let trait_name = trait_name.to_fullpath(self.engines(), self.namespace());

        self.namespace()
            .module(self.engines)
            .current_items()
            .implemented_traits
            .get_items_for_type_and_trait_name_and_trait_type_arguments(
                self.engines,
                type_id,
                &trait_name,
                trait_type_args,
            )
    }

    pub(crate) fn insert_trait_implementation_for_type(&mut self, type_id: TypeId) {
        let engines = self.engines;
        self.namespace_mut()
            .module_mut(engines)
            .current_items_mut()
            .implemented_traits
            .insert_for_type(engines, type_id, crate::namespace::CodeBlockFirstPass::Yes);
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
                crate::namespace::CodeBlockFirstPass::Yes,
            )
            .is_ok()
    }
}
