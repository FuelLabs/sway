use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::{BaseIdent, Ident, Named, Spanned};

use crate::{
    decl_engine::{DeclEngineGet, DeclEngineInsert, DeclRef, ReplaceFunctionImplementingType},
    language::{
        parsed,
        ty::{self, FunctionDecl, TyDecl},
        CallPath,
    },
    namespace::{IsExtendingExistingImpl, IsImplSelf},
    semantic_analysis::{
        collection_context::SymbolCollectionContext, type_check_context::EnforceTypeArguments,
        ConstShadowingMode, GenericShadowingMode, TypeCheckAnalysis, TypeCheckAnalysisContext,
        TypeCheckContext, TypeCheckFinalization, TypeCheckFinalizationContext,
    },
    type_system::*,
    Engines,
};

impl TyDecl {
    pub(crate) fn collect(
        handler: &Handler,
        engines: &Engines,
        ctx: &mut SymbolCollectionContext,
        decl: parsed::Declaration,
    ) -> Result<(), ErrorEmitted> {
        match &decl {
            parsed::Declaration::VariableDeclaration(decl_id) => {
                let var_decl = engines.pe().get_variable(decl_id);
                ctx.insert_parsed_symbol(handler, engines, var_decl.name.clone(), decl)?;
            }
            parsed::Declaration::ConstantDeclaration(decl_id) => {
                let const_decl = engines.pe().get_constant(decl_id).as_ref().clone();
                ctx.insert_parsed_symbol(handler, engines, const_decl.name.clone(), decl)?;
            }
            parsed::Declaration::TraitTypeDeclaration(decl_id) => {
                let trait_type_decl = engines.pe().get_trait_type(decl_id).as_ref().clone();
                ctx.insert_parsed_symbol(handler, engines, trait_type_decl.name.clone(), decl)?;
            }
            parsed::Declaration::EnumDeclaration(decl_id) => {
                let enum_decl = engines.pe().get_enum(decl_id).as_ref().clone();
                ctx.insert_parsed_symbol(handler, engines, enum_decl.name.clone(), decl)?;
            }
            parsed::Declaration::EnumVariantDeclaration(_decl) => {}
            parsed::Declaration::FunctionDeclaration(decl_id) => {
                let fn_decl = engines.pe().get_function(decl_id);
                let _ = ctx.insert_parsed_symbol(handler, engines, fn_decl.name.clone(), decl);
            }
            parsed::Declaration::TraitDeclaration(decl_id) => {
                let trait_decl = engines.pe().get_trait(decl_id).as_ref().clone();
                ctx.insert_parsed_symbol(handler, engines, trait_decl.name.clone(), decl)?;
            }
            parsed::Declaration::ImplTrait(decl_id) => {
                let impl_trait = engines.pe().get_impl_trait(decl_id).as_ref().clone();
                ctx.insert_parsed_symbol(
                    handler,
                    engines,
                    impl_trait.trait_name.suffix.clone(),
                    decl,
                )?;
            }
            parsed::Declaration::ImplSelf(decl_id) => {
                let impl_self = engines.pe().get_impl_self(decl_id).as_ref().clone();
                ctx.insert_parsed_symbol(
                    handler,
                    engines,
                    BaseIdent::new(impl_self.implementing_for.span),
                    decl,
                )?;
            }
            parsed::Declaration::StructDeclaration(decl_id) => {
                let struct_decl = engines.pe().get_struct(decl_id).as_ref().clone();
                ctx.insert_parsed_symbol(handler, engines, struct_decl.name.clone(), decl)?;
            }
            parsed::Declaration::AbiDeclaration(decl_id) => {
                let abi_decl = engines.pe().get_abi(decl_id).as_ref().clone();
                ctx.insert_parsed_symbol(handler, engines, abi_decl.name.clone(), decl)?;
            }
            parsed::Declaration::StorageDeclaration(decl_id) => {
                let _storage_decl = engines.pe().get_storage(decl_id).as_ref().clone();
                //ctx.insert_parsed_symbol(handler, storage_decl.name.clone(), decl)?;
            }
            parsed::Declaration::TypeAliasDeclaration(decl_id) => {
                let type_alias_decl = engines.pe().get_type_alias(decl_id).as_ref().clone();
                ctx.insert_parsed_symbol(handler, engines, type_alias_decl.name, decl.clone())?;
            }
        };

        Ok(())
    }

