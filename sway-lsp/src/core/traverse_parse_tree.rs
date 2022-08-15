#![allow(dead_code)]
use crate::{
    core::token::{AstToken, SymbolKind, Token, TokenMap, TypeDefinition},
    utils::token::{desugared_op, to_ident_key, type_info_to_symbol_kind},
};
use sway_core::{
    constants::{DESTRUCTURE_PREFIX, MATCH_RETURN_VAR_NAME_PREFIX, TUPLE_NAME_PREFIX},
    parse_tree::{Literal, MethodName},
    type_system::TypeArgument,
    AbiCastExpression, ArrayIndexExpression, AstNode, AstNodeContent, CodeBlock, Declaration,
    DelineatedPathExpression, Expression, ExpressionKind, FunctionApplicationExpression,
    FunctionDeclaration, FunctionParameter, IfExpression, IntrinsicFunctionExpression,
    LazyOperatorExpression, MatchExpression, MethodApplicationExpression, ReassignmentTarget,
    StorageAccessExpression, StructExpression, SubfieldExpression, TupleIndexExpression, TypeInfo,
    WhileLoopExpression,
};
use sway_types::{Ident, Span};

pub fn traverse_node(node: &AstNode, tokens: &TokenMap) {
    match &node.content {
        AstNodeContent::Declaration(declaration) => handle_declaration(declaration, tokens),
        AstNodeContent::Expression(expression) => handle_expression(expression, tokens),
        AstNodeContent::ImplicitReturnExpression(expression) => {
            handle_expression(expression, tokens)
        }
        AstNodeContent::ReturnStatement(return_statement) => {
            handle_expression(&return_statement.expr, tokens)
        }

        // TODO
        // handle other content types
        _ => {}
    };
}

fn handle_function_declation(func: &FunctionDeclaration, tokens: &TokenMap) {
    tokens.insert(
        to_ident_key(&func.name),
        Token::from_parsed(
            AstToken::FunctionDeclaration(func.clone()),
            SymbolKind::Function,
        ),
    );
    for node in &func.body.contents {
        traverse_node(node, tokens);
    }

    for parameter in &func.parameters {
        collect_function_parameter(parameter, tokens);
    }

    if let TypeInfo::Custom {
        name,
        type_arguments,
    } = &func.return_type
    {
        let token = Token::from_parsed(
            AstToken::FunctionDeclaration(func.clone()),
            SymbolKind::Struct,
        );
        tokens.insert(to_ident_key(name), token.clone());
        if let Some(args) = type_arguments {
            collect_type_args(args, &token, tokens);
        }
    }
}

