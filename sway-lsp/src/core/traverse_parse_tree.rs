#![allow(dead_code)]

use crate::{
    core::token::{AstToken, TokenMap, TokenType},
    utils::token::{desugared_op, to_ident_key},
};
use sway_core::{
    constants::{MATCH_RETURN_VAR_NAME_PREFIX, TUPLE_NAME_PREFIX},
    parse_tree::MethodName,
    AstNode, AstNodeContent, Declaration, Expression, FunctionDeclaration, ReassignmentTarget,
    TypeInfo, WhileLoop,
};
use sway_types::Ident;

pub fn traverse_node(node: &AstNode, tokens: &mut TokenMap) {
    match &node.content {
        AstNodeContent::Declaration(declaration) => handle_declaration(declaration, tokens),
        AstNodeContent::Expression(expression) => handle_expression(expression, tokens),
        AstNodeContent::ImplicitReturnExpression(expression) => {
            handle_expression(expression, tokens)
        }
        AstNodeContent::ReturnStatement(return_statement) => {
            handle_expression(&return_statement.expr, tokens)
        }
        AstNodeContent::WhileLoop(while_loop) => handle_while_loop(while_loop, tokens),
        // TODO
        // handle other content types
        _ => {}
    };
}

fn handle_function_declation(func: &FunctionDeclaration, tokens: &mut TokenMap) {
    tokens.insert(
        to_ident_key(&func.name),
        TokenType::from_parsed(AstToken::FunctionDeclaration(func.clone())),
    );
    for node in &func.body.contents {
        traverse_node(node, tokens);
    }
    for parameter in &func.parameters {
        tokens.insert(
            to_ident_key(&parameter.name),
            TokenType::from_parsed(AstToken::FunctionParameter(parameter.clone())),
        );
    }

    handle_custom_type(&func.return_type, tokens);
}

fn handle_custom_type(_type_info: &TypeInfo, _tokens: &mut TokenMap) {
    // TODO: Not obvious how to handle this now with the new types
    // I've opened an issue to track this here https://github.com/FuelLabs/sway/issues/2274
}

