pub mod code_block;
pub mod declaration;
pub mod expression;
pub mod mode;

pub use declaration::*;
pub(crate) use expression::*;
pub(crate) use mode::*;

use crate::{
    declaration_engine::{declaration_engine::*, DeclarationId},
    error::*,
    language::{parsed::*, ty},
    semantic_analysis::*,
    type_system::*,
    types::DeterministicallyAborts,
    Ident,
};

use sway_error::{
    error::CompileError,
    warning::{CompileWarning, Warning},
};
use sway_types::{span::Span, state::StateIndex, style::is_screaming_snake_case, Spanned};

impl ty::TyAstNode {
    pub(crate) fn type_check(mut ctx: TypeCheckContext, node: AstNode) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // A little utility used to check an ascribed type matches its associated expression.
        let mut type_check_ascribed_expr =
            |mut ctx: TypeCheckContext, type_ascription: TypeInfo, expr| {
                let type_id = check!(
                    ctx.resolve_type_with_self(
                        insert_type(type_ascription),
                        &node.span,
                        EnforceTypeArguments::No,
                        None
                    ),
                    insert_type(TypeInfo::ErrorRecovery),
                    warnings,
                    errors,
                );
                let ctx = ctx.with_type_annotation(type_id).with_help_text(
                    "This declaration's type annotation does not match up with the assigned \
                        expression's type.",
                );
                ty::TyExpression::type_check(ctx, expr)
            };

