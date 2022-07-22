#![allow(dead_code)]

use crate::{
    core::token::{TokenMap, TypedAstToken},
    utils::token::to_ident_key,
};
use sway_core::semantic_analysis::ast_node::{
    expression::{
        typed_expression::TypedExpression, typed_expression_variant::TypedExpressionVariant,
        TypedIntrinsicFunctionKind,
    },
    while_loop::TypedWhileLoop,
    TypedImplTrait, {TypedAstNode, TypedAstNodeContent, TypedDeclaration},
};
use sway_types::ident::Ident;

pub fn traverse_node(node: &TypedAstNode, tokens: &TokenMap) {
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

fn handle_declaration(declaration: &TypedDeclaration, tokens: &TokenMap) {
    match declaration {
        TypedDeclaration::VariableDeclaration(variable) => {
            if let Some(mut token) = tokens.get_mut(&to_ident_key(&variable.name)) {
                token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
            }
            handle_expression(&variable.body, tokens);
        }
        TypedDeclaration::ConstantDeclaration(const_decl) => {
            if let Some(mut token) = tokens.get_mut(&to_ident_key(&const_decl.name)) {
                token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
            }
            handle_expression(&const_decl.value, tokens);
        }
        TypedDeclaration::FunctionDeclaration(func) => {
            if let Some(mut token) = tokens.get_mut(&to_ident_key(&func.name)) {
                token.typed = Some(TypedAstToken::TypedFunctionDeclaration(func.clone()));
            }

            for node in &func.body.contents {
                traverse_node(node, tokens);
            }
            for parameter in &func.parameters {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&parameter.name)) {
                    token.typed = Some(TypedAstToken::TypedFunctionParameter(parameter.clone()));
                }
            }
        }
        TypedDeclaration::TraitDeclaration(trait_decl) => {
            if let Some(mut token) = tokens.get_mut(&to_ident_key(&trait_decl.name)) {
                token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
            }

            for train_fn in &trait_decl.interface_surface {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&train_fn.name)) {
                    token.typed = Some(TypedAstToken::TypedTraitFn(train_fn.clone()));
                }
            }
        }
        TypedDeclaration::StructDeclaration(struct_dec) => {
            if let Some(mut token) = tokens.get_mut(&to_ident_key(&struct_dec.name)) {
                token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
            }

            for field in &struct_dec.fields {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&field.name)) {
                    token.typed = Some(TypedAstToken::TypedStructField(field.clone()));
                }
            }
        }
        TypedDeclaration::EnumDeclaration(enum_decl) => {
            if let Some(mut token) = tokens.get_mut(&to_ident_key(&enum_decl.name)) {
                token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
            }

            for variant in &enum_decl.variants {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&variant.name)) {
                    token.typed = Some(TypedAstToken::TypedEnumVariant(variant.clone()));
                }
            }
        }
        TypedDeclaration::Reassignment(reassignment) => {
            handle_expression(&reassignment.rhs, tokens);

            if let Some(mut token) = tokens.get_mut(&to_ident_key(&reassignment.lhs_base_name)) {
                token.typed = Some(TypedAstToken::TypedReassignment(reassignment.clone()));
            }
        }
        TypedDeclaration::ImplTrait(TypedImplTrait {
            trait_name,
            methods,
            ..
        }) => {
            for ident in &trait_name.prefixes {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(ident)) {
                    token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
                }
            }

            if let Some(mut token) = tokens.get_mut(&to_ident_key(&trait_name.suffix)) {
                token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
            }

            for method in methods {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&method.name)) {
                    token.typed = Some(TypedAstToken::TypedFunctionDeclaration(method.clone()));
                }

                for node in &method.body.contents {
                    traverse_node(node, tokens);
                }
                for paramater in &method.parameters {
                    if let Some(mut token) = tokens.get_mut(&to_ident_key(&paramater.name)) {
                        token.typed =
                            Some(TypedAstToken::TypedFunctionParameter(paramater.clone()));
                    }
                }

                let return_type_ident = Ident::new(method.return_type_span.clone());
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&return_type_ident)) {
                    token.typed = Some(TypedAstToken::TypedFunctionDeclaration(method.clone()));
                }
            }
        }
        TypedDeclaration::AbiDeclaration(abi_decl) => {
            if let Some(mut token) = tokens.get_mut(&to_ident_key(&abi_decl.name)) {
                token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
            }

            for trait_fn in &abi_decl.interface_surface {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&trait_fn.name)) {
                    token.typed = Some(TypedAstToken::TypedTraitFn(trait_fn.clone()));
                }
            }
        }
        TypedDeclaration::GenericTypeForFunctionScope { name, .. } => {
            if let Some(mut token) = tokens.get_mut(&to_ident_key(name)) {
                token.typed = Some(TypedAstToken::TypedDeclaration(declaration.clone()));
            }
        }
        TypedDeclaration::ErrorRecovery => {}
        TypedDeclaration::StorageDeclaration(storage_decl) => {
            for field in &storage_decl.fields {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&field.name)) {
                    token.typed = Some(TypedAstToken::TypedStorageField(field.clone()));
                }
            }
        }
        TypedDeclaration::StorageReassignment(storage_reassignment) => {
            for field in &storage_reassignment.fields {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&field.name)) {
                    token.typed = Some(TypedAstToken::TypeCheckedStorageReassignDescriptor(
                        field.clone(),
                    ));
                }
            }
            handle_expression(&storage_reassignment.rhs, tokens);
        }
        TypedDeclaration::Break { .. } => {}
        TypedDeclaration::Continue { .. } => {}
    }
}