fn handle_declaration(declaration: &Declaration, tokens: &mut TokenMap) {
    match declaration {
        Declaration::VariableDeclaration(variable) => {
            // Don't collect tokens if the ident's name contains __tuple_ || __match_return_var_name_
            // The individual elements are handled in the subsequent VariableDeclaration's
            if !variable.name.as_str().contains(TUPLE_NAME_PREFIX)
                && !variable
                    .name
                    .as_str()
                    .contains(MATCH_RETURN_VAR_NAME_PREFIX)
            {
                tokens.insert(
                    to_ident_key(&variable.name),
                    TokenType::from_parsed(AstToken::Declaration(declaration.clone())),
                );
            }
            handle_expression(&variable.body, tokens);
        }
        Declaration::FunctionDeclaration(func) => {
            handle_function_declation(func, tokens);
        }
        Declaration::TraitDeclaration(trait_decl) => {
            tokens.insert(
                to_ident_key(&trait_decl.name),
                TokenType::from_parsed(AstToken::Declaration(declaration.clone())),
            );

            for trait_fn in &trait_decl.interface_surface {
                tokens.insert(
                    to_ident_key(&trait_fn.name),
                    TokenType::from_parsed(AstToken::TraitFn(trait_fn.clone())),
                );
            }

            for func_dec in &trait_decl.methods {
                handle_function_declation(func_dec, tokens);
            }
        }
        Declaration::StructDeclaration(struct_dec) => {
            tokens.insert(
                to_ident_key(&struct_dec.name),
                TokenType::from_parsed(AstToken::Declaration(declaration.clone())),
            );
            for field in &struct_dec.fields {
                tokens.insert(
                    to_ident_key(&field.name),
                    TokenType::from_parsed(AstToken::StructField(field.clone())),
                );
            }
        }
        Declaration::EnumDeclaration(enum_decl) => {
            tokens.insert(
                to_ident_key(&enum_decl.name),
                TokenType::from_parsed(AstToken::Declaration(declaration.clone())),
            );
            for variant in &enum_decl.variants {
                tokens.insert(
                    to_ident_key(&variant.name),
                    TokenType::from_parsed(AstToken::EnumVariant(variant.clone())),
                );
            }
        }
        Declaration::Reassignment(reassignment) => {
            handle_expression(&reassignment.rhs, tokens);

            match &reassignment.lhs {
                ReassignmentTarget::VariableExpression(exp) => {
                    handle_expression(exp, tokens);
                }
                ReassignmentTarget::StorageField(idents) => {
                    for ident in idents {
                        tokens.insert(
                            to_ident_key(ident),
                            TokenType::from_parsed(AstToken::Reassignment(reassignment.clone())),
                        );
                    }
                }
            }
        }
        Declaration::ImplTrait(impl_trait) => {
            for ident in &impl_trait.trait_name.prefixes {
                tokens.insert(
                    to_ident_key(ident),
                    TokenType::from_parsed(AstToken::Declaration(declaration.clone())),
                );
            }

            tokens.insert(
                to_ident_key(&impl_trait.trait_name.suffix),
                TokenType::from_parsed(AstToken::Declaration(declaration.clone())),
            );

            for func_dec in &impl_trait.functions {
                handle_function_declation(func_dec, tokens);
            }
        }
        Declaration::ImplSelf(impl_self) => {
            handle_custom_type(&impl_self.type_implementing_for, tokens);

            for func_dec in &impl_self.functions {
                handle_function_declation(func_dec, tokens);
            }
        }
        Declaration::AbiDeclaration(abi_decl) => {
            tokens.insert(
                to_ident_key(&abi_decl.name),
                TokenType::from_parsed(AstToken::Declaration(declaration.clone())),
            );
            for trait_fn in &abi_decl.interface_surface {
                tokens.insert(
                    to_ident_key(&trait_fn.name),
                    TokenType::from_parsed(AstToken::TraitFn(trait_fn.clone())),
                );

                for param in &trait_fn.parameters {
                    tokens.insert(
                        to_ident_key(&param.name),
                        TokenType::from_parsed(AstToken::FunctionParameter(param.clone())),
                    );
                }
                // TODO: handle return type
            }
        }
        Declaration::ConstantDeclaration(const_decl) => {
            tokens.insert(
                to_ident_key(&const_decl.name),
                TokenType::from_parsed(AstToken::Declaration(declaration.clone())),
            );
            handle_expression(&const_decl.value, tokens);
        }
        Declaration::StorageDeclaration(storage_decl) => {
            for field in &storage_decl.fields {
                tokens.insert(
                    to_ident_key(&field.name),
                    TokenType::from_parsed(AstToken::StorageField(field.clone())),
                );
            }
        }
        // TODO: collect these tokens as keywords once the compiler returns the span
        Declaration::Break { .. } | Declaration::Continue { .. } => {}
    }
}

