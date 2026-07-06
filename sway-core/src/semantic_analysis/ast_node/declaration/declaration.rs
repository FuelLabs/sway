use ast_elements::type_argument::GenericTypeArgument;
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::{Ident, Named, Spanned};

use crate::{
    decl_engine::{DeclEngineGet, DeclEngineInsert, DeclRef, ReplaceFunctionImplementingType},
    language::{
        parsed::{self, Declaration, StorageEntry},
        ty::{
            self, FunctionDecl, TyAbiDecl, TyConfigurableDecl, TyConstantDecl, TyDecl, TyEnumDecl,
            TyFunctionDecl, TyImplSelfOrTrait, TyStorageDecl, TyStorageField, TyStructDecl,
            TyTraitDecl, TyTraitFn, TyTraitType, TyTypeAliasDecl, TyVariableDecl,
        },
        CallPath,
    },
    namespace::{IsExtendingExistingImpl, IsImplInterfaceSurface, IsImplSelf, Items},
    semantic_analysis::{
        symbol_collection_context::SymbolCollectionContext, ConstShadowingMode,
        GenericShadowingMode, TypeCheckAnalysis, TypeCheckAnalysisContext, TypeCheckContext,
        TypeCheckFinalization, TypeCheckFinalizationContext,
    },
    type_system::*,
    Engines,
};

