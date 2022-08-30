use sway_types::{Span, Spanned};

use crate::declaration_engine::declaration_engine::DeclarationEngine;
use crate::type_system::{is_type_info_storage_only, resolve_type, TypeId};
use crate::types::ToCompileWrapper;
use crate::{error::ok, semantic_analysis, CompileError, CompileResult, CompileWarning};
use crate::{TypedDeclaration, TypedFunctionDeclaration};

use crate::semantic_analysis::{
    TypedAbiDeclaration, TypedAstNodeContent, TypedExpression, TypedExpressionVariant,
    TypedIntrinsicFunctionKind, TypedReassignment, TypedReturnStatement,
};

use super::{
    TypedEnumDeclaration, TypedImplTrait, TypedStorageDeclaration, TypedStructDeclaration,
    TypedTraitDeclaration,
};

fn ast_node_validate(x: &TypedAstNodeContent, de: &DeclarationEngine) -> CompileResult<()> {
    let errors: Vec<CompileError> = vec![];
    let warnings: Vec<CompileWarning> = vec![];
    match x {
        TypedAstNodeContent::ReturnStatement(TypedReturnStatement { expr })
        | TypedAstNodeContent::Expression(expr)
        | TypedAstNodeContent::ImplicitReturnExpression(expr) => expr_validate(expr, de),
        TypedAstNodeContent::Declaration(decl) => decl_validate(decl, de),
        TypedAstNodeContent::SideEffect => ok((), warnings, errors),
    }
}