fn handle_expression(expression: &Expression, tokens: &mut TokenMap) {
    match expression {
        Expression::Literal { .. } => {}
        Expression::FunctionApplication {
            call_path_binding,
            arguments,
            ..
        } => {
            // Don't collect applications of desugared operators due to mismatched ident lengths.
            if !desugared_op(&call_path_binding.inner.prefixes) {
                for ident in &call_path_binding.inner.prefixes {
                    tokens.insert(
                        to_ident_key(ident),
                        TokenType::from_parsed(AstToken::Expression(expression.clone())),
                    );
                }
                tokens.insert(
                    to_ident_key(&call_path_binding.inner.suffix),
                    TokenType::from_parsed(AstToken::Expression(expression.clone())),
                );
            }

            for exp in arguments {
                handle_expression(exp, tokens);
            }
        }
        Expression::LazyOperator { lhs, rhs, .. } => {
            handle_expression(lhs, tokens);
            handle_expression(rhs, tokens);
        }
        Expression::VariableExpression { name, .. } => {
            if !name.as_str().contains(TUPLE_NAME_PREFIX)
                && !name.as_str().contains(MATCH_RETURN_VAR_NAME_PREFIX)
            {
                tokens.insert(
                    to_ident_key(name),
                    TokenType::from_parsed(AstToken::Expression(expression.clone())),
                );
            }
        }
        Expression::Tuple { fields, .. } => {
            for exp in fields {
                handle_expression(exp, tokens);
            }
        }
        Expression::TupleIndex { prefix, .. } => {
            handle_expression(prefix, tokens);
        }
        Expression::Array { contents, .. } => {
            for exp in contents {
                handle_expression(exp, tokens);
            }
        }
        Expression::StructExpression {
            call_path_binding,
            fields,
            ..
        } => {
            for ident in &call_path_binding.inner.prefixes {
                tokens.insert(
                    to_ident_key(ident),
                    TokenType::from_parsed(AstToken::Expression(expression.clone())),
                );
            }
            tokens.insert(
                to_ident_key(&Ident::new(call_path_binding.inner.suffix.1.clone())),
                TokenType::from_parsed(AstToken::Expression(expression.clone())),
            );

            // handle_custom_type(&struct_name.suffix.0, tokens);

            for field in fields {
                tokens.insert(
                    to_ident_key(&field.name),
                    TokenType::from_parsed(AstToken::Expression(field.value.clone())),
                );
                handle_expression(&field.value, tokens);
            }
        }
        Expression::CodeBlock { contents, .. } => {
            for node in &contents.contents {
                traverse_node(node, tokens);
            }
        }
        Expression::IfExp {
            condition,
            then,
            r#else,
            ..
        } => {
            handle_expression(condition, tokens);
            handle_expression(then, tokens);
            if let Some(r#else) = r#else {
                handle_expression(r#else, tokens);
            }
        }
        Expression::MatchExp {
            value, branches, ..
        } => {
            handle_expression(value, tokens);
            for branch in branches {
                // TODO: handle_scrutinee(branch.scrutinee, tokens);
                handle_expression(&branch.result, tokens);
            }
        }
        Expression::AsmExpression { .. } => {
            //TODO handle asm expressions
        }
        Expression::MethodApplication {
            method_name_binding,
            arguments,
            contract_call_params,
            ..
        } => {
            let prefixes = match &method_name_binding.inner {
                MethodName::FromType {
                    call_path_binding, ..
                } => call_path_binding.inner.prefixes.clone(),
                MethodName::FromTrait { call_path, .. } => call_path.prefixes.clone(),
                _ => vec![],
            };

            // Don't collect applications of desugared operators due to mismatched ident lengths.
            if !desugared_op(&prefixes) {
                tokens.insert(
                    to_ident_key(&method_name_binding.inner.easy_name()),
                    TokenType::from_parsed(AstToken::Expression(expression.clone())),
                );
            }

            for exp in arguments {
                handle_expression(exp, tokens);
            }

            for field in contract_call_params {
                tokens.insert(
                    to_ident_key(&field.name),
                    TokenType::from_parsed(AstToken::Expression(field.value.clone())),
                );
                handle_expression(&field.value, tokens);
            }
        }
        Expression::SubfieldExpression {
            prefix,
            field_to_access,
            ..
        } => {
            tokens.insert(
                to_ident_key(field_to_access),
                TokenType::from_parsed(AstToken::Expression(expression.clone())),
            );
            handle_expression(prefix, tokens);
        }
        Expression::DelineatedPath {
            call_path_binding,
            args,
            ..
        } => {
            for ident in &call_path_binding.inner.prefixes {
                tokens.insert(
                    to_ident_key(ident),
                    TokenType::from_parsed(AstToken::Expression(expression.clone())),
                );
            }
            tokens.insert(
                to_ident_key(&call_path_binding.inner.suffix),
                TokenType::from_parsed(AstToken::Expression(expression.clone())),
            );

            for exp in args {
                handle_expression(exp, tokens);
            }
        }
        Expression::AbiCast {
            abi_name, address, ..
        } => {
            for ident in &abi_name.prefixes {
                tokens.insert(
                    to_ident_key(ident),
                    TokenType::from_parsed(AstToken::Expression(expression.clone())),
                );
            }
            tokens.insert(
                to_ident_key(&abi_name.suffix),
                TokenType::from_parsed(AstToken::Expression(expression.clone())),
            );
            handle_expression(address, tokens);
        }
        Expression::ArrayIndex { prefix, index, .. } => {
            handle_expression(prefix, tokens);
            handle_expression(index, tokens);
        }
        Expression::StorageAccess { field_names, .. } => {
            for field in field_names {
                tokens.insert(
                    to_ident_key(field),
                    TokenType::from_parsed(AstToken::Expression(expression.clone())),
                );
            }
        }
        Expression::IntrinsicFunction { arguments, .. } => {
            for argument in arguments {
                handle_expression(argument, tokens);
            }
        }
    }
}

fn handle_while_loop(while_loop: &WhileLoop, tokens: &mut TokenMap) {
    handle_expression(&while_loop.condition, tokens);
    for node in &while_loop.body.contents {
        traverse_node(node, tokens);
    }
}
