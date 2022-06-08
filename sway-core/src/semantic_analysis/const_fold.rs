use super::{
    TypedAstNode, TypedAstNodeContent, TypedCodeBlock, TypedDeclaration, TypedExpression,
    TypedExpressionVariant, TypedFunctionDeclaration, TypedStorageDeclaration,
    TypedStructExpressionField,
};

use std::collections::HashMap;
use sway_ir::{constant::Constant, context::Context, irtype::Aggregate};
use sway_types::{ident::Ident, span::Span};

/// Given an environment mapping names to constants,
/// attempt to evaluate a typed expression to a constant.
fn const_fold_typed_expr(
    context: &mut Context,
    known_consts: &HashMap<&Ident, Vec<&Constant>>,
    expr: &TypedExpression,
) -> Option<Constant> {
    match &expr.expression {
        TypedExpressionVariant::Literal(l) => Some(crate::optimize::convert_literal_to_constant(l)),
        TypedExpressionVariant::FunctionApplication {
            arguments,
            function_body,
            ..
        } => None,
        TypedExpressionVariant::LazyOperator { .. } => None,
        TypedExpressionVariant::VariableExpression { name } => known_consts
            .get(name)
            .map(|cvs| (*cvs.last().unwrap()).clone()),
        TypedExpressionVariant::Tuple { fields } => None,
        TypedExpressionVariant::Array { contents } => None,
        TypedExpressionVariant::ArrayIndex { .. } => None,
        TypedExpressionVariant::StructExpression { fields, .. } => {
            let (field_typs, field_vals): (Vec<_>, Vec<_>) = fields
                .iter()
                .filter_map(|TypedStructExpressionField { name: _, value }| {
                    const_fold_typed_expr(context, known_consts, value)
                        .and_then(|cv| Some((value.return_type, cv)))
                })
                .unzip();

            if field_vals.len() < fields.len() {
                // We couldn't evaluate all fields to a constant.
                return None;
            }
            let aggregate = crate::optimize::get_aggregate_for_types(context, &field_typs).unwrap();
            Some(Constant::new_struct(&aggregate, field_vals))
        }
        TypedExpressionVariant::CodeBlock(block) => None,
        TypedExpressionVariant::FunctionParameter => None,
        TypedExpressionVariant::IfExp { .. } => None,
        TypedExpressionVariant::AsmExpression { .. } => None,
        // like a variable expression but it has multiple parts,
        // like looking up a field in a struct
        TypedExpressionVariant::StructFieldAccess {
            field_to_access, ..
        } => None,
        TypedExpressionVariant::TupleElemAccess {
            elem_to_access_num, ..
        } => None,
        TypedExpressionVariant::EnumInstantiation {
            variant_name,
            tag,
            contents,
            ..
        } => None,
        TypedExpressionVariant::AbiCast { .. } => None,
        TypedExpressionVariant::StorageAccess(_) => None,
        TypedExpressionVariant::TypeProperty { .. } => None,
        TypedExpressionVariant::GetStorageKey { .. } => None,
        TypedExpressionVariant::SizeOfValue { expr } => None,
        TypedExpressionVariant::AbiName(_) => None,
        TypedExpressionVariant::EnumTag { exp } => None,
        TypedExpressionVariant::UnsafeDowncast { .. } => None,
    }
}

fn const_fold_typed_ast_node(
    context: &mut Context,
    known_consts: &HashMap<&Ident, Vec<&Constant>>,
    expr: &TypedAstNode,
) -> Option<Constant> {
    match &expr.content {
        TypedAstNodeContent::ReturnStatement(trs) => {
            const_fold_typed_expr(context, known_consts, &trs.expr)
        }
        TypedAstNodeContent::Declaration(td) => {
            // TODO: add the binding to known_consts (if it's a const) and proceed.
            None
        }
        TypedAstNodeContent::Expression(e) | TypedAstNodeContent::ImplicitReturnExpression(e) => {
            const_fold_typed_expr(context, known_consts, &e)
        }
        TypedAstNodeContent::WhileLoop(_) | TypedAstNodeContent::SideEffect => None,
    }
}

pub fn const_fold_typed_application(
    context: &mut Context,
    formal_args: &Vec<(Ident, TypedExpression)>,
    body: &TypedCodeBlock,
    actual_args: &Vec<Constant>,
) -> Option<Constant> {
    assert!(formal_args.len() == actual_args.len());

    let mut env = HashMap::<&Ident, Vec<&Constant>>::new();
    for (formal, actual) in std::iter::zip(formal_args, actual_args) {
        env.insert(&formal.0, vec![actual]);
    }
    // TODO: Maybe const_fold_typed_expr can take Vec<TypedAstNode> in the future.
    body.contents
        .last()
        .and_then(|first_expr| const_fold_typed_ast_node(context, &env, &first_expr))
}