fn expr_validate(expr: &TypedExpression, de: &DeclarationEngine) -> CompileResult<()> {
    let mut errors: Vec<CompileError> = vec![];
    let mut warnings: Vec<CompileWarning> = vec![];
    match &expr.expression {
        TypedExpressionVariant::Literal(_)
        | TypedExpressionVariant::VariableExpression { .. }
        | TypedExpressionVariant::FunctionParameter
        | TypedExpressionVariant::AsmExpression { .. }
        | TypedExpressionVariant::StorageAccess(_)
        | TypedExpressionVariant::AbiName(_) => (),
        TypedExpressionVariant::FunctionApplication { arguments, .. } => {
            for f in arguments {
                check!(expr_validate(&f.1, de), continue, warnings, errors);
            }
        }
        TypedExpressionVariant::LazyOperator {
            lhs: expr1,
            rhs: expr2,
            ..
        }
        | TypedExpressionVariant::ArrayIndex {
            prefix: expr1,
            index: expr2,
        } => {
            check!(expr_validate(expr1, de), (), warnings, errors);
            check!(expr_validate(expr2, de), (), warnings, errors);
        }
        TypedExpressionVariant::IntrinsicFunction(TypedIntrinsicFunctionKind {
            arguments: exprvec,
            ..
        })
        | TypedExpressionVariant::Tuple { fields: exprvec }
        | TypedExpressionVariant::Array { contents: exprvec } => {
            for f in exprvec {
                check!(expr_validate(f, de), continue, warnings, errors)
            }
        }
        TypedExpressionVariant::StructExpression { fields, .. } => {
            for f in fields {
                check!(expr_validate(&f.value, de), continue, warnings, errors);
            }
        }
        TypedExpressionVariant::CodeBlock(cb) => {
            check!(
                validate_decls_for_storage_only_types_in_codeblock(cb, de),
                (),
                warnings,
                errors
            );
        }
        TypedExpressionVariant::IfExp {
            condition,
            then,
            r#else,
        } => {
            check!(expr_validate(condition, de), (), warnings, errors);
            check!(expr_validate(then, de), (), warnings, errors);
            if let Some(r#else) = r#else {
                check!(expr_validate(r#else, de), (), warnings, errors);
            }
        }
        TypedExpressionVariant::StructFieldAccess { prefix: exp, .. }
        | TypedExpressionVariant::TupleElemAccess { prefix: exp, .. }
        | TypedExpressionVariant::AbiCast { address: exp, .. }
        | TypedExpressionVariant::EnumTag { exp }
        | TypedExpressionVariant::UnsafeDowncast { exp, .. } => {
            check!(expr_validate(exp, de), (), warnings, errors)
        }
        TypedExpressionVariant::EnumInstantiation { contents, .. } => {
            if let Some(f) = contents {
                check!(expr_validate(f, de), (), warnings, errors);
            }
        }
        TypedExpressionVariant::WhileLoop { condition, body } => {
            check!(expr_validate(condition, de), (), warnings, errors);
            check!(
                validate_decls_for_storage_only_types_in_codeblock(body, de),
                (),
                warnings,
                errors
            );
        }
        TypedExpressionVariant::Break => (),
        TypedExpressionVariant::Continue => (),
        TypedExpressionVariant::Reassignment(reassignment) => {
            let TypedReassignment {
                lhs_base_name, rhs, ..
            } = &**reassignment;
            check!(
                check_type(rhs.return_type, lhs_base_name.span(), false, de),
                (),
                warnings,
                errors,
            );
            check!(expr_validate(rhs, de), (), warnings, errors)
        }
        TypedExpressionVariant::StorageReassignment(storage_reassignment) => {
            let span = storage_reassignment.span();
            let rhs = &storage_reassignment.rhs;
            check!(
                check_type(rhs.return_type, span, false, de),
                (),
                warnings,
                errors,
            );
            check!(expr_validate(rhs, de), (), warnings, errors)
        }
    }
    ok((), warnings, errors)
}

fn check_type(
    ty: TypeId,
    span: Span,
    ignore_self: bool,
    de: &DeclarationEngine,
) -> CompileResult<()> {
    let mut warnings: Vec<CompileWarning> = vec![];
    let mut errors: Vec<CompileError> = vec![];

    let ti = resolve_type(ty, &span).unwrap();
    let nested_types = check!(
        ti.clone().extract_nested_types(&span),
        vec![],
        warnings,
        errors
    );
    for ty in nested_types {
        if ignore_self && ty.wrap_ref(de) == ti.wrap_ref(de) {
            continue;
        }
        if is_type_info_storage_only(&ty, de) {
            errors.push(CompileError::InvalidStorageOnlyTypeDecl {
                ty: ty.to_string(),
                span: span.clone(),
            });
        }
    }
    ok((), warnings, errors)
}

fn decl_validate(decl: &TypedDeclaration, de: &DeclarationEngine) -> CompileResult<()> {
    let mut warnings: Vec<CompileWarning> = vec![];
    let mut errors: Vec<CompileError> = vec![];
    match decl {
        TypedDeclaration::VariableDeclaration(semantic_analysis::TypedVariableDeclaration {
            body: expr,
            name,
            ..
        })
        | TypedDeclaration::ConstantDeclaration(semantic_analysis::TypedConstantDeclaration {
            value: expr,
            name,
            ..
        }) => {
            check!(
                check_type(expr.return_type, name.span(), false, de),
                (),
                warnings,
                errors
            );
            check!(expr_validate(expr, de), (), warnings, errors)
        }
        TypedDeclaration::FunctionDeclaration(TypedFunctionDeclaration {
            body,
            parameters,
            ..
        }) => {
            check!(
                validate_decls_for_storage_only_types_in_codeblock(body, de),
                (),
                warnings,
                errors
            );
            for param in parameters {
                check!(
                    check_type(param.type_id, param.type_span.clone(), false, de),
                    continue,
                    warnings,
                    errors
                );
            }
        }
        TypedDeclaration::AbiDeclaration(TypedAbiDeclaration { methods: _, .. })
        | TypedDeclaration::TraitDeclaration(TypedTraitDeclaration { methods: _, .. }) => {
            // These methods are not typed. They are however handled from ImplTrait.
        }
        TypedDeclaration::ImplTrait(TypedImplTrait { methods, .. }) => {
            for method in methods {
                check!(
                    validate_decls_for_storage_only_types_in_codeblock(&method.body, de),
                    continue,
                    warnings,
                    errors
                )
            }
        }
        TypedDeclaration::StructDeclaration(TypedStructDeclaration { fields, .. }) => {
            for field in fields {
                check!(
                    check_type(field.type_id, field.span.clone(), false, de),
                    continue,
                    warnings,
                    errors
                );
            }
        }
        TypedDeclaration::EnumDeclaration(TypedEnumDeclaration { variants, .. }) => {
            for variant in variants {
                check!(
                    check_type(variant.type_id, variant.span.clone(), false, de),
                    continue,
                    warnings,
                    errors
                );
            }
        }
        TypedDeclaration::StorageDeclaration(TypedStorageDeclaration { fields, .. }) => {
            for field in fields {
                check!(
                    check_type(field.type_id, field.name.span().clone(), true, de),
                    continue,
                    warnings,
                    errors
                );
            }
        }
        TypedDeclaration::GenericTypeForFunctionScope { .. } | TypedDeclaration::ErrorRecovery => {}
    }
    ok((), warnings, errors)
}

pub fn validate_decls_for_storage_only_types_in_ast(
    ast_n: &TypedAstNodeContent,
    de: &DeclarationEngine,
) -> CompileResult<()> {
    ast_node_validate(ast_n, de)
}

pub fn validate_decls_for_storage_only_types_in_codeblock(
    cb: &semantic_analysis::TypedCodeBlock,
    de: &DeclarationEngine,
) -> CompileResult<()> {
    let mut warnings: Vec<CompileWarning> = vec![];
    let mut errors: Vec<CompileError> = vec![];
    for x in &cb.contents {
        check!(
            ast_node_validate(&x.content, de),
            continue,
            warnings,
            errors
        )
    }
    ok((), warnings, errors)
}
