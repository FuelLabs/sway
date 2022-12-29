pub mod code_block;
pub mod declaration;
pub mod expression;
pub mod mode;

pub(crate) use expression::*;
pub(crate) use mode::*;

use crate::{
    error::*,
    language::{parsed::*, ty},
    semantic_analysis::*,
    type_system::*,
    types::DeterministicallyAborts,
    Ident,
};

use sway_error::{error::CompileError, warning::Warning};
use sway_types::{span::Span, state::StateIndex, Spanned};

impl ty::TyAstNode {
    pub(crate) fn type_check(ctx: TypeCheckContext, node: AstNode) -> CompileResult<Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        let type_engine = ctx.type_engine;
        let declaration_engine = ctx.declaration_engine;
        let engines = ctx.engines();

        let node = ty::TyAstNode {
            content: match node.content.clone() {
                AstNodeContent::UseStatement(a) => {
                    let path = if a.is_absolute {
                        a.call_path.clone()
                    } else {
                        ctx.namespace.find_module_path(&a.call_path)
                    };
                    let mut res = match a.import_type {
                        ImportType::Star => ctx.namespace.star_import(&path, engines),
                        ImportType::SelfImport => {
                            ctx.namespace.self_import(engines, &path, a.alias)
                        }
                        ImportType::Item(s) => {
                            ctx.namespace.item_import(engines, &path, &s, a.alias)
                        }
                    };
                    warnings.append(&mut res.warnings);
                    errors.append(&mut res.errors);
                    ty::TyAstNodeContent::SideEffect
                }
                AstNodeContent::IncludeStatement(_) => ty::TyAstNodeContent::SideEffect,
                AstNodeContent::Declaration(decl) => ty::TyAstNodeContent::Declaration(check!(
                    ty::TyDeclaration::type_check(ctx, decl),
                    return err(warnings, errors),
                    warnings,
                    errors
                )),
                AstNodeContent::Expression(expr) => {
                    let ctx = ctx
                        .with_type_annotation(
                            type_engine.insert_type(declaration_engine, TypeInfo::Unknown),
                        )
                        .with_help_text("");
                    let inner = check!(
                        ty::TyExpression::type_check(ctx, expr.clone()),
                        ty::TyExpression::error(expr.span(), engines),
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
                        ty::TyExpression::error(expr.span(), engines),
                        warnings,
                        errors
                    );
                    ty::TyAstNodeContent::ImplicitReturnExpression(typed_expr)
                }
            },
            span: node.span,
        };

        if let ty::TyAstNode {
            content: ty::TyAstNodeContent::Expression(ty::TyExpression { .. }),
            ..
        } = node
        {
            let warning = Warning::UnusedReturnValue {
                r#type: engines.help_out(node.type_info(type_engine)).to_string(),
            };
            assert_or_warn!(
                node.type_info(type_engine).can_safely_ignore(type_engine),
                warnings,
                node.span.clone(),
                warning
            );
        }

        ok(node, warnings, errors)
    }
}

pub(crate) fn reassign_storage_subfield(
    ctx: TypeCheckContext,
    fields: Vec<Ident>,
    rhs: Expression,
    span: Span,
) -> CompileResult<ty::TyStorageReassignment> {
    let mut errors = vec![];
    let mut warnings = vec![];

    let type_engine = ctx.type_engine;
    let declaration_engine = ctx.declaration_engine;
    let engines = ctx.engines();

    if !ctx.namespace.has_storage_declared() {
        errors.push(CompileError::NoDeclaredStorage { span });

        return err(warnings, errors);
    }

    let storage_fields = check!(
        ctx.namespace
            .get_storage_field_descriptors(declaration_engine, &span),
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

    let update_available_struct_fields = |id: TypeId| match type_engine.look_up_type_id(id) {
        TypeInfo::Struct { fields, .. } => fields,
        _ => vec![],
    };
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
        ty::TyExpression::error(span, engines),
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