impl TyDecl {
    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        decl: &Declaration,
    ) -> Result<(), ErrorEmitted> {
        match decl {
            Declaration::VariableDeclaration(decl_id) => {
                TyVariableDecl::collect(handler, engines, ctx, decl_id)?
            }
            Declaration::ConstantDeclaration(decl_id) => {
                TyConstantDecl::collect(handler, engines, ctx, decl_id)?
            }
            Declaration::ConfigurableDeclaration(decl_id) => {
                TyConfigurableDecl::collect(handler, engines, ctx, decl_id)?
            }
            Declaration::TraitTypeDeclaration(decl_id) => {
                TyTraitType::collect(handler, engines, ctx, decl_id)?
            }
            Declaration::TraitFnDeclaration(decl_id) => {
                TyTraitFn::collect(handler, engines, ctx, decl_id)?
            }
            Declaration::EnumDeclaration(decl_id) => {
                TyEnumDecl::collect(handler, engines, ctx, decl_id)?
            }
            Declaration::EnumVariantDeclaration(_decl) => {}
            Declaration::FunctionDeclaration(decl_id) => {
                TyFunctionDecl::collect(handler, engines, ctx, decl_id)?
            }
            Declaration::TraitDeclaration(decl_id) => {
                TyTraitDecl::collect(handler, engines, ctx, decl_id)?
            }
            Declaration::ImplSelfOrTrait(decl_id) => {
                TyImplSelfOrTrait::collect(handler, engines, ctx, decl_id)?
            }
            Declaration::StructDeclaration(decl_id) => {
                TyStructDecl::collect(handler, engines, ctx, decl_id)?
            }
            Declaration::AbiDeclaration(decl_id) => {
                TyAbiDecl::collect(handler, engines, ctx, decl_id)?
            }
            Declaration::StorageDeclaration(decl_id) => {
                TyStorageDecl::collect(handler, engines, ctx, decl_id)?
            }
            Declaration::TypeAliasDeclaration(decl_id) => {
                TyTypeAliasDecl::collect(handler, engines, ctx, decl_id)?
            }
            Declaration::ConstGenericDeclaration(_) => {
                return Err(handler.emit_err(CompileError::Internal(
                    "Unexpected error on const generics",
                    decl.span(engines),
                )));
            }
        };

        Ok(())
    }

    pub(crate) fn type_check(
        handler: &Handler,
        ctx: &mut TypeCheckContext,
        decl: &Declaration,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        let decl = match decl {
            Declaration::VariableDeclaration(decl_id) => {
                let decl = &*engines.pe().get_variable(decl_id);
                let var_decl = match ty::TyVariableDecl::type_check(handler, ctx.by_ref(), decl) {
                    Ok(res) => res,
                    Err(err) => return Ok(ty::TyDecl::ErrorRecovery(decl.name.span(), err)),
                };
                let typed_var_decl = ty::TyDecl::VariableDecl(Box::new(var_decl));
                ctx.insert_symbol(handler, decl.name.clone(), typed_var_decl.clone())?;
                typed_var_decl
            }
            Declaration::ConstantDeclaration(decl_id) => {
                let decl = &*engines.pe().get_constant(decl_id);
                let const_decl = match ty::TyConstantDecl::type_check(handler, ctx.by_ref(), decl) {
                    Ok(res) => res,
                    Err(err) => return Ok(ty::TyDecl::ErrorRecovery(decl.span.clone(), err)),
                };
                let name = const_decl.name().clone();
                let typed_const_decl: ty::TyDecl =
                    decl_engine.insert(const_decl, Some(decl_id)).into();
                ctx.insert_symbol(handler, name, typed_const_decl.clone())?;
                typed_const_decl
            }
            Declaration::ConfigurableDeclaration(decl_id) => {
                let decl = &*engines.pe().get_configurable(decl_id);
                let typed_const_decl =
                    match ty::TyConfigurableDecl::type_check(handler, ctx.by_ref(), decl) {
                        Ok(config_decl) => {
                            config_decl.forbid_const_generics(handler, engines)?;
                            ty::TyDecl::from(decl_engine.insert(config_decl, Some(decl_id)))
                        }
                        Err(err) => ty::TyDecl::ErrorRecovery(decl.span.clone(), err),
                    };
                ctx.insert_symbol(handler, decl.name.clone(), typed_const_decl.clone())?;
                typed_const_decl
            }
            Declaration::TraitTypeDeclaration(decl_id) => {
                let decl = &*engines.pe().get_trait_type(decl_id);
                let type_decl = match ty::TyTraitType::type_check(handler, ctx.by_ref(), decl) {
                    Ok(res) => res,
                    Err(err) => return Ok(ty::TyDecl::ErrorRecovery(decl.span.clone(), err)),
                };
                let name = type_decl.name().clone();
                let typed_type_decl: ty::TyDecl =
                    decl_engine.insert(type_decl, Some(decl_id)).into();
                ctx.insert_symbol(handler, name, typed_type_decl.clone())?;
                typed_type_decl
            }
            Declaration::EnumDeclaration(decl_id) => {
                let decl = &*engines.pe().get_enum(decl_id);
                let enum_decl = match ty::TyEnumDecl::type_check(handler, ctx.by_ref(), decl) {
                    Ok(res) => res,
                    Err(err) => return Ok(ty::TyDecl::ErrorRecovery(decl.span.clone(), err)),
                };
                let name = enum_decl.call_path.suffix.clone();
                let typed_enum_decl: ty::TyDecl =
                    decl_engine.insert(enum_decl, Some(decl_id)).into();
                ctx.insert_symbol(handler, name, typed_enum_decl.clone())?;
                typed_enum_decl
            }
            Declaration::EnumVariantDeclaration(_decl) => {
                unreachable!("Type-checked above as part of the containing enum.")
            }
            Declaration::FunctionDeclaration(decl_id) => {
                let fn_decl = &*engines.pe().get_function(decl_id);
                let mut ctx = ctx.by_ref().with_type_annotation(type_engine.new_unknown());
                let fn_decl = match ty::TyFunctionDecl::type_check(
                    handler,
                    ctx.by_ref(),
                    fn_decl,
                    false,
                    false,
                    None,
                ) {
                    Ok(res) => res,
                    Err(err) => return Ok(ty::TyDecl::ErrorRecovery(fn_decl.span.clone(), err)),
                };

                let name = fn_decl.name.clone();
                let decl: ty::TyDecl = decl_engine.insert(fn_decl, Some(decl_id)).into();
                let _ = ctx.insert_symbol(handler, name, decl.clone());
                decl
            }
            Declaration::TraitDeclaration(decl_id) => {
                let trait_decl = &*engines.pe().get_trait(decl_id);
                let mut trait_decl =
                    match ty::TyTraitDecl::type_check(handler, ctx.by_ref(), trait_decl) {
                        Ok(res) => res,
                        Err(err) => {
                            return Ok(ty::TyDecl::ErrorRecovery(trait_decl.span.clone(), err))
                        }
                    };
                let name = trait_decl.name.clone();

                // save decl_refs for the LSP
                for supertrait in trait_decl.supertraits.iter_mut() {
                    let _ =
                        ctx.resolve_call_path(handler, &supertrait.name)
                            .map(|supertrait_decl| {
                                if let ty::TyDecl::TraitDecl(ty::TraitDecl {
                                    decl_id: supertrait_decl_id,
                                }) = supertrait_decl
                                {
                                    let supertrait_decl = engines.de().get(&supertrait_decl_id);
                                    supertrait.decl_ref = Some(DeclRef::new(
                                        supertrait_decl.name.clone(),
                                        supertrait_decl_id,
                                        supertrait_decl.span.clone(),
                                    ));
                                }
                            });
                }

                let decl: ty::TyDecl = decl_engine.insert(trait_decl.clone(), Some(decl_id)).into();

                trait_decl
                    .items
                    .iter_mut()
                    .for_each(|item| item.replace_implementing_type(engines, decl.clone()));

                ctx.insert_symbol(handler, name, decl.clone())?;
                decl
            }
            Declaration::ImplSelfOrTrait(decl_id) => {
                let impl_self_or_trait = &*engines.pe().get_impl_self_or_trait(decl_id);
                let mut impl_trait = if impl_self_or_trait.is_self {
                    let impl_trait_decl = match ty::TyImplSelfOrTrait::type_check_impl_self(
                        handler,
                        ctx.by_ref(),
                        decl_id,
                        impl_self_or_trait,
                    ) {
                        Ok(val) => val,
                        Err(err) => {
                            return Ok(ty::TyDecl::ErrorRecovery(
                                impl_self_or_trait.block_span.clone(),
                                err,
                            ))
                        }
                    };

                    let impl_trait =
                        if let TyDecl::ImplSelfOrTrait(impl_trait_id) = &impl_trait_decl {
                            decl_engine.get_impl_self_or_trait(&impl_trait_id.decl_id)
                        } else {
                            unreachable!();
                        };
                    ctx.insert_trait_implementation(
                        handler,
                        impl_trait.trait_name.clone(),
                        impl_trait.trait_type_arguments.clone(),
                        impl_trait.implementing_for.type_id,
                        impl_trait.impl_type_parameters.clone(),
                        &impl_trait.items,
                        &impl_trait.span,
                        impl_trait
                            .trait_decl_ref
                            .as_ref()
                            .map(|decl_ref| decl_ref.decl_span().clone()),
                        IsImplSelf::Yes,
                        IsExtendingExistingImpl::No,
                        IsImplInterfaceSurface::No,
                    )?;

                    return Ok(impl_trait_decl);
                } else {
                    match ty::TyImplSelfOrTrait::type_check_impl_trait(
                        handler,
                        ctx.by_ref(),
                        impl_self_or_trait,
                    ) {
                        Ok(res) => res,
                        Err(err) => {
                            return Ok(ty::TyDecl::ErrorRecovery(
                                impl_self_or_trait.block_span.clone(),
                                err,
                            ))
                        }
                    }
                };

                // Insert prefixed symbols when implementing_for is Contract
                let is_contract = engines
                    .te()
                    .get(impl_trait.implementing_for.type_id)
                    .is_contract();
                if is_contract {
                    for i in &impl_trait.items {
                        if let ty::TyTraitItem::Fn(f) = i {
                            let decl = engines.de().get(f.id());
                            let collecting_unifications = ctx.collecting_unifications();
                            let _ = ctx.namespace.current_module_mut().write(engines, |m| {
                                Items::insert_typed_symbol(
                                    handler,
                                    engines,
                                    m,
                                    Ident::new_no_span(format!(
                                        "__contract_entry_{}",
                                        decl.name.clone()
                                    )),
                                    TyDecl::FunctionDecl(FunctionDecl { decl_id: *f.id() }),
                                    ConstShadowingMode::ItemStyle,
                                    GenericShadowingMode::Allow,
                                    collecting_unifications,
                                )
                            });
                        }
                    }
                }

                // Choose which items are going to be visible depending if this is an abi impl
                // or trait impl
                let t = ctx.resolve_call_path(&Handler::default(), &impl_trait.trait_name);

                let empty_vec = vec![];
                let impl_trait_items = if let Ok(ty::TyDecl::TraitDecl { .. }) = t {
                    &impl_trait.items
                } else {
                    &empty_vec
                };

                ctx.insert_trait_implementation(
                    handler,
                    impl_trait.trait_name.clone(),
                    impl_trait.trait_type_arguments.clone(),
                    impl_trait.implementing_for.type_id,
                    impl_trait.impl_type_parameters.clone(),
                    impl_trait_items,
                    &impl_trait.span,
                    impl_trait
                        .trait_decl_ref
                        .as_ref()
                        .map(|decl_ref| decl_ref.decl_span().clone()),
                    IsImplSelf::No,
                    IsExtendingExistingImpl::No,
                    IsImplInterfaceSurface::No,
                )?;
                let impl_trait_decl: ty::TyDecl =
                    decl_engine.insert(impl_trait.clone(), Some(decl_id)).into();
                impl_trait.items.iter_mut().for_each(|item| {
                    item.replace_implementing_type(engines, impl_trait_decl.clone());
                });
                impl_trait_decl
            }
            Declaration::StructDeclaration(decl_id) => {
                let decl = &*engines.pe().get_struct(decl_id);
                let decl: ty::TyStructDecl =
                    match ty::TyStructDecl::type_check(handler, ctx.by_ref(), decl) {
                        Ok(res) => res,
                        Err(err) => {
                            return Ok(ty::TyDecl::ErrorRecovery(decl.span.clone(), err));
                        }
                    };
                let name = decl.call_path.suffix.clone();
                let decl: ty::TyDecl = decl_engine.insert(decl, Some(decl_id)).into();
                ctx.insert_symbol(handler, name, decl.clone())?;
                decl
            }
            Declaration::AbiDeclaration(decl_id) => {
                let abi_decl = &*engines.pe().get_abi(decl_id);
                let mut abi_decl = match ty::TyAbiDecl::type_check(handler, ctx.by_ref(), abi_decl)
                {
                    Ok(res) => res,
                    Err(err) => {
                        return Ok(ty::TyDecl::ErrorRecovery(abi_decl.span.clone(), err));
                    }
                };
                let name = abi_decl.name.clone();

                // save decl_refs for the LSP
                for supertrait in abi_decl.supertraits.iter_mut() {
                    let _ =
                        ctx.resolve_call_path(handler, &supertrait.name)
                            .map(|supertrait_decl| {
                                if let ty::TyDecl::TraitDecl(ty::TraitDecl {
                                    decl_id: supertrait_decl_id,
                                }) = supertrait_decl
                                {
                                    let supertrait_decl = engines.de().get(&supertrait_decl_id);
                                    supertrait.decl_ref = Some(DeclRef::new(
                                        supertrait_decl.name.clone(),
                                        supertrait_decl_id,
                                        supertrait_decl.span.clone(),
                                    ));
                                }
                            });
                }

                let decl: ty::TyDecl = decl_engine.insert(abi_decl.clone(), Some(decl_id)).into();
                abi_decl
                    .items
                    .iter_mut()
                    .for_each(|item| item.replace_implementing_type(engines, decl.clone()));
                ctx.insert_symbol(handler, name, decl.clone())?;
                decl
            }
            Declaration::StorageDeclaration(decl_id) => {
                fn type_check_storage_entries(
                    handler: &Handler,
                    mut ctx: TypeCheckContext,
                    entries: &[StorageEntry],
                    fields_buf: &mut Vec<TyStorageField>,
                    namespace_names: &[Ident],
                ) -> Result<(), ErrorEmitted> {
                    let engines = ctx.engines;
                    for entry in entries.iter() {
                        if let StorageEntry::Field(parsed::StorageField {
                            name,
                            key_expression,
                            initializer,
                            type_argument,
                            attributes,
                            span: field_span,
                            ..
                        }) = entry
                        {
                            let mut type_argument = type_argument.clone();
                            type_argument.type_id = ctx.by_ref().resolve_type(
                                handler,
                                type_argument.type_id,
                                &name.span(),
                                EnforceTypeArguments::Yes,
                                None,
                            )?;

                            let mut ctx = ctx
                                .by_ref()
                                .with_type_annotation(type_argument.type_id)
                                .with_storage_declaration();
                            let initializer =
                                ty::TyExpression::type_check(handler, ctx.by_ref(), initializer)?;

                            let key_expression = match key_expression {
                                Some(key_expression) => {
                                    let key_ctx = ctx
                                        .with_type_annotation(engines.te().id_of_b256())
                                        .with_help_text("Storage keys must have type \"b256\".");

                                    // TODO: Remove the `handler.scope` once https://github.com/FuelLabs/sway/issues/5606 gets solved.
                                    //       We need it here so that we can short-circuit in case of a `TypeMismatch` error which is
                                    //       not treated as an error in the `type_check()`'s result.
                                    let typed_expr = handler.scope(|handler| {
                                        ty::TyExpression::type_check(
                                            handler,
                                            key_ctx,
                                            key_expression,
                                        )
                                    })?;

                                    Some(typed_expr)
                                }
                                None => None,
                            };

                            fields_buf.push(ty::TyStorageField {
                                name: name.clone(),
                                namespace_names: namespace_names.to_vec(),
                                key_expression,
                                type_argument,
                                initializer,
                                span: field_span.clone(),
                                attributes: attributes.clone(),
                            });
                        } else if let StorageEntry::Namespace(namespace) = entry {
                            let mut new_namespace_names = namespace_names.to_vec();
                            new_namespace_names.push(namespace.name.clone());
                            type_check_storage_entries(
                                handler,
                                ctx.by_ref(),
                                &namespace
                                    .entries
                                    .iter()
                                    .map(|e| (**e).clone())
                                    .collect::<Vec<_>>(),
                                fields_buf,
                                &new_namespace_names,
                            )?;
                        }
                    }

                    Ok(())
                }

                let parsed::StorageDeclaration {
                    span,
                    entries,
                    attributes,
                    storage_keyword,
                } = &*engines.pe().get_storage(decl_id);

                let mut fields_buf = vec![];

                type_check_storage_entries(handler, ctx.by_ref(), entries, &mut fields_buf, &[])?;

                let decl = ty::TyStorageDecl {
                    fields: fields_buf,
                    span: span.clone(),
                    attributes: attributes.clone(),
                    storage_keyword: storage_keyword.clone(),
                };

                let decl_ref = decl_engine.insert(decl, Some(decl_id));

                // insert the storage declaration into the symbols
                // if there already was one, return an error that duplicate storage
                // declarations are not allowed
                ctx.namespace_mut()
                    .current_module_mut()
                    .write(engines, |m| {
                        m.current_items_mut()
                            .set_storage_declaration(handler, decl_ref.clone())
                    })?;

                decl_ref.into()
            }
            Declaration::TypeAliasDeclaration(decl_id) => {
                let decl = engines.pe().get_type_alias(decl_id);
                let span = decl.name.span();
                let name = decl.name.clone();
                let ty = &decl.ty;

                // Resolve the type that the type alias replaces
                let new_ty = ctx
                    .resolve_type(handler, ty.type_id, &span, EnforceTypeArguments::Yes, None)
                    .unwrap_or_else(|err| type_engine.id_of_error_recovery(err));

                // create the type alias decl using the resolved type above
                let decl = ty::TyTypeAliasDecl {
                    name: name.clone(),
                    call_path: CallPath::from(name.clone()).to_fullpath(engines, ctx.namespace()),
                    attributes: decl.attributes.clone(),
                    ty: GenericTypeArgument {
                        initial_type_id: ty.initial_type_id,
                        type_id: new_ty,
                        call_path_tree: ty.call_path_tree.as_ref().cloned(),
                        span: ty.span.clone(),
                    },
                    visibility: decl.visibility,
                    span,
                };

                let decl: ty::TyDecl = decl_engine.insert(decl, Some(decl_id)).into();

                ctx.insert_symbol(handler, name, decl.clone())?;
                decl
            }
            Declaration::TraitFnDeclaration(_decl_id) => {
                unreachable!();
            }
            Declaration::ConstGenericDeclaration(_) => {
                unreachable!("ConstGenericDecl is not reachable from AstNode")
            }
        };

        Ok(decl)
    }
}