    pub(crate) fn type_check(
        handler: &Handler,
        mut ctx: TypeCheckContext,
        decl: parsed::Declaration,
    ) -> Result<ty::TyDecl, ErrorEmitted> {
        let type_engine = ctx.engines.te();
        let decl_engine = ctx.engines.de();
        let engines = ctx.engines();

        let decl = match decl {
            parsed::Declaration::VariableDeclaration(decl_id) => {
                let var_decl = engines.pe().get_variable(&decl_id);
                let mut type_ascription = var_decl.type_ascription.clone();

                type_ascription.type_id = ctx
                    .resolve_type(
                        handler,
                        type_ascription.type_id,
                        &type_ascription.span,
                        EnforceTypeArguments::Yes,
                        None,
                    )
                    .unwrap_or_else(|err| {
                        type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None)
                    });
                let mut ctx = ctx
                    .with_type_annotation(type_ascription.type_id)
                    .with_help_text(
                        "Variable declaration's type annotation does not match up \
                        with the assigned expression's type.",
                    );
                let result = ty::TyExpression::type_check(handler, ctx.by_ref(), &var_decl.body);
                let body = result.unwrap_or_else(|err| {
                    ty::TyExpression::error(err, var_decl.name.span(), engines)
                });

                // TODO: Integers shouldn't be anything special. RHS expressions should be written in
                //       a way to always use the context provided from the LHS, and if the LHS is
                //       an integer, RHS should properly unify or type check should fail.
                //       Remove this special case as a part of the initiative of improving type inference.
                // Integers are special in the sense that we can't only rely on the type of `body`
                // to get the type of the variable. The type of the variable *has* to follow
                // `type_ascription` if `type_ascription` is a concrete integer type that does not
                // conflict with the type of `body` (i.e. passes the type checking above).
                let return_type = match &*type_engine.get(type_ascription.type_id) {
                    TypeInfo::UnsignedInteger(_) => type_ascription.type_id,
                    _ => match &*type_engine.get(body.return_type) {
                        // If RHS type check ends up in an error we want to use the
                        // provided type ascription as the variable type. E.g.:
                        //   let v: Struct<u8> = Struct<u64> { x: 0 }; // `v` should be "Struct<u8>".
                        //   let v: ExistingType = non_existing_identifier; // `v` should be "ExistingType".
                        //   let v = <some error>; // `v` will remain "{unknown}".
                        // TODO: Refine and improve this further. E.g.,
                        //   let v: Struct { /* MISSING FIELDS */ }; // Despite the error, `v` should be of type "Struct".
                        TypeInfo::ErrorRecovery(_) => type_ascription.type_id,
                        _ => body.return_type,
                    },
                };

                let typed_var_decl = ty::TyDecl::VariableDecl(Box::new(ty::TyVariableDecl {
                    name: var_decl.name.clone(),
                    body,
                    mutability: ty::VariableMutability::new_from_ref_mut(
                        false,
                        var_decl.is_mutable,
                    ),
                    return_type,
                    type_ascription,
                }));
                ctx.insert_symbol(handler, var_decl.name.clone(), typed_var_decl.clone())?;
                typed_var_decl
            }
            parsed::Declaration::ConstantDeclaration(decl_id) => {
                let decl = engines.pe().get_constant(&decl_id).as_ref().clone();
                let span = decl.span.clone();
                let const_decl = match ty::TyConstantDecl::type_check(handler, ctx.by_ref(), decl) {
                    Ok(res) => res,
                    Err(err) => return Ok(ty::TyDecl::ErrorRecovery(span, err)),
                };
                let typed_const_decl: ty::TyDecl = decl_engine.insert(const_decl.clone()).into();
                ctx.insert_symbol(handler, const_decl.name().clone(), typed_const_decl.clone())?;
                typed_const_decl
            }
            parsed::Declaration::TraitTypeDeclaration(decl_id) => {
                let decl = engines.pe().get_trait_type(&decl_id).as_ref().clone();
                let span = decl.span.clone();
                let type_decl = match ty::TyTraitType::type_check(handler, ctx.by_ref(), decl) {
                    Ok(res) => res,
                    Err(err) => return Ok(ty::TyDecl::ErrorRecovery(span, err)),
                };
                let typed_type_decl: ty::TyDecl = decl_engine.insert(type_decl.clone()).into();
                ctx.insert_symbol(handler, type_decl.name().clone(), typed_type_decl.clone())?;
                typed_type_decl
            }
            parsed::Declaration::EnumDeclaration(decl_id) => {
                let decl = engines.pe().get_enum(&decl_id).as_ref().clone();
                let span = decl.span.clone();
                let enum_decl = match ty::TyEnumDecl::type_check(handler, ctx.by_ref(), decl) {
                    Ok(res) => res,
                    Err(err) => return Ok(ty::TyDecl::ErrorRecovery(span, err)),
                };
                let call_path = enum_decl.call_path.clone();
                let decl: ty::TyDecl = decl_engine.insert(enum_decl).into();
                ctx.insert_symbol(handler, call_path.suffix, decl.clone())?;

                decl
            }
            parsed::Declaration::EnumVariantDeclaration(_decl) => {
                // Type-checked above as part of the containing enum.
                unreachable!()
            }
            parsed::Declaration::FunctionDeclaration(decl_id) => {
                let fn_decl = engines.pe().get_function(&decl_id);
                let span = fn_decl.span.clone();

                let mut ctx =
                    ctx.with_type_annotation(type_engine.insert(engines, TypeInfo::Unknown, None));
                let fn_decl = match ty::TyFunctionDecl::type_check(
                    handler,
                    ctx.by_ref(),
                    &fn_decl,
                    false,
                    false,
                    None,
                ) {
                    Ok(res) => res,
                    Err(err) => return Ok(ty::TyDecl::ErrorRecovery(span, err)),
                };

                let name = fn_decl.name.clone();
                let decl: ty::TyDecl = decl_engine.insert(fn_decl).into();
                let _ = ctx.insert_symbol(handler, name, decl.clone());
                decl
            }
            parsed::Declaration::TraitDeclaration(decl_id) => {
                let trait_decl = engines.pe().get_trait(&decl_id).as_ref().clone();
                let span = trait_decl.span.clone();
                let mut trait_decl =
                    match ty::TyTraitDecl::type_check(handler, ctx.by_ref(), trait_decl) {
                        Ok(res) => res,
                        Err(err) => return Ok(ty::TyDecl::ErrorRecovery(span, err)),
                    };
                let name = trait_decl.name.clone();

                // save decl_refs for the LSP
                for supertrait in trait_decl.supertraits.iter_mut() {
                    let _ = ctx
                        .namespace()
                        .resolve_call_path_typed(
                            handler,
                            engines,
                            &supertrait.name,
                            ctx.self_type(),
                        )
                        .map(|supertrait_decl| {
                            if let ty::TyDecl::TraitDecl(ty::TraitDecl {
                                decl_id: supertrait_decl_id,
                            }) = supertrait_decl
                            {
                                supertrait.decl_ref = Some(DeclRef::new(
                                    engines.de().get(&supertrait_decl_id).name.clone(),
                                    supertrait_decl_id,
                                    engines.de().get(&supertrait_decl_id).span.clone(),
                                ));
                            }
                        });
                }

                let decl: ty::TyDecl = decl_engine.insert(trait_decl.clone()).into();

                trait_decl
                    .items
                    .iter_mut()
                    .for_each(|item| item.replace_implementing_type(engines, decl.clone()));
                ctx.insert_symbol(handler, name, decl.clone())?;
                decl
            }
            parsed::Declaration::ImplTrait(decl_id) => {
                let impl_trait = engines.pe().get_impl_trait(&decl_id).as_ref().clone();
                let span = impl_trait.block_span.clone();
                let mut impl_trait =
                    match ty::TyImplTrait::type_check_impl_trait(handler, ctx.by_ref(), impl_trait)
                    {
                        Ok(res) => res,
                        Err(err) => return Ok(ty::TyDecl::ErrorRecovery(span, err)),
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
                            let _ = ctx.namespace.module_mut(ctx.engines()).write(engines, |m| {
                                m.current_items_mut().insert_typed_symbol(
                                    handler,
                                    engines,
                                    Ident::new_no_span(format!(
                                        "__contract_entry_{}",
                                        decl.name.clone()
                                    )),
                                    TyDecl::FunctionDecl(FunctionDecl { decl_id: *f.id() }),
                                    ConstShadowingMode::ItemStyle,
                                    GenericShadowingMode::Allow,
                                )
                            });
                        }
                    }
                }