fn handle_declaration(declaration: &Declaration, tokens: &TokenMap) {
    match declaration {
        Declaration::VariableDeclaration(variable) => {
            // Don't collect tokens if the ident's name contains __tuple_ || __match_return_var_name_
            // The individual elements are handled in the subsequent VariableDeclaration's
            if !variable.name.as_str().contains(TUPLE_NAME_PREFIX)
                && !variable
                    .name
                    .as_str()
                    .contains(MATCH_RETURN_VAR_NAME_PREFIX)
                && !variable.name.as_str().contains(DESTRUCTURE_PREFIX)
            {
                tokens.insert(
                    to_ident_key(&variable.name),
                    Token::from_parsed(
                        AstToken::Declaration(declaration.clone()),
                        SymbolKind::Variable,
                    ),
                );

                if let Some(type_ascription_span) = &variable.type_ascription_span {
                    tokens.insert(
                        to_ident_key(&Ident::new(type_ascription_span.clone())),
                        Token::from_parsed(AstToken::Declaration(declaration.clone())),
                    );
                }
            }
            handle_expression(&variable.body, tokens);
        }
        Declaration::FunctionDeclaration(func) => {
            handle_function_declation(func, tokens);
        }
        Declaration::TraitDeclaration(trait_decl) => {
            tokens.insert(
                to_ident_key(&trait_decl.name),
                Token::from_parsed(
                    AstToken::Declaration(declaration.clone()),
                    SymbolKind::Trait,
                ),
            );

            for trait_fn in &trait_decl.interface_surface {
                tokens.insert(
                    to_ident_key(&trait_fn.name),
                    Token::from_parsed(AstToken::TraitFn(trait_fn.clone()), SymbolKind::Method),
                );
            }

            for func_dec in &trait_decl.methods {
                handle_function_declation(func_dec, tokens);
            }
        }
        Declaration::StructDeclaration(struct_dec) => {
            tokens.insert(
                to_ident_key(&struct_dec.name),
                Token::from_parsed(
                    AstToken::Declaration(declaration.clone()),
                    SymbolKind::Struct,
                ),
            );
            for field in &struct_dec.fields {
                let token =
                    Token::from_parsed(AstToken::StructField(field.clone()), SymbolKind::Field);
                tokens.insert(to_ident_key(&field.name), token.clone());

                collect_type_info_token(
                    tokens,
                    &token,
                    &field.type_info,
                    Some(field.type_span.clone()),
                );
            }
        }
        Declaration::EnumDeclaration(enum_decl) => {
            tokens.insert(
                to_ident_key(&enum_decl.name),
                Token::from_parsed(AstToken::Declaration(declaration.clone()), SymbolKind::Enum),
            );
            for variant in &enum_decl.variants {
                tokens.insert(
                    to_ident_key(&variant.name),
                    Token::from_parsed(AstToken::EnumVariant(variant.clone()), SymbolKind::Variant),
                );
            }
        }
        Declaration::ImplTrait(impl_trait) => {
            for ident in &impl_trait.trait_name.prefixes {
                tokens.insert(
                    to_ident_key(ident),
                    Token::from_parsed(
                        AstToken::Declaration(declaration.clone()),
                        SymbolKind::Unknown,
                    ),
                );
            }

            tokens.insert(
                to_ident_key(&impl_trait.trait_name.suffix),
                Token::from_parsed(
                    AstToken::Declaration(declaration.clone()),
                    SymbolKind::Unknown,
                ),
            );

            for func_dec in &impl_trait.functions {
                handle_function_declation(func_dec, tokens);
            }
        }
        Declaration::ImplSelf(impl_self) => {
            if let TypeInfo::Custom {
                name,
                type_arguments,
            } = &impl_self.type_implementing_for
            {
                let token = Token::from_parsed(
                    AstToken::Declaration(declaration.clone()),
                    SymbolKind::Struct,
                );
                tokens.insert(to_ident_key(name), token.clone());
                if let Some(args) = type_arguments {
                    collect_type_args(args, &token, tokens);
                }
            }

            for func_dec in &impl_self.functions {
                handle_function_declation(func_dec, tokens);
            }
        }
        Declaration::AbiDeclaration(abi_decl) => {
            tokens.insert(
                to_ident_key(&abi_decl.name),
                Token::from_parsed(
                    AstToken::Declaration(declaration.clone()),
                    SymbolKind::Unknown,
                ),
            );
            for trait_fn in &abi_decl.interface_surface {
                tokens.insert(
                    to_ident_key(&trait_fn.name),
                    Token::from_parsed(AstToken::TraitFn(trait_fn.clone()), SymbolKind::Function),
                );

                for parameter in &trait_fn.parameters {
                    collect_function_parameter(parameter, tokens);
                }

                if let TypeInfo::Custom {
                    name,
                    type_arguments,
                } = &trait_fn.return_type
                {
                    let token =
                        Token::from_parsed(AstToken::TraitFn(trait_fn.clone()), SymbolKind::Struct);
                    tokens.insert(to_ident_key(name), token.clone());
                    if let Some(args) = type_arguments {
                        collect_type_args(args, &token, tokens);
                    }
                }
            }
        }
        Declaration::ConstantDeclaration(const_decl) => {
            tokens.insert(
                to_ident_key(&const_decl.name),
                Token::from_parsed(
                    AstToken::Declaration(declaration.clone()),
                    SymbolKind::Const,
                ),
            );
            handle_expression(&const_decl.value, tokens);
        }
        Declaration::StorageDeclaration(storage_decl) => {
            for field in &storage_decl.fields {
                let token =
                    Token::from_parsed(AstToken::StorageField(field.clone()), SymbolKind::Field);
                tokens.insert(to_ident_key(&field.name), token.clone());

                collect_type_info_token(tokens, &token, &field.type_info, None);
                handle_expression(&field.initializer, tokens);
            }
        }
    }
}