fn handle_expression(expression: &TypedExpression, tokens: &TokenMap) {
    match &expression.expression {
        TypedExpressionVariant::Literal { .. } => {}
        TypedExpressionVariant::FunctionApplication {
            call_path,
            contract_call_params,
            arguments,
            function_decl,
            ..
        } => {
            for ident in &call_path.prefixes {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(ident)) {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                }
            }

            if let Some(mut token) = tokens.get_mut(&to_ident_key(&call_path.suffix)) {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
            }

            for exp in contract_call_params.values() {
                handle_expression(exp, tokens);
            }

            for (ident, exp) in arguments {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(ident)) {
                    token.typed = Some(TypedAstToken::TypedExpression(exp.clone()));
                }
                handle_expression(exp, tokens);
            }

            for node in &function_decl.body.contents {
                traverse_node(node, tokens);
            }
        }
        TypedExpressionVariant::LazyOperator { lhs, rhs, .. } => {
            handle_expression(lhs, tokens);
            handle_expression(rhs, tokens);
        }
        TypedExpressionVariant::VariableExpression { ref name } => {
            if let Some(mut token) = tokens.get_mut(&to_ident_key(name)) {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
            }
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
            if let Some(mut token) = tokens.get_mut(&to_ident_key(struct_name)) {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
            }

            for field in fields {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&field.name)) {
                    token.typed = Some(TypedAstToken::TypedExpression(field.value.clone()));
                }
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

            if let Some(mut token) = tokens.get_mut(&to_ident_key(&field_to_access.name)) {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
            }
        }
        TypedExpressionVariant::TupleElemAccess { prefix, .. } => {
            handle_expression(prefix, tokens);
        }
        TypedExpressionVariant::EnumInstantiation { .. } => {}
        TypedExpressionVariant::AbiCast {
            abi_name, address, ..
        } => {
            for ident in &abi_name.prefixes {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(ident)) {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                }
            }

            if let Some(mut token) = tokens.get_mut(&to_ident_key(&abi_name.suffix)) {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
            }

            handle_expression(address, tokens);
        }
        TypedExpressionVariant::StorageAccess(storage_access) => {
            for field in &storage_access.fields {
                if let Some(mut token) = tokens.get_mut(&to_ident_key(&field.name)) {
                    token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
                }
            }
        }
        TypedExpressionVariant::IntrinsicFunction(kind) => {
            handle_intrinsic_function(kind, tokens);
        }
        TypedExpressionVariant::AbiName { .. } => {}
        TypedExpressionVariant::EnumTag { exp } => {
            handle_expression(exp, tokens);
        }
        TypedExpressionVariant::UnsafeDowncast { exp, variant } => {
            handle_expression(exp, tokens);
            if let Some(mut token) = tokens.get_mut(&to_ident_key(&variant.name)) {
                token.typed = Some(TypedAstToken::TypedExpression(expression.clone()));
            }
        }
    }
}

fn handle_intrinsic_function(
    TypedIntrinsicFunctionKind { arguments, .. }: &TypedIntrinsicFunctionKind,
    tokens: &TokenMap,
) {
    for arg in arguments {
        handle_expression(arg, tokens);
    }
}

fn handle_while_loop(while_loop: &TypedWhileLoop, tokens: &TokenMap) {
    handle_expression(&while_loop.condition, tokens);
    for node in &while_loop.body.contents {
        traverse_node(node, tokens);
    }
}