impl TypeCheckAnalysis for TyDecl {
    fn type_check_analyze(
        &self,
        handler: &Handler,
        ctx: &mut TypeCheckAnalysisContext,
    ) -> Result<(), ErrorEmitted> {
        match self {
            TyDecl::VariableDecl(var_decl) => {
                var_decl.type_check_analyze(handler, ctx)?;
            }
            TyDecl::ConstantDecl(node) => {
                let const_decl = ctx.engines.de().get_constant(&node.decl_id);
                const_decl.type_check_analyze(handler, ctx)?;
            }
            TyDecl::ConfigurableDecl(node) => {
                let const_decl = ctx.engines.de().get_configurable(&node.decl_id);
                const_decl.type_check_analyze(handler, ctx)?;
            }
            TyDecl::ConstGenericDecl(_) => {
                unreachable!("ConstGenericDecl is not reachable from AstNode")
            }
            TyDecl::FunctionDecl(node) => {
                let fn_decl = ctx.engines.de().get_function(&node.decl_id);
                fn_decl.type_check_analyze(handler, ctx)?;
            }
            TyDecl::TraitDecl(node) => {
                let trait_decl = ctx.engines.de().get_trait(&node.decl_id);
                trait_decl.type_check_analyze(handler, ctx)?;
            }
            TyDecl::StructDecl(node) => {
                let struct_decl = ctx.engines.de().get_struct(&node.decl_id);
                struct_decl.type_check_analyze(handler, ctx)?;
            }
            TyDecl::EnumDecl(node) => {
                let enum_decl = ctx.engines.de().get_enum(&node.decl_id);
                enum_decl.type_check_analyze(handler, ctx)?;
            }
            TyDecl::EnumVariantDecl(_) => {}
            TyDecl::ImplSelfOrTrait(node) => {
                node.type_check_analyze(handler, ctx)?;
            }
            TyDecl::AbiDecl(node) => {
                let abi_decl = ctx.engines.de().get_abi(&node.decl_id);
                abi_decl.type_check_analyze(handler, ctx)?;
            }
            TyDecl::GenericTypeForFunctionScope(_) => {}
            TyDecl::ErrorRecovery(_, _) => {}
            TyDecl::StorageDecl(node) => {
                let storage_decl = ctx.engines.de().get_storage(&node.decl_id);
                storage_decl.type_check_analyze(handler, ctx)?;
            }
            TyDecl::TypeAliasDecl(_) => {}
            TyDecl::TraitTypeDecl(_) => {}
        }

        Ok(())
    }
}

