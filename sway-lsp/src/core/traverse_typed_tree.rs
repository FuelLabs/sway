#![allow(dead_code)]

use crate::core::typed_token_type::{TokenMap, TokenType};
use sway_core::semantic_analysis::ast_node::{
    expression::{
        typed_expression::TypedExpression, typed_expression_variant::TypedExpressionVariant,
    },
    while_loop::TypedWhileLoop,
    {TypedAstNode, TypedAstNodeContent, TypedDeclaration},
};
use sway_core::type_engine::TypeId;
use sway_types::{ident::Ident, span::Span};

pub fn traverse_node(node: &TypedAstNode, tokens: &mut TokenMap) {
    match &node.content {
        TypedAstNodeContent::ReturnStatement(return_statement) => {
            handle_expression(&return_statement.expr, tokens)
        }
        TypedAstNodeContent::Declaration(declaration) => handle_declaration(declaration, tokens),
        TypedAstNodeContent::Expression(expression) => handle_expression(expression, tokens),
        TypedAstNodeContent::ImplicitReturnExpression(expression) => {
            handle_expression(expression, tokens)
        }
        TypedAstNodeContent::WhileLoop(while_loop) => handle_while_loop(while_loop, tokens),
        TypedAstNodeContent::SideEffect => (),
    };
}

// We need to do this work around as the custom PartialEq for Ident impl
// only checks for the string, not the span.
fn to_ident_key(ident: &Ident) -> (Ident, Span) {
    (ident.clone(), ident.span().clone())
}

fn handle_declaration(declaration: &TypedDeclaration, tokens: &mut TokenMap) {
    match declaration {
        TypedDeclaration::VariableDeclaration(variable) => {
            tokens.insert(
                to_ident_key(&variable.name),
                TokenType::TypedDeclaration(declaration.clone()),
            );
            handle_expression(&variable.body, tokens);
        }
        TypedDeclaration::ConstantDeclaration(const_decl) => {
            tokens.insert(
                to_ident_key(&const_decl.name),
                TokenType::TypedDeclaration(declaration.clone()),
            );
            handle_expression(&const_decl.value, tokens);
        }
        TypedDeclaration::FunctionDeclaration(func) => {
            tokens.insert(
                to_ident_key(&func.name),
                TokenType::TypedFunctionDeclaration(func.clone()),
            );
            for node in &func.body.contents {
                traverse_node(node, tokens);
            }
            for parameter in &func.parameters {
                tokens.insert(
                    to_ident_key(&parameter.name),
                    TokenType::TypedFunctionParameter(parameter.clone()),
                );
            }
        }
        TypedDeclaration::TraitDeclaration(trait_decl) => {
            tokens.insert(
                to_ident_key(&trait_decl.name),
                TokenType::TypedDeclaration(declaration.clone()),
            );
            for train_fn in &trait_decl.interface_surface {
                tokens.insert(
                    to_ident_key(&train_fn.name),
                    TokenType::TypedTraitFn(train_fn.clone()),
                );
            }
        }
        TypedDeclaration::StructDeclaration(struct_dec) => {
            tokens.insert(
                to_ident_key(&struct_dec.name),
                TokenType::TypedDeclaration(declaration.clone()),
            );
            for field in &struct_dec.fields {
                tokens.insert(
                    to_ident_key(&field.name),
                    TokenType::TypedStructField(field.clone()),
                );
            }
        }
        TypedDeclaration::EnumDeclaration(enum_decl) => {
            tokens.insert(
                to_ident_key(&enum_decl.name),
                TokenType::TypedDeclaration(declaration.clone()),
            );
            for variant in &enum_decl.variants {
                tokens.insert(
                    to_ident_key(&variant.name),
                    TokenType::TypedEnumVariant(variant.clone()),
                );
            }
        }
        TypedDeclaration::Reassignment(reassignment) => {
            handle_expression(&reassignment.rhs, tokens);
            for lhs in &reassignment.lhs {
                tokens.insert(
                    to_ident_key(&lhs.name),
                    TokenType::ReassignmentLhs(lhs.clone()),
                );
            }
        }
        TypedDeclaration::ImplTrait {
            trait_name,
            methods,
            ..
        } => {
            for ident in &trait_name.prefixes {
                tokens.insert(
                    to_ident_key(ident),
                    TokenType::TypedDeclaration(declaration.clone()),
                );
            }
            // This is reporting the train name as r#Self and not the actual name
            // Also the span is referencing the declerations span.
            //tokens.insert(to_ident_key(&trait_name.suffix), TokenType::TypedDeclaration(declaration.clone()));

            for method in methods {
                tokens.insert(
                    to_ident_key(&method.name),
                    TokenType::TypedFunctionDeclaration(method.clone()),
                );
                for node in &method.body.contents {
                    traverse_node(node, tokens);
                }
                for paramater in &method.parameters {
                    tokens.insert(
                        to_ident_key(&paramater.name),
                        TokenType::TypedFunctionParameter(paramater.clone()),
                    );
                }
            }
        }
        TypedDeclaration::AbiDeclaration(abi_decl) => {
            tokens.insert(
                to_ident_key(&abi_decl.name),
                TokenType::TypedDeclaration(declaration.clone()),
            );
            for trait_fn in &abi_decl.interface_surface {
                tokens.insert(
                    to_ident_key(&trait_fn.name),
                    TokenType::TypedTraitFn(trait_fn.clone()),
                );
            }
        }
        TypedDeclaration::GenericTypeForFunctionScope { name } => {
            tokens.insert(
                to_ident_key(name),
                TokenType::TypedDeclaration(declaration.clone()),
            );
        }
        TypedDeclaration::ErrorRecovery => {}
        TypedDeclaration::StorageDeclaration(storage_decl) => {
            for field in &storage_decl.fields {
                tokens.insert(
                    to_ident_key(&field.name),
                    TokenType::TypedStorageField(field.clone()),
                );
            }
        }
        TypedDeclaration::StorageReassignment(storage_reassignment) => {
            for field in &storage_reassignment.fields {
                tokens.insert(
                    to_ident_key(&field.name),
                    TokenType::TypeCheckedStorageReassignDescriptor(field.clone()),
                );
            }
            handle_expression(&storage_reassignment.rhs, tokens);
        }
    }
}

