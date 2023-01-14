use sway_error::warning::{CompileWarning, Warning};
use sway_types::{style::is_screaming_snake_case, Spanned};

use crate::{
    decl_engine::ReplaceFunctionImplementingType,
    error::*,
    language::{parsed, ty},
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
                type_ascription,
                type_ascription_span,
                body,
                is_mutable,
            }) => {
                let type_ascription = check!(
                    ctx.resolve_type_with_self(
                        type_engine.insert(decl_engine, type_ascription),
                        &type_ascription_span.clone().unwrap_or_else(|| name.span()),
                        EnforceTypeArguments::Yes,
                        None
                    ),
                    type_engine.insert(decl_engine, TypeInfo::ErrorRecovery),
                    warnings,
                    errors
                );
                let mut ctx = ctx.with_type_annotation(type_ascription).with_help_text(
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
                let return_type = match type_engine.get(type_ascription) {
                    TypeInfo::UnsignedInteger(_) => type_ascription,
                    _ => body.return_type,
                };
                let typed_var_decl =
                    ty::TyDeclaration::VariableDeclaration(Box::new(ty::TyVariableDeclaration {
                        name: name.clone(),
                        body,
                        mutability: ty::VariableMutability::new_from_ref_mut(false, is_mutable),
                        return_type,
                        type_ascription,
                        type_ascription_span,
                    }));
                ctx.namespace.insert_symbol(name, typed_var_decl.clone());
                typed_var_decl
            }
            parsed::Declaration::ConstantDeclaration(parsed::ConstantDeclaration {
                name,
                type_ascription,
                type_ascription_span,
                value,
                visibility,
                attributes,
                is_configurable,
                span,
            }) => {
                let type_ascription = check!(
                    ctx.resolve_type_with_self(
                        type_engine.insert(decl_engine, type_ascription),
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
                    .with_type_annotation(type_ascription)
                    .with_help_text(
                        "This declaration's type annotation does not match up with the assigned \
                        expression's type.",
                    );
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
                // Integers are special in the sense that we can't only rely on the type of `body`
                // to get the type of the variable. The type of the variable *has* to follow
                // `type_ascription` if `type_ascription` is a concrete integer type that does not
                // conflict with the type of `body` (i.e. passes the type checking above).
                let return_type = match type_engine.get(type_ascription) {
                    TypeInfo::UnsignedInteger(_) => type_ascription,
                    _ => value.return_type,
                };
                let decl = ty::TyConstantDeclaration {
                    name: name.clone(),
                    value,
                    visibility,
                    return_type,
                    attributes,
                    type_ascription_span,
                    is_configurable,
                    span,
                };
                let typed_const_decl =
                    ty::TyDeclaration::ConstantDeclaration(decl_engine.insert(decl));
                ctx.namespace.insert_symbol(name, typed_const_decl.clone());
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
                let name = enum_decl.name.clone();
                let decl = ty::TyDeclaration::EnumDeclaration(decl_engine.insert(enum_decl));
                check!(
                    ctx.namespace.insert_symbol(name, decl.clone()),
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
                let decl = ty::TyDeclaration::FunctionDeclaration(decl_engine.insert(fn_decl));
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
                let decl_id = decl_engine.insert(trait_decl.clone());
                let decl = ty::TyDeclaration::TraitDeclaration(decl_id);
                trait_decl
                    .methods
                    .iter_mut()
                    .for_each(|method| method.replace_implementing_type(engines, decl.clone()));
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
                        impl_trait.implementing_for_type_id,
                        &impl_trait.methods,
                        &impl_trait.span,
                        false,
                        engines,
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let impl_trait_decl =
                    ty::TyDeclaration::ImplTrait(decl_engine.insert(impl_trait.clone()));
                impl_trait.methods.iter_mut().for_each(|method| {
                    method.replace_implementing_type(engines, impl_trait_decl.clone())
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
                        impl_trait.implementing_for_type_id,
                        &impl_trait.methods,
                        &impl_trait.span,
                        true,
                        engines,
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let impl_trait_decl =
                    ty::TyDeclaration::ImplTrait(decl_engine.insert(impl_trait.clone()));
                impl_trait.methods.iter_mut().for_each(|method| {
                    method.replace_implementing_type(engines, impl_trait_decl.clone())
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
                let name = decl.name.clone();
                let decl_id = decl_engine.insert(decl);
                let decl = ty::TyDeclaration::StructDeclaration(decl_id);
                // insert the struct decl into namespace
                check!(
                    ctx.namespace.insert_symbol(name, decl.clone()),
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
                let decl = ty::TyDeclaration::AbiDeclaration(decl_engine.insert(abi_decl.clone()));
                abi_decl
                    .methods
                    .iter_mut()
                    .for_each(|method| method.replace_implementing_type(engines, decl.clone()));
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
                ..
            }) => {
                let mut fields_buf = Vec::with_capacity(fields.len());
                for parsed::StorageField {
                    name,
                    type_info,
                    initializer,
                    type_info_span,
                    attributes,
                    span: field_span,
                    ..
                } in fields
                {
                    let type_id = check!(
                        ctx.resolve_type_without_self(
                            type_engine.insert(decl_engine, type_info),
                            &name.span(),
                            None
                        ),
                        return err(warnings, errors),
                        warnings,
                        errors
                    );

                    let mut ctx = ctx.by_ref().with_type_annotation(type_id);
                    let initializer = check!(
                        ty::TyExpression::type_check(ctx.by_ref(), initializer),
                        return err(warnings, errors),
                        warnings,
                        errors,
                    );

                    fields_buf.push(ty::TyStorageField {
                        name,
                        type_id,
                        type_span: type_info_span,
                        initializer,
                        span: field_span,
                        attributes,
                    });
                }
                let decl = ty::TyStorageDeclaration::new(fields_buf, span, attributes);
                let decl_id = decl_engine.insert(decl);
                // insert the storage declaration into the symbols
                // if there already was one, return an error that duplicate storage

                // declarations are not allowed
                check!(
                    ctx.namespace.set_storage_declaration(decl_id.clone()),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                ty::TyDeclaration::StorageDeclaration(decl_id)
            }
        };

        ok(decl, warnings, errors)
    }
}