impl TypeCheckFinalization for TyDecl {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        let decl_engine = ctx.engines.de();
        match self {
            TyDecl::VariableDecl(node) => {
                node.type_check_finalize(handler, ctx)?;
            }
            TyDecl::ConstantDecl(node) => {
                let mut const_decl = (*ctx.engines.de().get_constant(&node.decl_id)).clone();
                const_decl.type_check_finalize(handler, ctx)?;
            }
            TyDecl::ConfigurableDecl(node) => {
                let mut config_decl = (*ctx.engines.de().get_configurable(&node.decl_id)).clone();
                config_decl.type_check_finalize(handler, ctx)?;
            }
            TyDecl::ConstGenericDecl(_) => {
                unreachable!("ConstGenericDecl is not reachable from AstNode")
            }
            TyDecl::FunctionDecl(node) => {
                let mut fn_decl = (*ctx.engines.de().get_function(&node.decl_id)).clone();
                fn_decl.type_check_finalize(handler, ctx)?;
            }
            TyDecl::TraitDecl(node) => {
                let mut trait_decl = (*ctx.engines.de().get_trait(&node.decl_id)).clone();
                trait_decl.type_check_finalize(handler, ctx)?;
            }
            TyDecl::StructDecl(node) => {
                let mut struct_decl = (*ctx.engines.de().get_struct(&node.decl_id)).clone();
                struct_decl.type_check_finalize(handler, ctx)?;
            }
            TyDecl::EnumDecl(node) => {
                let mut enum_decl = (*ctx.engines.de().get_enum(&node.decl_id)).clone();
                enum_decl.type_check_finalize(handler, ctx)?;
            }
            TyDecl::EnumVariantDecl(_) => {}
            TyDecl::ImplSelfOrTrait(node) => {
                let mut impl_trait = (*decl_engine.get_impl_self_or_trait(&node.decl_id)).clone();
                impl_trait.type_check_finalize(handler, ctx)?;
            }
            TyDecl::AbiDecl(node) => {
                let mut abi_decl = (*decl_engine.get_abi(&node.decl_id)).clone();
                abi_decl.type_check_finalize(handler, ctx)?;
            }
            TyDecl::GenericTypeForFunctionScope(_) => {}
            TyDecl::ErrorRecovery(_, _) => {}
            TyDecl::StorageDecl(node) => {
                let mut storage_decl = (*decl_engine.get_storage(&node.decl_id)).clone();
                storage_decl.type_check_finalize(handler, ctx)?;
            }
            TyDecl::TypeAliasDecl(node) => {
                let mut type_alias_decl = (*decl_engine.get_type_alias(&node.decl_id)).clone();
                type_alias_decl.type_check_finalize(handler, ctx)?;
            }
            TyDecl::TraitTypeDecl(_node) => {}
        }

        Ok(())
    }
}
