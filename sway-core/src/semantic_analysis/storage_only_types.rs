use sway_error::error::CompileError;
use sway_types::{Span, Spanned};

use crate::{
    declaration_engine::declaration_engine::*,
    error::*,
    language::ty,
    semantic_analysis::{
        TyAstNodeContent, TyCodeBlock, TyConstantDeclaration, TyFunctionDeclaration, TyImplTrait,
        TyIntrinsicFunctionKind, TyReassignment, TyStorageDeclaration, TyStructDeclaration,
    },
    type_system::*,
};

fn ast_node_validate(x: &TyAstNodeContent) -> CompileResult<()> {
    let errors: Vec<CompileError> = vec![];
    let warnings: Vec<CompileWarning> = vec![];
    match x {
        TyAstNodeContent::Expression(expr) | TyAstNodeContent::ImplicitReturnExpression(expr) => {
            expr_validate(expr)
        }
        TyAstNodeContent::Declaration(decl) => decl_validate(decl),
        TyAstNodeContent::SideEffect => ok((), warnings, errors),
    }
}

fn expr_validate(expr: &ty::TyExpression) -> CompileResult<()> {
    let mut errors: Vec<CompileError> = vec![];
    let mut warnings: Vec<CompileWarning> = vec![];
    match &expr.expression {
        ty::TyExpressionVariant::Literal(_)
        | ty::TyExpressionVariant::VariableExpression { .. }
        | ty::TyExpressionVariant::FunctionParameter
        | ty::TyExpressionVariant::AsmExpression { .. }
        | ty::TyExpressionVariant::StorageAccess(_)
        | ty::TyExpressionVariant::AbiName(_) => (),
        ty::TyExpressionVariant::FunctionApplication { arguments, .. } => {
            for f in arguments {
                check!(expr_validate(&f.1), continue, warnings, errors);
            }
        }
        ty::TyExpressionVariant::LazyOperator {
            lhs: expr1,
            rhs: expr2,
            ..
        }
        | ty::TyExpressionVariant::ArrayIndex {
            prefix: expr1,
            index: expr2,
        } => {
            check!(expr_validate(expr1), (), warnings, errors);
            check!(expr_validate(expr2), (), warnings, errors);
        }
        ty::TyExpressionVariant::IntrinsicFunction(TyIntrinsicFunctionKind {
            arguments: exprvec,
            ..
        })
        | ty::TyExpressionVariant::Tuple { fields: exprvec }
        | ty::TyExpressionVariant::Array { contents: exprvec } => {
            for f in exprvec {
                check!(expr_validate(f), continue, warnings, errors)
            }
        }
        ty::TyExpressionVariant::StructExpression { fields, .. } => {
            for f in fields {
                check!(expr_validate(&f.value), continue, warnings, errors);
            }
        }
        ty::TyExpressionVariant::CodeBlock(cb) => {
            check!(
                validate_decls_for_storage_only_types_in_codeblock(cb),
                (),
                warnings,
                errors
            );
        }
        ty::TyExpressionVariant::IfExp {
            condition,
            then,
            r#else,
        } => {
            check!(expr_validate(condition), (), warnings, errors);
            check!(expr_validate(then), (), warnings, errors);
            if let Some(r#else) = r#else {
                check!(expr_validate(r#else), (), warnings, errors);
            }
        }
        ty::TyExpressionVariant::StructFieldAccess { prefix: exp, .. }
        | ty::TyExpressionVariant::TupleElemAccess { prefix: exp, .. }
        | ty::TyExpressionVariant::AbiCast { address: exp, .. }
        | ty::TyExpressionVariant::EnumTag { exp }
        | ty::TyExpressionVariant::UnsafeDowncast { exp, .. } => {
            check!(expr_validate(exp), (), warnings, errors)
        }
        ty::TyExpressionVariant::EnumInstantiation { contents, .. } => {
            if let Some(f) = contents {
                check!(expr_validate(f), (), warnings, errors);
            }
        }
        ty::TyExpressionVariant::WhileLoop { condition, body } => {
            check!(expr_validate(condition), (), warnings, errors);
            check!(
                validate_decls_for_storage_only_types_in_codeblock(body),
                (),
                warnings,
                errors
            );
        }
        ty::TyExpressionVariant::Break => (),
        ty::TyExpressionVariant::Continue => (),
        ty::TyExpressionVariant::Reassignment(reassignment) => {
            let TyReassignment {
                lhs_base_name, rhs, ..
            } = &**reassignment;
            check!(
                check_type(rhs.return_type, lhs_base_name.span(), false),
                (),
                warnings,
                errors,
            );
            check!(expr_validate(rhs), (), warnings, errors)
        }
        ty::TyExpressionVariant::StorageReassignment(storage_reassignment) => {
            let span = storage_reassignment.span();
            let rhs = &storage_reassignment.rhs;
            check!(
                check_type(rhs.return_type, span, false),
                (),
                warnings,
                errors,
            );
            check!(expr_validate(rhs), (), warnings, errors)
        }
        ty::TyExpressionVariant::Return(stmt) => {
            check!(expr_validate(&stmt.expr), (), warnings, errors)
        }
    }
    ok((), warnings, errors)
}

fn check_type(ty: TypeId, span: Span, ignore_self: bool) -> CompileResult<()> {
    let mut warnings: Vec<CompileWarning> = vec![];
    let mut errors: Vec<CompileError> = vec![];

    let type_info = check!(
        CompileResult::from(to_typeinfo(ty, &span).map_err(CompileError::from)),
        TypeInfo::ErrorRecovery,
        warnings,
        errors
    );
    let nested_types = check!(
        type_info.clone().extract_nested_types(&span),
        vec![],
        warnings,
        errors
    );
    for ty in nested_types {
        if ignore_self && ty == type_info {
            continue;
        }
        if is_type_info_storage_only(&ty) {
            errors.push(CompileError::InvalidStorageOnlyTypeDecl {
                ty: ty.to_string(),
                span: span.clone(),
            });
        }
    }
    ok((), warnings, errors)
}

fn decl_validate(decl: &ty::TyDeclaration) -> CompileResult<()> {
    let mut warnings: Vec<CompileWarning> = vec![];
    let mut errors: Vec<CompileError> = vec![];
    match decl {
        ty::TyDeclaration::VariableDeclaration(decl) => {
            check!(
                check_type(decl.body.return_type, decl.name.span(), false),
                (),
                warnings,
                errors
            );
            check!(expr_validate(&decl.body), (), warnings, errors)
        }
        ty::TyDeclaration::ConstantDeclaration(decl_id) => {
            let TyConstantDeclaration {
                value: expr, name, ..
            } = check!(
                CompileResult::from(de_get_constant(decl_id.clone(), &decl_id.span())),
                return err(warnings, errors),
                warnings,
                errors
            );
            check!(
                check_type(expr.return_type, name.span(), false),
                (),
                warnings,
                errors
            );
            check!(expr_validate(&expr), (), warnings, errors)
        }
        ty::TyDeclaration::FunctionDeclaration(decl_id) => {
            let TyFunctionDeclaration {
                body, parameters, ..
            } = check!(
                CompileResult::from(de_get_function(decl_id.clone(), &decl.span())),
                return err(warnings, errors),
                warnings,
                errors
            );
            check!(
                validate_decls_for_storage_only_types_in_codeblock(&body),
                (),
                warnings,
                errors
            );
            for param in parameters {
                check!(
                    check_type(param.type_id, param.type_span.clone(), false),
                    continue,
                    warnings,
                    errors
                );
            }
        }
        ty::TyDeclaration::AbiDeclaration(_) | ty::TyDeclaration::TraitDeclaration(_) => {
            // These methods are not typed. They are however handled from ImplTrait.
        }
        ty::TyDeclaration::ImplTrait(decl_id) => {
            let TyImplTrait { methods, .. } = check!(
                CompileResult::from(de_get_impl_trait(decl_id.clone(), &decl_id.span())),
                return err(warnings, errors),
                warnings,
                errors
            );
            for method in methods {
                check!(
                    validate_decls_for_storage_only_types_in_codeblock(&method.body),
                    continue,
                    warnings,
                    errors
                )
            }
        }
        ty::TyDeclaration::StructDeclaration(decl_id) => {
            let TyStructDeclaration { fields, .. } = check!(
                CompileResult::from(de_get_struct(decl_id.clone(), &decl_id.span())),
                return err(warnings, errors),
                warnings,
                errors,
            );
            for field in fields {
                check!(
                    check_type(field.type_id, field.span.clone(), false),
                    continue,
                    warnings,
                    errors
                );
            }
        }
        ty::TyDeclaration::EnumDeclaration(decl_id) => {
            let ty::TyEnumDeclaration { variants, .. } = check!(
                CompileResult::from(de_get_enum(decl_id.clone(), &decl.span())),
                return err(warnings, errors),
                warnings,
                errors
            );
            for variant in variants {
                check!(
                    check_type(variant.type_id, variant.span.clone(), false),
                    continue,
                    warnings,
                    errors
                );
            }
        }
        ty::TyDeclaration::StorageDeclaration(decl_id) => {
            let TyStorageDeclaration { fields, .. } = check!(
                CompileResult::from(de_get_storage(decl_id.clone(), &decl.span())),
                return err(warnings, errors),
                warnings,
                errors
            );
            for field in fields {
                check!(
                    check_type(field.type_id, field.name.span().clone(), true),
                    continue,
                    warnings,
                    errors
                );
            }
        }
        ty::TyDeclaration::GenericTypeForFunctionScope { .. }
        | ty::TyDeclaration::ErrorRecovery => {}
    }
    ok((), warnings, errors)
}

pub fn validate_decls_for_storage_only_types_in_ast(ast_n: &TyAstNodeContent) -> CompileResult<()> {
    ast_node_validate(ast_n)
}

pub fn validate_decls_for_storage_only_types_in_codeblock(cb: &TyCodeBlock) -> CompileResult<()> {
    let mut warnings: Vec<CompileWarning> = vec![];
    let mut errors: Vec<CompileError> = vec![];
    for x in &cb.contents {
        check!(ast_node_validate(&x.content), continue, warnings, errors)
    }
    ok((), warnings, errors)
}
