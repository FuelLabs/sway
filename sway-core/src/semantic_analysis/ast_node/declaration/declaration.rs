use sway_types::{Named, Spanned};

use crate::{
    decl_engine::{DeclEngineInsert, DeclRef, ReplaceFunctionImplementingType},
    error::*,
    language::{parsed, ty, ty::TyImplItem},
    semantic_analysis::TypeCheckContext,
    type_system::*,
    CompileResult,
};

use std::collections::{BTreeMap, HashSet};

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
                    ctx.resolve_type_with_self(
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
                let typed_const_decl: ty::TyDecl = decl_engine.insert(const_decl.clone()).into();
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
                let enum_decl = check!(
                    ty::TyEnumDecl::type_check(ctx.by_ref(), decl),
                    return ok(ty::TyDecl::ErrorRecovery(span), warnings, errors),
                    warnings,
                    errors
                );
                let call_path = enum_decl.call_path.clone();
                let decl: ty::TyDecl = decl_engine.insert(enum_decl).into();
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
                let fn_decl = check!(
                    ty::TyFunctionDecl::type_check(ctx.by_ref(), fn_decl, false, false),
                    return ok(ty::TyDecl::ErrorRecovery(span), warnings, errors),
                    warnings,
                    errors
                );
                let name = fn_decl.name.clone();
                let decl: ty::TyDecl = decl_engine.insert(fn_decl).into();
                ctx.namespace.insert_symbol(name, decl.clone());
                decl
            }
            parsed::Declaration::TraitDeclaration(trait_decl) => {
                let span = trait_decl.span.clone();
                let mut trait_decl = check!(
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
                                subst_list: _,
                                decl_span: supertrait_decl_span,
                            } = supertrait_decl
                            {
                                supertrait.decl_ref = Some(DeclRef::new(
                                    supertrait_name,
                                    supertrait_decl_id,
                                    supertrait_decl_span,
                                ));
                            }
                        });
                }

                let decl: ty::TyDecl = decl_engine.insert(trait_decl.clone()).into();

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
                let mut impl_trait = check!(
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

                let impl_trait_decl: ty::TyDecl = decl_engine.insert(impl_trait.clone()).into();
                impl_trait.items.iter_mut().for_each(|item| {
                    item.replace_implementing_type(engines, impl_trait_decl.clone());
                });
                impl_trait_decl
            }
            parsed::Declaration::ImplSelf(impl_self) => {
                let span = impl_self.block_span.clone();
                let mut impl_trait = check!(
                    ty::TyImplTrait::type_check_impl_self(ctx.by_ref(), impl_self),
                    return ok(ty::TyDecl::ErrorRecovery(span), warnings, errors),
                    warnings,
                    errors
                );

                // Set of all type IDs that have corresponding items implemented for them in this
                // `impl` block. In most cases, this is a single type ID that is equal to
                // `impl_trait.implementing_for.type_id`.
                let mut processed_ids = HashSet::new();

                // This is a map from type IDs to a vector of impl items. In most cases, all the
                // items are available to `impl_trait.implementing_for.type_id` (i.e. `Self`).
                // However, there are situations where `self`, in some of the impl methods, has a
                // type ascription that is different from `Self`. In that case, we have to keep
                // track of which impl items are available to which type ID.
                //
                // For now, only `std::core::experimental::StorageHandle` is allowed as a type
                // ascription for `self`, so check that here as well
                let type_id_to_impl_items: BTreeMap<TypeId, Vec<TyImplItem>> = impl_trait
                    .items
                    .iter()
                    .map(|item| match item {
                        TyImplItem::Fn(func) => {
                            // For function items, check if the first argument is `self`. If so,
                            // map the corresponding type ID to this item. Make sure that type IDs
                            // that are subset of each other are uniqued using `processed_ids`.
                            let func = decl_engine.get_function(func);
                            (
                                match func.parameters.first() {
                                    Some(first_arg) if first_arg.is_self() => {
                                        let self_type_id = first_arg.type_argument.type_id;
                                        match processed_ids.iter().find(|p| {
                                            type_engine
                                                .get(self_type_id)
                                                .is_subset_of(&type_engine.get(**p), engines)
                                        }) {
                                            Some(id) => *id,
                                            _ => {
                                                processed_ids.insert(self_type_id);
                                                self_type_id
                                            }
                                        }
                                    }
                                    _ => impl_trait.implementing_for.type_id,
                                },
                                item,
                            )
                        }
                        _ => {
                            // Other items are only available for `Self`
                            (impl_trait.implementing_for.type_id, item)
                        }
                    })
                    .fold(
                        BTreeMap::new(),
                        |mut acc: BTreeMap<TypeId, Vec<_>>, (type_id, item)| {
                            acc.entry(type_id)
                                .and_modify(|v| v.push(item.clone()))
                                .or_insert(vec![item.clone()]);
                            acc
                        },
                    );

                // For each type ID discovered in type_id_to_impl_items, insert the collected items
                // into the trait map
                for type_id in type_id_to_impl_items.keys() {
                    check!(
                        ctx.namespace.insert_trait_implementation(
                            impl_trait.trait_name.clone(),
                            impl_trait.trait_type_arguments.clone(),
                            *type_id,
                            &type_id_to_impl_items[type_id],
                            &impl_trait.span,
                            true,
                            engines,
                        ),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );
                }

                let impl_trait_decl: ty::TyDecl = decl_engine.insert(impl_trait.clone()).into();
                impl_trait.items.iter_mut().for_each(|item| {
                    item.replace_implementing_type(engines, impl_trait_decl.clone())
                });
                impl_trait_decl
            }
            parsed::Declaration::StructDeclaration(decl) => {
                let span = decl.span.clone();
                let decl = check!(
                    ty::TyStructDecl::type_check(ctx.by_ref(), decl),
                    return ok(ty::TyDecl::ErrorRecovery(span), warnings, errors),
                    warnings,
                    errors
                );
                let call_path = decl.call_path.clone();
                let decl: ty::TyDecl = decl_engine.insert(decl).into();
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
                                subst_list: _,
                                decl_span: supertrait_decl_span,
                            } = supertrait_decl
                            {
                                supertrait.decl_ref = Some(DeclRef::new(
                                    supertrait_name,
                                    supertrait_decl_id,
                                    supertrait_decl_span,
                                ));
                            }
                        });
                }

                let decl: ty::TyDecl = decl_engine.insert(abi_decl.clone()).into();
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
                        ctx.resolve_type_without_self(type_argument.type_id, &name.span(), None),
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
                let decl_ref = decl_engine.insert(decl);
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
                    ctx.resolve_type_with_self(ty.type_id, &span, EnforceTypeArguments::Yes, None),
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

                let decl: ty::TyDecl = decl_engine.insert(decl).into();

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