                // Choose which items are going to be visible depending if this is an abi impl
                // or trait impl
                let t = ctx.namespace().resolve_call_path_typed(
                    &Handler::default(),
                    engines,
                    &impl_trait.trait_name,
                    ctx.self_type(),
                );

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
                    impl_trait_items,
                    &impl_trait.span,
                    impl_trait
                        .trait_decl_ref
                        .as_ref()
                        .map(|decl_ref| decl_ref.decl_span().clone()),
                    IsImplSelf::No,
                    IsExtendingExistingImpl::No,
                )?;
                let impl_trait_decl: ty::TyDecl = decl_engine.insert(impl_trait.clone()).into();
                impl_trait.items.iter_mut().for_each(|item| {
                    item.replace_implementing_type(engines, impl_trait_decl.clone());
                });
                impl_trait_decl
            }
            parsed::Declaration::ImplSelf(decl_id) => {
                let impl_self = engines.pe().get_impl_self(&decl_id).as_ref().clone();
                let span = impl_self.block_span.clone();
                let impl_trait_decl =
                    match ty::TyImplTrait::type_check_impl_self(handler, ctx.by_ref(), impl_self) {
                        Ok(val) => val,
                        Err(err) => return Ok(ty::TyDecl::ErrorRecovery(span, err)),
                    };
                let impl_trait = if let TyDecl::ImplTrait(impl_trait_id) = &impl_trait_decl {
                    decl_engine.get_impl_trait(&impl_trait_id.decl_id)
                } else {
                    unreachable!();
                };
                ctx.insert_trait_implementation(
                    handler,
                    impl_trait.trait_name.clone(),
                    impl_trait.trait_type_arguments.clone(),
                    impl_trait.implementing_for.type_id,
                    &impl_trait.items,
                    &impl_trait.span,
                    impl_trait
                        .trait_decl_ref
                        .as_ref()
                        .map(|decl_ref| decl_ref.decl_span().clone()),
                    IsImplSelf::Yes,
                    IsExtendingExistingImpl::No,
                )?;
                impl_trait_decl
            }
            parsed::Declaration::StructDeclaration(decl_id) => {
                let decl = engines.pe().get_struct(&decl_id).as_ref().clone();
                let span = decl.span.clone();
                let decl: ty::TyStructDecl =
                    match ty::TyStructDecl::type_check(handler, ctx.by_ref(), decl) {
                        Ok(res) => res,
                        Err(err) => {
                            return Ok(ty::TyDecl::ErrorRecovery(span, err));
                        }
                    };
                let call_path = decl.call_path.clone();
                let decl: ty::TyDecl = decl_engine.insert(decl).into();

                // insert the struct decl into namespace
                ctx.insert_symbol(handler, call_path.suffix, decl.clone())?;

                decl
            }
            parsed::Declaration::AbiDeclaration(decl_id) => {
                let abi_decl = engines.pe().get_abi(&decl_id).as_ref().clone();
                let span = abi_decl.span.clone();
                let mut abi_decl = match ty::TyAbiDecl::type_check(handler, ctx.by_ref(), abi_decl)
                {
                    Ok(res) => res,
                    Err(err) => {
                        return Ok(ty::TyDecl::ErrorRecovery(span, err));
                    }
                };
                let name = abi_decl.name.clone();

                // save decl_refs for the LSP
                for supertrait in abi_decl.supertraits.iter_mut() {
                    let _ = ctx
                        .namespace()
                        .resolve_call_path_typed(
                            handler,
                            engines,
                            &supertrait.name,
                            ctx.self_type(),
                        )
                        .map(|supertrait_decl| {
                            if let ty::TyDecl::TraitDecl(ty::TraitDecl {
                                decl_id: supertrait_decl_id,
                            }) = supertrait_decl
                            {
                                supertrait.decl_ref = Some(DeclRef::new(
                                    engines.de().get(&supertrait_decl_id).name.clone(),
                                    supertrait_decl_id,
                                    engines.de().get(&supertrait_decl_id).span.clone(),
                                ));
                            }
                        });
                }

                let decl: ty::TyDecl = decl_engine.insert(abi_decl.clone()).into();
                abi_decl
                    .items
                    .iter_mut()
                    .for_each(|item| item.replace_implementing_type(engines, decl.clone()));
                ctx.insert_symbol(handler, name, decl.clone())?;
                decl
            }
            parsed::Declaration::StorageDeclaration(decl_id) => {
                let parsed::StorageDeclaration {
                    span,
                    fields,
                    attributes,
                    storage_keyword,
                } = engines.pe().get_storage(&decl_id).as_ref().clone();
                let mut fields_buf = Vec::with_capacity(fields.len());
                for parsed::StorageField {
                    name,
                    initializer,
                    mut type_argument,
                    attributes,
                    span: field_span,
                    ..
                } in fields
                {
                    type_argument.type_id = ctx.resolve_type(
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
                        ty::TyExpression::type_check(handler, ctx.by_ref(), &initializer)?;

                    fields_buf.push(ty::TyStorageField {
                        name,
                        type_argument,
                        initializer,
                        span: field_span,
                        attributes,
                    });
                }
                let decl = ty::TyStorageDecl {
                    fields: fields_buf,
                    span,
                    attributes,
                    storage_keyword,
                };
                let decl_ref = decl_engine.insert(decl);
                // insert the storage declaration into the symbols
                // if there already was one, return an error that duplicate storage

                // declarations are not allowed
                ctx.namespace_mut()
                    .module_mut(engines)
                    .write(engines, |m| {
                        m.current_items_mut()
                            .set_storage_declaration(handler, decl_ref.clone())
                    })?;
                decl_ref.into()
            }
            parsed::Declaration::TypeAliasDeclaration(decl_id) => {
                let decl = engines.pe().get_type_alias(&decl_id);
                let span = decl.name.span();
                let name = decl.name.clone();
                let ty = &decl.ty;

                // Resolve the type that the type alias replaces
                let new_ty = ctx
                    .resolve_type(handler, ty.type_id, &span, EnforceTypeArguments::Yes, None)
                    .unwrap_or_else(|err| {
                        type_engine.insert(engines, TypeInfo::ErrorRecovery(err), None)
                    });

                // create the type alias decl using the resolved type above
                let decl = ty::TyTypeAliasDecl {
                    name: name.clone(),
                    call_path: CallPath::from(name.clone()).to_fullpath(engines, ctx.namespace()),
                    attributes: decl.attributes.clone(),
                    ty: TypeArgument {
                        initial_type_id: ty.initial_type_id,
                        type_id: new_ty,
                        call_path_tree: ty.call_path_tree.clone(),
                        span: ty.span.clone(),
                    },
                    visibility: decl.visibility,
                    span,
                };

                let decl: ty::TyDecl = decl_engine.insert(decl).into();

                // insert the type alias name and decl into namespace
                ctx.insert_symbol(handler, name, decl.clone())?;
                decl
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
            TyDecl::VariableDecl(node) => {
                node.type_check_analyze(handler, ctx)?;
            }
            TyDecl::ConstantDecl(node) => {
                let const_decl = ctx.engines.de().get_constant(&node.decl_id);
                const_decl.type_check_analyze(handler, ctx)?;
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
            TyDecl::ImplTrait(node) => {
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
            TyDecl::ImplTrait(node) => {
                let mut impl_trait = (*decl_engine.get_impl_trait(&node.decl_id)).clone();
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
