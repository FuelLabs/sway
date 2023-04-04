use sway_types::{Named, Spanned};

use crate::{
    decl_engine::{DeclEngineInsert, DeclRef, ReplaceFunctionImplementingType},
    error::*,
    language::{parsed, ty},
    semantic_analysis::TypeCheckContext,
    type_system::*,
    CompileResult,
};

impl ty::TyDecl {
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        decl: parsed::Declaration,
    ) -> CompileResult<ty::TyDecl> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;
        let engines = ctx.engines();

        let decl = match decl {
            parsed::Declaration::VariableDeclaration(parsed::VariableDeclaration {
                name,
                mut type_ascription,
                body,
                is_mutable,
            }) => {
                type_ascription.type_id = check!(
                    ctx.resolve_type(
                        type_ascription.type_id,
                        &type_ascription.span,
                        EnforceTypeArguments::Yes,
                        None
                    ),
                    type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
                    warnings,
                    errors
                );
                let mut ctx = ctx
                    .with_type_annotation(type_ascription.type_id)
                    .with_help_text(
                        "Variable declaration's type annotation does not match up \
                        with the assigned expression's type.",
                    );
                let result = ty::TyExpression::type_check(ctx.by_ref(), body);
                let body = check!(
                    result,
                    ty::TyExpression::error(name.span(), engines),
                    warnings,
                    errors
                );

                // Integers are special in the sense that we can't only rely on the type of `body`
                // to get the type of the variable. The type of the variable *has* to follow
                // `type_ascription` if `type_ascription` is a concrete integer type that does not
                // conflict with the type of `body` (i.e. passes the type checking above).
                let return_type = match type_engine.get(type_ascription.type_id) {
                    TypeInfo::UnsignedInteger(_) => type_ascription.type_id,
                    _ => body.return_type,
                };
                let typed_var_decl = ty::TyDecl::VariableDecl(Box::new(ty::TyVariableDecl {
                    name: name.clone(),
                    body,
                    mutability: ty::VariableMutability::new_from_ref_mut(false, is_mutable),
                    return_type,
                    type_ascription,
                }));
                check!(
                    ctx.namespace.insert_symbol(name, typed_var_decl.clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                typed_var_decl
            }
            parsed::Declaration::ConstantDeclaration(decl) => {
                let span = decl.span.clone();
                let const_decl = check!(
                    ty::TyConstantDecl::type_check(ctx.by_ref(), decl),
                    return ok(ty::TyDecl::ErrorRecovery(span), warnings, errors),
                    warnings,
                    errors
                );
                let typed_const_decl: ty::TyDecl = decl_engine
                    .insert(const_decl.clone(), SubstList::new())
                    .into();
                check!(
                    ctx.namespace
                        .insert_symbol(const_decl.name().clone(), typed_const_decl.clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                typed_const_decl
            }
            parsed::Declaration::EnumDeclaration(decl) => {
                let span = decl.span.clone();
                let (enum_decl, subst_list) = check!(
                    ty::TyEnumDecl::type_check(ctx.by_ref(), decl),
                    return ok(ty::TyDecl::ErrorRecovery(span), warnings, errors),
                    warnings,
                    errors
                );
                let call_path = enum_decl.call_path.clone();
                let decl: ty::TyDecl = decl_engine.insert(enum_decl, subst_list).into();
                check!(
                    ctx.namespace.insert_symbol(call_path.suffix, decl.clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                decl
            }
            parsed::Declaration::FunctionDeclaration(fn_decl) => {
                let span = fn_decl.span.clone();
                let mut ctx =
                    ctx.with_type_annotation(type_engine.insert(decl_engine, TypeInfo::Unknown));
                let (fn_decl, subst_list) = check!(
                    ty::TyFunctionDecl::type_check(ctx.by_ref(), fn_decl, false, false),
                    return ok(ty::TyDecl::ErrorRecovery(span), warnings, errors),
                    warnings,
                    errors
                );
                let name = fn_decl.name.clone();
                let decl: ty::TyDecl = decl_engine.insert(fn_decl, subst_list).into();
                ctx.namespace.insert_symbol(name, decl.clone());
                decl
            }
            parsed::Declaration::TraitDeclaration(trait_decl) => {
                let span = trait_decl.span.clone();
                let (mut trait_decl, subst_list) = check!(
                    ty::TyTraitDecl::type_check(ctx.by_ref(), trait_decl),
                    return ok(ty::TyDecl::ErrorRecovery(span), warnings, errors),
                    warnings,
                    errors
                );
                let name = trait_decl.name.clone();

                // save decl_refs for the LSP
                for supertrait in trait_decl.supertraits.iter_mut() {
                    ctx.namespace
                        .resolve_call_path(&supertrait.name)
                        .cloned()
                        .map(|supertrait_decl| {
                            if let ty::TyDecl::TraitDecl {
                                name: supertrait_name,
                                decl_id: supertrait_decl_id,
                                subst_list: supertrait_subst_list,
                                decl_span: supertrait_decl_span,
                            } = supertrait_decl
                            {
                                supertrait.decl_ref = Some(DeclRef::new(
                                    supertrait_name,
                                    supertrait_decl_id,
                                    supertrait_subst_list.unscoped_copy(),
                                    supertrait_decl_span,
                                ));
                            }
                        });
                }

                let decl: ty::TyDecl = decl_engine.insert(trait_decl.clone(), subst_list).into();

                trait_decl
                    .items
                    .iter_mut()
                    .for_each(|item| item.replace_implementing_type(engines, decl.clone()));
                check!(
                    ctx.namespace.insert_symbol(name, decl.clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                decl
            }
            parsed::Declaration::ImplTrait(impl_trait) => {
                let span = impl_trait.block_span.clone();
                let (mut impl_trait, subst_list) = check!(
                    ty::TyImplTrait::type_check_impl_trait(ctx.by_ref(), impl_trait),
                    return ok(ty::TyDecl::ErrorRecovery(span), warnings, errors),
                    warnings,
                    errors
                );
                check!(
                    ctx.namespace.insert_trait_implementation(
                        impl_trait.trait_name.clone(),
                        impl_trait.trait_type_arguments.clone(),
                        impl_trait.implementing_for.type_id,
                        &impl_trait.items,
                        &impl_trait.span,
                        false,
                        engines,
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let impl_trait_decl: ty::TyDecl =
                    decl_engine.insert(impl_trait.clone(), subst_list).into();
                impl_trait.items.iter_mut().for_each(|item| {
                    item.replace_implementing_type(engines, impl_trait_decl.clone());
                });
                impl_trait_decl
            }
            parsed::Declaration::ImplSelf(impl_self) => {
                let span = impl_self.block_span.clone();
                let (mut impl_trait, subst_list) = check!(
                    ty::TyImplTrait::type_check_impl_self(ctx.by_ref(), impl_self),
                    return ok(ty::TyDecl::ErrorRecovery(span), warnings, errors),
                    warnings,
                    errors
                );
                check!(
                    ctx.namespace.insert_trait_implementation(
                        impl_trait.trait_name.clone(),
                        impl_trait.trait_type_arguments.clone(),
                        impl_trait.implementing_for.type_id,
                        &impl_trait.items,
                        &impl_trait.span,
                        true,
                        engines,
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let impl_trait_decl: ty::TyDecl =
                    decl_engine.insert(impl_trait.clone(), subst_list).into();
                impl_trait.items.iter_mut().for_each(|item| {
                    item.replace_implementing_type(engines, impl_trait_decl.clone())
                });
                impl_trait_decl
            }
            parsed::Declaration::StructDeclaration(decl) => {
                let span = decl.span.clone();
                let (decl, subst_list) = check!(
                    ty::TyStructDecl::type_check(ctx.by_ref(), decl),
                    return ok(ty::TyDecl::ErrorRecovery(span), warnings, errors),
                    warnings,
                    errors
                );
                let call_path = decl.call_path.clone();
                let decl: ty::TyDecl = decl_engine.insert(decl, subst_list).into();
                // insert the struct decl into namespace
                check!(
                    ctx.namespace.insert_symbol(call_path.suffix, decl.clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                decl
            }
            parsed::Declaration::AbiDeclaration(abi_decl) => {
                let span = abi_decl.span.clone();
                let mut abi_decl = check!(
                    ty::TyAbiDecl::type_check(ctx.by_ref(), abi_decl),
                    return ok(ty::TyDecl::ErrorRecovery(span), warnings, errors),
                    warnings,
                    errors
                );
                let name = abi_decl.name.clone();

                // save decl_refs for the LSP
                for supertrait in abi_decl.supertraits.iter_mut() {
                    ctx.namespace
                        .resolve_call_path(&supertrait.name)
                        .cloned()
                        .map(|supertrait_decl| {
                            if let ty::TyDecl::TraitDecl {
                                name: supertrait_name,
                                decl_id: supertrait_decl_id,
                                subst_list: supertrait_subst_list,
                                decl_span: supertrait_decl_span,
                            } = supertrait_decl
                            {
                                supertrait.decl_ref = Some(DeclRef::new(
                                    supertrait_name,
                                    supertrait_decl_id,
                                    supertrait_subst_list.unscoped_copy(),
                                    supertrait_decl_span,
                                ));
                            }
                        });
                }

                let decl: ty::TyDecl = decl_engine
                    .insert(abi_decl.clone(), SubstList::new())
                    .into();
                abi_decl
                    .items
                    .iter_mut()
                    .for_each(|item| item.replace_implementing_type(engines, decl.clone()));
                check!(
                    ctx.namespace.insert_symbol(name, decl.clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                decl
            }
            parsed::Declaration::StorageDeclaration(parsed::StorageDeclaration {
                span,
                fields,
                attributes,
                storage_keyword,
            }) => {
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
                    type_argument.type_id = check!(
                        ctx.resolve_type(
                            type_argument.type_id,
                            &name.span(),
                            EnforceTypeArguments::Yes,
                            None
                        ),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );

                    let mut ctx = ctx.by_ref().with_type_annotation(type_argument.type_id);
                    let initializer = check!(
                        ty::TyExpression::type_check(ctx.by_ref(), initializer),
                        return err(warnings, errors),
                        warnings,
                        errors,
                    );

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
                let decl_ref = decl_engine.insert(decl, SubstList::new());
                // insert the storage declaration into the symbols
                // if there already was one, return an error that duplicate storage

                // declarations are not allowed
                check!(
                    ctx.namespace.set_storage_declaration(decl_ref.clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                decl_ref.into()
            }
            parsed::Declaration::TypeAliasDeclaration(decl) => {
                let span = decl.name.span();
                let name = decl.name.clone();
                let ty = decl.ty;

                // Resolve the type that the type alias replaces
                let new_ty = check!(
                    ctx.resolve_type(ty.type_id, &span, EnforceTypeArguments::Yes, None),
                    type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
                    warnings,
                    errors
                );

                // create the type alias decl using the resolved type above
                let decl = ty::TyTypeAliasDecl {
                    name: name.clone(),
                    attributes: decl.attributes,
                    ty: TypeArgument {
                        initial_type_id: ty.initial_type_id,
                        type_id: new_ty,
                        call_path_tree: ty.call_path_tree,
                        span: ty.span,
                    },
                    visibility: decl.visibility,
                    span,
                };

                let decl: ty::TyDecl = decl_engine.insert(decl, SubstList::new()).into();

                // insert the type alias name and decl into namespace
                check!(
                    ctx.namespace.insert_symbol(name, decl.clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                decl
            }
        };

        ok(decl, warnings, errors)
    }
}