        let node = ty::TyAstNode {
            content: match node.content.clone() {
                AstNodeContent::UseStatement(a) => {
                    let path = if a.is_absolute {
                        a.call_path.clone()
                    } else {
                        ctx.namespace.find_module_path(&a.call_path)
                    };
                    let mut res = match a.import_type {
                        ImportType::Star => ctx.namespace.star_import(&path),
                        ImportType::SelfImport => ctx.namespace.self_import(&path, a.alias),
                        ImportType::Item(s) => ctx.namespace.item_import(&path, &s, a.alias),
                    };
                    warnings.append(&mut res.warnings);
                    errors.append(&mut res.errors);
                    ty::TyAstNodeContent::SideEffect
                }
                AstNodeContent::IncludeStatement(_) => ty::TyAstNodeContent::SideEffect,
                AstNodeContent::Declaration(a) => {
                    ty::TyAstNodeContent::Declaration(match a {
                        Declaration::VariableDeclaration(VariableDeclaration {
                            name,
                            type_ascription,
                            type_ascription_span,
                            body,
                            is_mutable,
                        }) => {
                            let type_ascription = check!(
                                ctx.resolve_type_with_self(
                                    insert_type(type_ascription),
                                    &type_ascription_span.clone().unwrap_or_else(|| name.span()),
                                    EnforceTypeArguments::Yes,
                                    None
                                ),
                                insert_type(TypeInfo::ErrorRecovery),
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
                                ty::TyExpression::error(name.span()),
                                warnings,
                                errors
                            );
                            let typed_var_decl = ty::TyDeclaration::VariableDeclaration(Box::new(
                                ty::TyVariableDeclaration {
                                    name: name.clone(),
                                    body,
                                    mutability: ty::VariableMutability::new_from_ref_mut(
                                        false, is_mutable,
                                    ),
                                    type_ascription,
                                    type_ascription_span,
                                },
                            ));
                            ctx.namespace.insert_symbol(name, typed_var_decl.clone());
                            typed_var_decl
                        }
                        Declaration::ConstantDeclaration(ConstantDeclaration {
                            name,
                            type_ascription,
                            value,
                            visibility,
                            attributes,
                            span,
                            ..
                        }) => {
                            let result =
                                type_check_ascribed_expr(ctx.by_ref(), type_ascription, value);

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
                                ty::TyExpression::error(name.span()),
                                warnings,
                                errors
                            );
                            let decl = ty::TyConstantDeclaration {
                                name: name.clone(),
                                value,
                                visibility,
                                attributes,
                                span,
                            };
                            let typed_const_decl =
                                ty::TyDeclaration::ConstantDeclaration(de_insert_constant(decl));
                            ctx.namespace.insert_symbol(name, typed_const_decl.clone());
                            typed_const_decl
                        }
                        Declaration::EnumDeclaration(decl) => {
                            let enum_decl = check!(
                                ty::TyEnumDeclaration::type_check(ctx.by_ref(), decl),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let name = enum_decl.name.clone();
                            let decl =
                                ty::TyDeclaration::EnumDeclaration(de_insert_enum(enum_decl));
                            check!(
                                ctx.namespace.insert_symbol(name, decl.clone()),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            decl
                        }
                        Declaration::FunctionDeclaration(fn_decl) => {
                            let mut ctx = ctx.with_type_annotation(insert_type(TypeInfo::Unknown));
                            let fn_decl = check!(
                                ty::TyFunctionDeclaration::type_check(
                                    ctx.by_ref(),
                                    fn_decl.clone(),
                                    false
                                ),
                                ty::TyFunctionDeclaration::error(fn_decl),
                                warnings,
                                errors
                            );

                            let name = fn_decl.name.clone();
                            let decl =
                                ty::TyDeclaration::FunctionDeclaration(de_insert_function(fn_decl));
                            ctx.namespace.insert_symbol(name, decl.clone());
                            decl
                        }
                        Declaration::TraitDeclaration(trait_decl) => {
                            let trait_decl = check!(
                                ty::TyTraitDeclaration::type_check(ctx.by_ref(), trait_decl),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let name = trait_decl.name.clone();
                            let decl_id = de_insert_trait(trait_decl);
                            let decl = ty::TyDeclaration::TraitDeclaration(decl_id);
                            ctx.namespace.insert_symbol(name, decl.clone());
                            decl
                        }
                        Declaration::ImplTrait(impl_trait) => {
                            let impl_trait = check!(
                                ty::TyImplTrait::type_check_impl_trait(ctx.by_ref(), impl_trait),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let mut methods = vec![];
                            for method_id in &impl_trait.methods {
                                match de_get_function(method_id.clone(), &impl_trait.span) {
                                    Ok(method) => methods.push(method),
                                    Err(err) => errors.push(err),
                                }
                            }
                            check!(
                                ctx.namespace.insert_trait_implementation(
                                    impl_trait.trait_name.clone(),
                                    impl_trait.trait_type_arguments.clone(),
                                    impl_trait.implementing_for_type_id,
                                    methods,
                                    &impl_trait.span,
                                ),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            ty::TyDeclaration::ImplTrait(de_insert_impl_trait(impl_trait))
                        }
                        Declaration::ImplSelf(impl_self) => {
                            let impl_trait = check!(
                                ty::TyImplTrait::type_check_impl_self(ctx.by_ref(), impl_self),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let mut methods = vec![];
                            for method_id in &impl_trait.methods {
                                match de_get_function(method_id.clone(), &impl_trait.span) {
                                    Ok(method) => methods.push(method),
                                    Err(err) => errors.push(err),
                                }
                            }
                            // specifically dont check for conflicting trait definitions
                            // because its totally ok to defined multiple impl selfs for
                            // the same type
                            ctx.namespace.insert_trait_implementation(
                                impl_trait.trait_name.clone(),
                                impl_trait.trait_type_arguments.clone(),
                                impl_trait.implementing_for_type_id,
                                methods,
                                &impl_trait.span,
                            );
                            ty::TyDeclaration::ImplTrait(de_insert_impl_trait(impl_trait))
                        }
                        Declaration::StructDeclaration(decl) => {
                            let decl = check!(
                                ty::TyStructDeclaration::type_check(ctx.by_ref(), decl),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let name = decl.name.clone();
                            let decl_id = de_insert_struct(decl);
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
                        Declaration::AbiDeclaration(abi_decl) => {
                            let abi_decl = check!(
                                ty::TyAbiDeclaration::type_check(ctx.by_ref(), abi_decl),
                                return err(warnings, errors),
                                warnings,
                                errors
                            );
                            let name = abi_decl.name.clone();
                            let decl = ty::TyDeclaration::AbiDeclaration(de_insert_abi(abi_decl));
                            ctx.namespace.insert_symbol(name, decl.clone());
                            decl
                        }
                        Declaration::StorageDeclaration(StorageDeclaration {
                            span,
                            fields,
                            attributes,
                            ..
                        }) => {
                            let mut fields_buf = Vec::with_capacity(fields.len());
                            for StorageField {
                                name,
                                type_info,
                                initializer,
                                type_info_span,
                                attributes,
                                ..
                            } in fields
                            {
                                let type_id = check!(
                                    ctx.resolve_type_without_self(
                                        insert_type(type_info),
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
                                    span: span.clone(),
                                    attributes,
                                });
                            }
                            let decl = ty::TyStorageDeclaration::new(fields_buf, span, attributes);
                            let decl_id = de_insert_storage(decl);
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
                    })
                }
                AstNodeContent::Expression(expr) => {
                    let ctx = ctx
                        .with_type_annotation(insert_type(TypeInfo::Unknown))
                        .with_help_text("");
                    let inner = check!(
                        ty::TyExpression::type_check(ctx, expr.clone()),
                        ty::TyExpression::error(expr.span()),
                        warnings,
                        errors
                    );
                    ty::TyAstNodeContent::Expression(inner)
                }
                AstNodeContent::ImplicitReturnExpression(expr) => {
                    let ctx =
                        ctx.with_help_text("Implicit return must match up with block's type.");
                    let typed_expr = check!(
                        ty::TyExpression::type_check(ctx, expr.clone()),
                        ty::TyExpression::error(expr.span()),
                        warnings,
                        errors
                    );
                    ty::TyAstNodeContent::ImplicitReturnExpression(typed_expr)
                }
            },
            span: node.span.clone(),
        };

        if let ty::TyAstNode {
            content: ty::TyAstNodeContent::Expression(ty::TyExpression { .. }),
            ..
        } = node
        {
            let warning = Warning::UnusedReturnValue {
                r#type: node.type_info().to_string(),
            };
            assert_or_warn!(
                node.type_info().can_safely_ignore(),
                warnings,
                node.span.clone(),
                warning
            );
        }

        ok(node, warnings, errors)
    }
}

fn type_check_interface_surface(
    mut ctx: TypeCheckContext,
    interface_surface: Vec<TraitFn>,
) -> CompileResult<Vec<DeclarationId>> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut typed_surface = vec![];
    for trait_fn in interface_surface.into_iter() {
        let TraitFn {
            name,
            purity,
            parameters,
            return_type,
            return_type_span,
            ..
        } = trait_fn;

        // create a namespace for the trait function
        let mut fn_namespace = ctx.namespace.clone();
        let mut fn_ctx = ctx.by_ref().scoped(&mut fn_namespace).with_purity(purity);

        // type check the parameters
        let mut typed_parameters = vec![];
        for param in parameters.into_iter() {
            typed_parameters.push(check!(
                ty::TyFunctionParameter::type_check_interface_parameter(fn_ctx.by_ref(), param),
                continue,
                warnings,
                errors
            ));
        }

        // type check the return type
        let return_type = check!(
            fn_ctx.resolve_type_with_self(
                insert_type(return_type),
                &return_type_span,
                EnforceTypeArguments::Yes,
                None
            ),
            insert_type(TypeInfo::ErrorRecovery),
            warnings,
            errors,
        );

        typed_surface.push(de_insert_trait_fn(ty::TyTraitFn {
            name,
            purity,
            return_type_span,
            parameters: typed_parameters,
            return_type,
            attributes: trait_fn.attributes,
        }));
    }
    ok(typed_surface, warnings, errors)
}

pub(crate) fn reassign_storage_subfield(
    ctx: TypeCheckContext,
    fields: Vec<Ident>,
    rhs: Expression,
    span: Span,
) -> CompileResult<ty::TyStorageReassignment> {
    let mut errors = vec![];
    let mut warnings = vec![];
    if !ctx.namespace.has_storage_declared() {
        errors.push(CompileError::NoDeclaredStorage { span });

        return err(warnings, errors);
    }

    let storage_fields = check!(
        ctx.namespace.get_storage_field_descriptors(&span),
        return err(warnings, errors),
        warnings,
        errors
    );
    let mut type_checked_buf = vec![];
    let mut fields: Vec<_> = fields.into_iter().rev().collect();

    let first_field = fields.pop().expect("guaranteed by grammar");
    let (ix, initial_field_type) = match storage_fields
        .iter()
        .enumerate()
        .find(|(_, ty::TyStorageField { name, .. })| name == &first_field)
    {
        Some((
            ix,
            ty::TyStorageField {
                type_id: r#type, ..
            },
        )) => (StateIndex::new(ix), r#type),
        None => {
            errors.push(CompileError::StorageFieldDoesNotExist {
                name: first_field.clone(),
            });
            return err(warnings, errors);
        }
    };

    type_checked_buf.push(ty::TyStorageReassignDescriptor {
        name: first_field.clone(),
        type_id: *initial_field_type,
        span: first_field.span(),
    });

    fn update_available_struct_fields(id: TypeId) -> Vec<ty::TyStructField> {
        match look_up_type_id(id) {
            TypeInfo::Struct { fields, .. } => fields,
            _ => vec![],
        }
    }
    let mut curr_type = *initial_field_type;

    // if the previously iterated type was a struct, put its fields here so we know that,
    // in the case of a subfield, we can type check the that the subfield exists and its type.
    let mut available_struct_fields = update_available_struct_fields(*initial_field_type);

    // get the initial field's type
    // make sure the next field exists in that type
    for field in fields.into_iter().rev() {
        match available_struct_fields
            .iter()
            .find(|x| x.name.as_str() == field.as_str())
        {
            Some(struct_field) => {
                curr_type = struct_field.type_id;
                type_checked_buf.push(ty::TyStorageReassignDescriptor {
                    name: field.clone(),
                    type_id: struct_field.type_id,
                    span: field.span().clone(),
                });
                available_struct_fields = update_available_struct_fields(struct_field.type_id);
            }
            None => {
                let available_fields = available_struct_fields
                    .iter()
                    .map(|x| x.name.as_str())
                    .collect::<Vec<_>>();
                errors.push(CompileError::FieldNotFound {
                    field_name: field.clone(),
                    available_fields: available_fields.join(", "),
                    struct_name: type_checked_buf.last().unwrap().name.clone(),
                });
                return err(warnings, errors);
            }
        }
    }
    let ctx = ctx.with_type_annotation(curr_type).with_help_text("");
    let rhs = check!(
        ty::TyExpression::type_check(ctx, rhs),
        ty::TyExpression::error(span),
        warnings,
        errors
    );

    ok(
        ty::TyStorageReassignment {
            fields: type_checked_buf,
            ix,
            rhs,
        },
        warnings,
        errors,
    )
}