fn handle_expression(expression: &TypedExpression, tokens: &mut TokenMap) {
    match &expression.expression {
        TypedExpressionVariant::Literal { .. } => {}
        TypedExpressionVariant::FunctionApplication {
            call_path,
            contract_call_params,
            arguments,
            function_body,
            ..
        } => {
            for ident in &call_path.prefixes {
                tokens.insert(
                    to_ident_key(ident),
                    TokenType::TypedExpression(expression.clone()),
                );
            }
            tokens.insert(
                to_ident_key(&call_path.suffix),
                TokenType::TypedExpression(expression.clone()),
            );

            for exp in contract_call_params.values() {
                handle_expression(exp, tokens);
            }

            for (ident, exp) in arguments {
                tokens.insert(to_ident_key(ident), TokenType::TypedExpression(exp.clone()));
                handle_expression(exp, tokens);
            }

            for node in &function_body.contents {
                traverse_node(node, tokens);
            }
        }
        TypedExpressionVariant::LazyOperator { lhs, rhs, .. } => {
            handle_expression(lhs, tokens);
            handle_expression(rhs, tokens);
        }
        TypedExpressionVariant::VariableExpression { ref name } => {
            tokens.insert(
                to_ident_key(name),
                TokenType::TypedExpression(expression.clone()),
            );
        }
        TypedExpressionVariant::Tuple { fields } => {
            for exp in fields {
                handle_expression(exp, tokens);
            }
        }
        TypedExpressionVariant::Array { contents } => {
            for exp in contents {
                handle_expression(exp, tokens);
            }
        }
        TypedExpressionVariant::ArrayIndex { prefix, index } => {
            handle_expression(prefix, tokens);
            handle_expression(index, tokens);
        }
        TypedExpressionVariant::StructExpression {
            ref struct_name,
            ref fields,
        } => {
            tokens.insert(
                to_ident_key(struct_name),
                TokenType::TypedExpression(expression.clone()),
            );
            for field in fields {
                tokens.insert(
                    to_ident_key(&field.name),
                    TokenType::TypedExpression(field.value.clone()),
                );
                handle_expression(&field.value, tokens);
            }
        }
        TypedExpressionVariant::CodeBlock(code_block) => {
            for node in &code_block.contents {
                traverse_node(node, tokens);
            }
        }
        TypedExpressionVariant::FunctionParameter { .. } => {}
        TypedExpressionVariant::IfExp {
            condition,
            then,
            r#else,
        } => {
            handle_expression(condition, tokens);
            handle_expression(then, tokens);
            if let Some(r#else) = r#else {
                handle_expression(r#else, tokens);
            }
        }
        TypedExpressionVariant::AsmExpression { .. } => {}
        TypedExpressionVariant::StructFieldAccess {
            prefix,
            field_to_access,
            ..
        } => {
            handle_expression(prefix, tokens);
            tokens.insert(
                to_ident_key(&field_to_access.name),
                TokenType::TypedExpression(expression.clone()),
            );
        }
        TypedExpressionVariant::TupleElemAccess { prefix, .. } => {
            handle_expression(prefix, tokens);
        }
        TypedExpressionVariant::EnumInstantiation { .. } => {}
        TypedExpressionVariant::AbiCast {
            abi_name, address, ..
        } => {
            for ident in &abi_name.prefixes {
                tokens.insert(
                    to_ident_key(ident),
                    TokenType::TypedExpression(expression.clone()),
                );
            }
            tokens.insert(
                to_ident_key(&abi_name.suffix),
                TokenType::TypedExpression(expression.clone()),
            );
            handle_expression(address, tokens);
        }
        TypedExpressionVariant::StorageAccess(storage_access) => {
            for field in &storage_access.fields {
                tokens.insert(
                    to_ident_key(&field.name),
                    TokenType::TypedExpression(expression.clone()),
                );
            }
        }
        TypedExpressionVariant::TypeProperty { .. } => {}
        TypedExpressionVariant::GenerateUid { .. } => {}
        TypedExpressionVariant::SizeOfValue { expr } => {
            handle_expression(expr, tokens);
        }
        TypedExpressionVariant::AbiName { .. } => {}
        TypedExpressionVariant::EnumTag { exp } => {
            handle_expression(exp, tokens);
        }
        TypedExpressionVariant::UnsafeDowncast { exp, variant } => {
            handle_expression(exp, tokens);
            tokens.insert(
                to_ident_key(&variant.name),
                TokenType::TypedExpression(expression.clone()),
            );
        }
    }
}

fn handle_while_loop(while_loop: &TypedWhileLoop, tokens: &mut TokenMap) {
    handle_expression(&while_loop.condition, tokens);
    for node in &while_loop.body.contents {
        traverse_node(node, tokens);
    }
}

pub fn get_type_id(token: &TokenType) -> Option<TypeId> {
    match token {
        TokenType::TypedDeclaration(dec) => match dec {
            TypedDeclaration::VariableDeclaration(var_decl) => Some(var_decl.type_ascription),
            TypedDeclaration::ConstantDeclaration(const_decl) => Some(const_decl.value.return_type),
            _ => None,
        },
        TokenType::TypedExpression(exp) => Some(exp.return_type),
        TokenType::TypedFunctionParameter(func_param) => Some(func_param.r#type),
        TokenType::TypedStructField(struct_field) => Some(struct_field.r#type),
        TokenType::TypedEnumVariant(enum_var) => Some(enum_var.r#type),
        TokenType::TypedTraitFn(trait_fn) => Some(trait_fn.return_type),
        TokenType::TypedStorageField(storage_field) => Some(storage_field.r#type),
        TokenType::TypeCheckedStorageReassignDescriptor(storage_desc) => Some(storage_desc.r#type),
        TokenType::ReassignmentLhs(lhs) => Some(lhs.r#type),
        _ => None,
    }
}