fn handle_expression(expression: &Expression, tokens: &TokenMap) {
    let span = &expression.span;
    match &expression.kind {
        ExpressionKind::Literal(value) => {
            let symbol_kind = match &value {
                Literal::U8(..)
                | Literal::U16(..)
                | Literal::U32(..)
                | Literal::U64(..)
                | Literal::Numeric(..) => SymbolKind::NumericLiteral,
                Literal::String(..) => SymbolKind::StringLiteral,
                Literal::Byte(..) | Literal::B256(..) => SymbolKind::ByteLiteral,
                Literal::Boolean(..) => SymbolKind::BoolLiteral,
            };

            tokens.insert(
                to_ident_key(&Ident::new(span.clone())),
                Token::from_parsed(AstToken::Expression(expression.clone()), symbol_kind),
            );
        }
        ExpressionKind::FunctionApplication(function_application_expression) => {
            let FunctionApplicationExpression {
                call_path_binding,
                arguments,
            } = &**function_application_expression;
            // Don't collect applications of desugared operators due to mismatched ident lengths.
            if !desugared_op(&call_path_binding.inner.prefixes) {
                for ident in &call_path_binding.inner.prefixes {
                    tokens.insert(
                        to_ident_key(ident),
                        Token::from_parsed(
                            AstToken::Expression(expression.clone()),
                            SymbolKind::Unknown,
                        ),
                    );
                }
                tokens.insert(
                    to_ident_key(&call_path_binding.inner.suffix),
                    Token::from_parsed(
                        AstToken::Expression(expression.clone()),
                        SymbolKind::Unknown,
                    ),
                );
            }

            for exp in arguments {
                handle_expression(exp, tokens);
            }
        }
        ExpressionKind::LazyOperator(LazyOperatorExpression { lhs, rhs, .. }) => {
            handle_expression(lhs, tokens);
            handle_expression(rhs, tokens);
        }
        ExpressionKind::Variable(name) => {
            if !name.as_str().contains(TUPLE_NAME_PREFIX)
                && !name.as_str().contains(MATCH_RETURN_VAR_NAME_PREFIX)
                && !name.as_str().contains(DESTRUCTURE_PREFIX)
            {
                tokens.insert(
                    to_ident_key(name),
                    Token::from_parsed(
                        AstToken::Expression(expression.clone()),
                        SymbolKind::Variable,
                    ),
                );
            }
        }
        ExpressionKind::Tuple(fields) => {
            for exp in fields {
                handle_expression(exp, tokens);
            }
        }
        ExpressionKind::TupleIndex(TupleIndexExpression { prefix, .. }) => {
            handle_expression(prefix, tokens);
        }
        ExpressionKind::Array(contents) => {
            for exp in contents {
                handle_expression(exp, tokens);
            }
        }
        ExpressionKind::Struct(struct_expression) => {
            let StructExpression {
                call_path_binding,
                fields,
            } = &**struct_expression;
            for ident in &call_path_binding.inner.prefixes {
                tokens.insert(
                    to_ident_key(ident),
                    Token::from_parsed(
                        AstToken::Expression(expression.clone()),
                        SymbolKind::Struct,
                    ),
                );
            }

            if let (
                TypeInfo::Custom {
                    name,
                    type_arguments,
                },
                ..,
            ) = &call_path_binding.inner.suffix
            {
                let token = Token::from_parsed(
                    AstToken::Expression(expression.clone()),
                    SymbolKind::Struct,
                );
                tokens.insert(to_ident_key(name), token.clone());
                if let Some(args) = type_arguments {
                    collect_type_args(args, &token, tokens);
                }
            }

            for field in fields {
                tokens.insert(
                    to_ident_key(&field.name),
                    Token::from_parsed(
                        AstToken::StructExpressionField(field.clone()),
                        SymbolKind::Field,
                    ),
                );
                handle_expression(&field.value, tokens);
            }
        }
        ExpressionKind::CodeBlock(contents) => {
            for node in &contents.contents {
                traverse_node(node, tokens);
            }
        }
        ExpressionKind::If(IfExpression {
            condition,
            then,
            r#else,
            ..
        }) => {
            handle_expression(condition, tokens);
            handle_expression(then, tokens);
            if let Some(r#else) = r#else {
                handle_expression(r#else, tokens);
            }
        }
        ExpressionKind::Match(MatchExpression {
            value, branches, ..
        }) => {
            handle_expression(value, tokens);
            for branch in branches {
                // TODO: handle_scrutinee(branch.scrutinee, tokens);
                handle_expression(&branch.result, tokens);
            }
        }
        ExpressionKind::Asm(_) => {
            //TODO handle asm expressions
        }
        ExpressionKind::MethodApplication(method_application_expression) => {
            let MethodApplicationExpression {
                method_name_binding,
                arguments,
                contract_call_params,
            } = &**method_application_expression;
            let prefixes = match &method_name_binding.inner {
                MethodName::FromType {
                    call_path_binding, ..
                } => call_path_binding.inner.prefixes.clone(),
                MethodName::FromTrait { call_path, .. } => call_path.prefixes.clone(),
                _ => vec![],
            };

            if let MethodName::FromType {
                call_path_binding, ..
            } = &method_name_binding.inner
            {
                if let (
                    TypeInfo::Custom {
                        name,
                        type_arguments,
                    },
                    ..,
                ) = &call_path_binding.inner.suffix
                {
                    let token = Token::from_parsed(
                        AstToken::Expression(expression.clone()),
                        SymbolKind::Struct,
                    );
                    tokens.insert(to_ident_key(name), token.clone());
                    if let Some(args) = type_arguments {
                        collect_type_args(args, &token, tokens);
                    }
                }
            }

            // Don't collect applications of desugared operators due to mismatched ident lengths.
            if !desugared_op(&prefixes) {
                tokens.insert(
                    to_ident_key(&method_name_binding.inner.easy_name()),
                    Token::from_parsed(
                        AstToken::Expression(expression.clone()),
                        SymbolKind::Unknown,
                    ),
                );
            }

            for exp in arguments {
                handle_expression(exp, tokens);
            }

            for field in contract_call_params {
                tokens.insert(
                    to_ident_key(&field.name),
                    Token::from_parsed(
                        AstToken::Expression(field.value.clone()),
                        SymbolKind::Field,
                    ),
                );
                handle_expression(&field.value, tokens);
            }
        }
        ExpressionKind::Subfield(SubfieldExpression {
            prefix,
            field_to_access,
            ..
        }) => {
            tokens.insert(
                to_ident_key(field_to_access),
                Token::from_parsed(AstToken::Expression(expression.clone()), SymbolKind::Field),
            );
            handle_expression(prefix, tokens);
        }
        ExpressionKind::DelineatedPath(delineated_path_expression) => {
            let DelineatedPathExpression {
                call_path_binding,
                args,
            } = &**delineated_path_expression;
            for ident in &call_path_binding.inner.prefixes {
                tokens.insert(
                    to_ident_key(ident),
                    Token::from_parsed(
                        AstToken::Expression(expression.clone()),
                        SymbolKind::Unknown,
                    ),
                );
            }
            tokens.insert(
                to_ident_key(&call_path_binding.inner.suffix),
                Token::from_parsed(
                    AstToken::Expression(expression.clone()),
                    SymbolKind::Unknown,
                ),
            );

            for exp in args {
                handle_expression(exp, tokens);
            }
        }
        ExpressionKind::AbiCast(abi_cast_expression) => {
            let AbiCastExpression { abi_name, address } = &**abi_cast_expression;
            for ident in &abi_name.prefixes {
                tokens.insert(
                    to_ident_key(ident),
                    Token::from_parsed(
                        AstToken::Expression(expression.clone()),
                        SymbolKind::Unknown,
                    ),
                );
            }
            tokens.insert(
                to_ident_key(&abi_name.suffix),
                Token::from_parsed(
                    AstToken::Expression(expression.clone()),
                    SymbolKind::Unknown,
                ),
            );
            handle_expression(address, tokens);
        }
        ExpressionKind::ArrayIndex(ArrayIndexExpression { prefix, index, .. }) => {
            handle_expression(prefix, tokens);
            handle_expression(index, tokens);
        }
        ExpressionKind::StorageAccess(StorageAccessExpression { field_names, .. }) => {
            for field in field_names {
                tokens.insert(
                    to_ident_key(field),
                    Token::from_parsed(AstToken::Expression(expression.clone()), SymbolKind::Field),
                );
            }
        }
        ExpressionKind::IntrinsicFunction(IntrinsicFunctionExpression { arguments, .. }) => {
            for argument in arguments {
                handle_expression(argument, tokens);
            }
        }
        ExpressionKind::WhileLoop(WhileLoopExpression {
            body, condition, ..
        }) => handle_while_loop(body, condition, tokens),
        // TODO: collect these tokens as keywords once the compiler returns the span
        ExpressionKind::Break | ExpressionKind::Continue => (),
        ExpressionKind::Reassignment(reassignment) => {
            handle_expression(&reassignment.rhs, tokens);

            match &reassignment.lhs {
                ReassignmentTarget::VariableExpression(exp) => {
                    handle_expression(exp, tokens);
                }
                ReassignmentTarget::StorageField(idents) => {
                    for ident in idents {
                        tokens.insert(
                            to_ident_key(ident),
                            Token::from_parsed(AstToken::Reassignment(reassignment.clone()), SymbolKind::Field),
                        );
                    }
                }
            }
        }
    }
}

fn handle_while_loop(body: &CodeBlock, condition: &Expression, tokens: &TokenMap) {
    handle_expression(condition, tokens);
    for node in &body.contents {
        traverse_node(node, tokens);
    }
}

fn collect_type_args(type_arguments: &Vec<TypeArgument>, token: &Token, tokens: &TokenMap) {
    for arg in type_arguments {
        let mut token = token.clone();
        let type_info = sway_core::type_system::look_up_type_id(arg.type_id);
        let symbol_kind = type_info_to_symbol_kind(&type_info);
        token.kind = symbol_kind;
        token.type_def = Some(TypeDefinition::TypeId(arg.type_id));
        tokens.insert(to_ident_key(&Ident::new(arg.span.clone())), token);
    }
}

fn collect_type_info_token(
    tokens: &TokenMap,
    token: &Token,
    type_info: &TypeInfo,
    type_span: Option<Span>,
) {
    let mut token = token.clone();
    let symbol_kind = type_info_to_symbol_kind(type_info);
    token.kind = symbol_kind;

    match type_info {
        TypeInfo::UnsignedInteger(..) | TypeInfo::Boolean | TypeInfo::Byte | TypeInfo::B256 => {
            if let Some(type_span) = type_span {
                tokens.insert(to_ident_key(&Ident::new(type_span)), token);
            }
        }
        TypeInfo::Ref(type_id, span) => {
            token.type_def = Some(TypeDefinition::TypeId(*type_id));
            tokens.insert(to_ident_key(&Ident::new(span.clone())), token);
        }
        TypeInfo::Custom {
            name,
            type_arguments,
        } => {
            token.type_def = Some(TypeDefinition::Ident(name.clone()));
            tokens.insert(to_ident_key(name), token.clone());
            if let Some(args) = type_arguments {
                collect_type_args(args, &token, tokens);
            }
        }
        _ => (),
    }
}

fn collect_function_parameter(parameter: &FunctionParameter, tokens: &TokenMap) {
    let token = Token::from_parsed(
        AstToken::FunctionParameter(parameter.clone()),
        SymbolKind::ValueParam,
    );
    tokens.insert(to_ident_key(&parameter.name), token.clone());

    let type_info = sway_core::type_system::look_up_type_id(parameter.type_id);

    collect_type_info_token(
        tokens,
        &token,
        &type_info,
        Some(parameter.type_span.clone()),
    );
}
