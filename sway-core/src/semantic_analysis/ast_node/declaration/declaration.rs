use sway_error::warning::{CompileWarning, Warning};
use sway_types::{style::is_screaming_snake_case, Spanned};

use crate::{
    decl_engine::{DeclEngineIndex, DeclRef, ReplaceFunctionImplementingType},
    error::*,
    language::{parsed, ty, CallPath},
    semantic_analysis::TypeCheckContext,
    type_system::*,
    CompileResult,
};

impl ty::TyDeclaration {
    pub(crate) fn type_check(
        mut ctx: TypeCheckContext,
        decl: parsed::Declaration,
    ) -> CompileResult<ty::TyDeclaration> {
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
                let typed_var_decl =
                    ty::TyDeclaration::VariableDeclaration(Box::new(ty::TyVariableDeclaration {
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
            parsed::Declaration::ConstantDeclaration(parsed::ConstantDeclaration {
                name,
                mut type_ascription,
                value,
                visibility,
                attributes,
                is_configurable,
                span,
            }) => {
                type_ascription.type_id = check!(
                    ctx.resolve_type(
                        type_ascription.type_id,
                        &span,
                        EnforceTypeArguments::No,
                        None
                    ),
                    type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
                    warnings,
                    errors,
                );

                let mut ctx = ctx
                    .by_ref()
                    .with_type_annotation(type_ascription.type_id)
                    .with_help_text(
                        "This declaration's type annotation does not match up with the assigned \
                        expression's type.",
                    );

                let value = match value {
                    Some(value) => {
                        let result = ty::TyExpression::type_check(ctx.by_ref(), value);

                        if !is_screaming_snake_case(name.as_str()) {
                            warnings.push(CompileWarning {
                                span: name.span(),
                                warning_content: Warning::NonScreamingSnakeCaseConstName {
                                    name: name.clone(),
                                },
                            })
                        }

                        let value = check!(
                            result,
                            ty::TyExpression::error(name.span(), engines),
                            warnings,
                            errors
                        );

                        Some(value)
                    }
                    None => None,
                };

                // Integers are special in the sense that we can't only rely on the type of `body`
                // to get the type of the variable. The type of the variable *has* to follow
                // `type_ascription` if `type_ascription` is a concrete integer type that does not
                // conflict with the type of `body` (i.e. passes the type checking above).
                type_ascription.type_id = match type_engine.get(type_ascription.type_id) {
                    TypeInfo::UnsignedInteger(_) => type_ascription.type_id,
                    _ => match &value {
                        Some(value) => value.return_type,
                        None => type_ascription.type_id,
                    },
                };

                let mut path: CallPath = name.into();
                path = path.to_fullpath(ctx.namespace);

                let decl = ty::TyConstantDeclaration {
                    call_path: path.clone(),
                    value,
                    visibility,
                    attributes,
                    type_ascription,
                    is_configurable,
                    span,
                };
                let typed_const_decl: ty::TyDeclaration = decl_engine.insert(decl).into();
                check!(
                    ctx.namespace
                        .insert_symbol(path.suffix, typed_const_decl.clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                typed_const_decl
            }
            parsed::Declaration::EnumDeclaration(decl) => {
                let span = decl.span.clone();
                let enum_decl = check!(
                    ty::TyEnumDeclaration::type_check(ctx.by_ref(), decl),
                    return ok(ty::TyDeclaration::ErrorRecovery(span), warnings, errors),
                    warnings,
                    errors
                );
                let call_path = enum_decl.call_path.clone();
                let decl: ty::TyDeclaration = decl_engine.insert(enum_decl).into();
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
                    ty::TyFunctionDeclaration::type_check(ctx.by_ref(), fn_decl, false, false),
                    return ok(ty::TyDeclaration::ErrorRecovery(span), warnings, errors),
                    warnings,
                    errors
                );
                let name = fn_decl.name.clone();
                let decl: ty::TyDeclaration = decl_engine.insert(fn_decl).into();
                ctx.namespace.insert_symbol(name, decl.clone());
                decl
            }
            parsed::Declaration::TraitDeclaration(trait_decl) => {
                let span = trait_decl.span.clone();
                let mut trait_decl = check!(
                    ty::TyTraitDeclaration::type_check(ctx.by_ref(), trait_decl),
                    return ok(ty::TyDeclaration::ErrorRecovery(span), warnings, errors),
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
                            if let ty::TyDeclaration::TraitDeclaration {
                                name: supertrait_name,
                                decl_id: supertrait_decl_id,
                                type_subst_list: _,
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

                let decl: ty::TyDeclaration = decl_engine.insert(trait_decl.clone()).into();

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
                    return ok(ty::TyDeclaration::ErrorRecovery(span), warnings, errors),
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
                let impl_trait_decl: ty::TyDeclaration =
                    decl_engine.insert(impl_trait.clone()).into();
                impl_trait.items.iter_mut().for_each(|item| match item {
                    ty::TyImplItem::Fn(method) => {
                        method.replace_implementing_type(engines, impl_trait_decl.clone())
                    }
                });
                impl_trait_decl
            }
            parsed::Declaration::ImplSelf(impl_self) => {
                let span = impl_self.block_span.clone();
                let mut impl_trait = check!(
                    ty::TyImplTrait::type_check_impl_self(ctx.by_ref(), impl_self),
                    return ok(ty::TyDeclaration::ErrorRecovery(span), warnings, errors),
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
                let impl_trait_decl: ty::TyDeclaration =
                    decl_engine.insert(impl_trait.clone()).into();
                impl_trait.items.iter_mut().for_each(|item| match item {
                    ty::TyImplItem::Fn(method) => {
                        method.replace_implementing_type(engines, impl_trait_decl.clone())
                    }
                });
                impl_trait_decl
            }
            parsed::Declaration::StructDeclaration(decl) => {
                let span = decl.span.clone();
                let decl = check!(
                    ty::TyStructDeclaration::type_check(ctx.by_ref(), decl),
                    return ok(ty::TyDeclaration::ErrorRecovery(span), warnings, errors),
                    warnings,
                    errors
                );
                let call_path = decl.call_path.clone();
                let decl: ty::TyDeclaration = decl_engine.insert(decl).into();
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
                    ty::TyAbiDeclaration::type_check(ctx.by_ref(), abi_decl),
                    return ok(ty::TyDeclaration::ErrorRecovery(span), warnings, errors),
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
                            if let ty::TyDeclaration::TraitDeclaration {
                                name: supertrait_name,
                                decl_id: supertrait_decl_id,
                                type_subst_list: _,
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

                let decl: ty::TyDeclaration = decl_engine.insert(abi_decl.clone()).into();
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
                let decl = ty::TyStorageDeclaration {
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
        };

        ok(decl, warnings, errors)
    }
}
