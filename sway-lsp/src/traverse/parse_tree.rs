#![allow(dead_code)]
use std::iter;

use crate::core::{
    token::{
        desugared_op, to_ident_key, type_info_to_symbol_kind, AstToken, SymbolKind, Token,
        TypeDefinition,
    },
    token_map::TokenMap,
};
use sway_core::{
    language::{
        parsed::{
            AbiCastExpression, AmbiguousPathExpression, ArrayIndexExpression, AstNode,
            AstNodeContent, CodeBlock, Declaration, DelineatedPathExpression, Expression,
            ExpressionKind, FunctionApplicationExpression, FunctionDeclaration, FunctionParameter,
            IfExpression, IntrinsicFunctionExpression, LazyOperatorExpression, MatchExpression,
            MethodApplicationExpression, MethodName, ReassignmentTarget, Scrutinee,
            StorageAccessExpression, StructExpression, StructScrutineeField, SubfieldExpression,
            TraitFn, TupleIndexExpression, WhileLoopExpression,
        },
        Literal,
    },
    type_system::{TypeArgument, TypeParameter},
    TypeEngine, TypeInfo,
};
use sway_types::constants::{DESTRUCTURE_PREFIX, MATCH_RETURN_VAR_NAME_PREFIX, TUPLE_NAME_PREFIX};
use sway_types::{Ident, Span, Spanned};

pub fn traverse_node(type_engine: &TypeEngine, node: &AstNode, tokens: &TokenMap) {
    match &node.content {
        AstNodeContent::Declaration(declaration) => {
            handle_declaration(type_engine, declaration, tokens)
        }
        AstNodeContent::Expression(expression) => {
            handle_expression(type_engine, expression, tokens)
        }
        AstNodeContent::ImplicitReturnExpression(expression) => {
            handle_expression(type_engine, expression, tokens)
        }

        // TODO
        // handle other content types
        _ => {}
    };
}

fn handle_function_declation(
    type_engine: &TypeEngine,
    func: &FunctionDeclaration,
    tokens: &TokenMap,
) {
    let token = Token::from_parsed(
        AstToken::FunctionDeclaration(func.clone()),
        SymbolKind::Function,
    );
    tokens.insert(to_ident_key(&func.name), token.clone());
    for node in &func.body.contents {
        traverse_node(type_engine, node, tokens);
    }

    for parameter in &func.parameters {
        collect_function_parameter(type_engine, parameter, tokens);
    }

    for type_param in &func.type_parameters {
        collect_type_parameter(
            type_param,
            tokens,
            AstToken::FunctionDeclaration(func.clone()),
        );
    }

    collect_type_info_token(
        type_engine,
        tokens,
        &token,
        &func.return_type,
        Some(func.return_type_span.clone()),
        None,
    );
}

fn handle_declaration(type_engine: &TypeEngine, declaration: &Declaration, tokens: &TokenMap) {
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
                let symbol_kind = if variable.name.as_str().contains(DESTRUCTURE_PREFIX) {
                    SymbolKind::Struct
                } else {
                    SymbolKind::Variable
                };

                let token =
                    Token::from_parsed(AstToken::Declaration(declaration.clone()), symbol_kind);
                // We want to use the span from variable.name to construct a
                // new Ident as the name_override_opt can be set to one of the
                // const prefixes and not the actual token name.
                tokens.insert(
                    to_ident_key(&Ident::new(variable.name.span())),
                    token.clone(),
                );

                if let Some(type_ascription_span) = &variable.type_ascription_span {
                    collect_type_info_token(
                        type_engine,
                        tokens,
                        &token,
                        &variable.type_ascription,
                        Some(type_ascription_span.clone()),
                        None,
                    );
                }
            }
            handle_expression(type_engine, &variable.body, tokens);
        }
        Declaration::FunctionDeclaration(func) => {
            handle_function_declation(type_engine, func, tokens);
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
                collect_trait_fn(type_engine, trait_fn, tokens);
            }

            for func_dec in &trait_decl.methods {
                handle_function_declation(type_engine, func_dec, tokens);
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
                    type_engine,
                    tokens,
                    &token,
                    &field.type_info,
                    Some(field.type_span.clone()),
                    None,
                );
            }

            for type_param in &struct_dec.type_parameters {
                collect_type_parameter(
                    type_param,
                    tokens,
                    AstToken::Declaration(declaration.clone()),
                );
            }
        }
        Declaration::EnumDeclaration(enum_decl) => {
            tokens.insert(
                to_ident_key(&enum_decl.name),
                Token::from_parsed(AstToken::Declaration(declaration.clone()), SymbolKind::Enum),
            );

            for type_param in &enum_decl.type_parameters {
                collect_type_parameter(
                    type_param,
                    tokens,
                    AstToken::Declaration(declaration.clone()),
                );
            }

            for variant in &enum_decl.variants {
                let token =
                    Token::from_parsed(AstToken::EnumVariant(variant.clone()), SymbolKind::Variant);
                tokens.insert(to_ident_key(&variant.name), token.clone());

                collect_type_info_token(
                    type_engine,
                    tokens,
                    &token,
                    &variant.type_info,
                    Some(variant.type_span.clone()),
                    Some(SymbolKind::Variant),
                );
            }
        }
        Declaration::ImplTrait(impl_trait) => {
            for ident in &impl_trait.trait_name.prefixes {
                tokens.insert(
                    to_ident_key(ident),
                    Token::from_parsed(
                        AstToken::Declaration(declaration.clone()),
                        SymbolKind::Module,
                    ),
                );
            }

            tokens.insert(
                to_ident_key(&impl_trait.trait_name.suffix),
                Token::from_parsed(
                    AstToken::Declaration(declaration.clone()),
                    SymbolKind::Trait,
                ),
            );

            tokens.insert(
                to_ident_key(&Ident::new(impl_trait.type_implementing_for_span.clone())),
                Token::from_parsed(
                    AstToken::Declaration(declaration.clone()),
                    type_info_to_symbol_kind(type_engine, &impl_trait.type_implementing_for),
                ),
            );

            for type_param in &impl_trait.impl_type_parameters {
                collect_type_parameter(
                    type_param,
                    tokens,
                    AstToken::Declaration(declaration.clone()),
                );
            }

            for func_dec in &impl_trait.functions {
                handle_function_declation(type_engine, func_dec, tokens);
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
                if let Some(type_arguments) = type_arguments {
                    for type_arg in type_arguments {
                        collect_type_arg(type_engine, type_arg, &token, tokens);
                    }
                }
            }

            for type_param in &impl_self.impl_type_parameters {
                collect_type_parameter(
                    type_param,
                    tokens,
                    AstToken::Declaration(declaration.clone()),
                );
            }

            for func_dec in &impl_self.functions {
                handle_function_declation(type_engine, func_dec, tokens);
            }
        }
        Declaration::AbiDeclaration(abi_decl) => {
            tokens.insert(
                to_ident_key(&abi_decl.name),
                Token::from_parsed(
                    AstToken::Declaration(declaration.clone()),
                    SymbolKind::Trait,
                ),
            );

            for trait_fn in &abi_decl.interface_surface {
                collect_trait_fn(type_engine, trait_fn, tokens);
            }
        }
        Declaration::ConstantDeclaration(const_decl) => {
            let token = Token::from_parsed(
                AstToken::Declaration(declaration.clone()),
                SymbolKind::Const,
            );
            tokens.insert(to_ident_key(&const_decl.name), token.clone());

            collect_type_info_token(
                type_engine,
                tokens,
                &token,
                &const_decl.type_ascription,
                const_decl.type_ascription_span.clone(),
                None,
            );
            handle_expression(type_engine, &const_decl.value, tokens);
        }
        Declaration::StorageDeclaration(storage_decl) => {
            for field in &storage_decl.fields {
                let token =
                    Token::from_parsed(AstToken::StorageField(field.clone()), SymbolKind::Field);
                tokens.insert(to_ident_key(&field.name), token.clone());

                collect_type_info_token(
                    type_engine,
                    tokens,
                    &token,
                    &field.type_info,
                    Some(field.type_info_span.clone()),
                    None,
                );
                handle_expression(type_engine, &field.initializer, tokens);
            }
        }
    }
}

fn handle_expression(type_engine: &TypeEngine, expression: &Expression, tokens: &TokenMap) {
    let span = &expression.span;
    match &expression.kind {
        ExpressionKind::Error(_part_spans) => {
            // FIXME(Centril): Left for @JoshuaBatty to use.
        }
        ExpressionKind::Literal(value) => {
            let symbol_kind = literal_to_symbol_kind(value);

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
                            SymbolKind::Module,
                        ),
                    );
                }

                let token = Token::from_parsed(
                    AstToken::Expression(expression.clone()),
                    SymbolKind::Function,
                );

                tokens.insert(to_ident_key(&call_path_binding.inner.suffix), token.clone());

                for type_arg in &call_path_binding.type_arguments {
                    collect_type_arg(type_engine, type_arg, &token, tokens);
                }
            }

            for exp in arguments {
                handle_expression(type_engine, exp, tokens);
            }
        }
        ExpressionKind::LazyOperator(LazyOperatorExpression { lhs, rhs, .. }) => {
            handle_expression(type_engine, lhs, tokens);
            handle_expression(type_engine, rhs, tokens);
        }
        ExpressionKind::Variable(name) => {
            if !name.as_str().contains(TUPLE_NAME_PREFIX)
                && !name.as_str().contains(MATCH_RETURN_VAR_NAME_PREFIX)
            {
                let symbol_kind = if name.as_str().contains(DESTRUCTURE_PREFIX) {
                    SymbolKind::Struct
                } else {
                    SymbolKind::Variable
                };

                tokens.insert(
                    to_ident_key(name),
                    Token::from_parsed(AstToken::Expression(expression.clone()), symbol_kind),
                );
            }
        }
        ExpressionKind::Tuple(fields) => {
            for exp in fields {
                handle_expression(type_engine, exp, tokens);
            }
        }
        ExpressionKind::TupleIndex(TupleIndexExpression { prefix, .. }) => {
            handle_expression(type_engine, prefix, tokens);
        }
        ExpressionKind::Array(contents) => {
            for exp in contents {
                handle_expression(type_engine, exp, tokens);
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

            let name = &call_path_binding.inner.suffix;
            let type_arguments = &call_path_binding.type_arguments;

            let token =
                Token::from_parsed(AstToken::Expression(expression.clone()), SymbolKind::Struct);
            tokens.insert(to_ident_key(name), token.clone());
            for type_arg in type_arguments {
                collect_type_arg(type_engine, type_arg, &token, tokens);
            }

            for field in fields {
                tokens.insert(
                    to_ident_key(&field.name),
                    Token::from_parsed(
                        AstToken::StructExpressionField(field.clone()),
                        SymbolKind::Field,
                    ),
                );
                handle_expression(type_engine, &field.value, tokens);
            }
        }
        ExpressionKind::CodeBlock(contents) => {
            for node in &contents.contents {
                traverse_node(type_engine, node, tokens);
            }
        }
        ExpressionKind::If(IfExpression {
            condition,
            then,
            r#else,
            ..
        }) => {
            handle_expression(type_engine, condition, tokens);
            handle_expression(type_engine, then, tokens);
            if let Some(r#else) = r#else {
                handle_expression(type_engine, r#else, tokens);
            }
        }
        ExpressionKind::Match(MatchExpression {
            value, branches, ..
        }) => {
            handle_expression(type_engine, value, tokens);
            for branch in branches {
                collect_scrutinee(&branch.scrutinee, tokens);
                handle_expression(type_engine, &branch.result, tokens);
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
                let token = Token::from_parsed(
                    AstToken::Expression(expression.clone()),
                    SymbolKind::Struct,
                );
                let (type_info, span) = &call_path_binding.inner.suffix;
                collect_type_info_token(
                    type_engine,
                    tokens,
                    &token,
                    type_info,
                    Some(span.clone()),
                    None,
                );
            }

            // Don't collect applications of desugared operators due to mismatched ident lengths.
            if !desugared_op(&prefixes) {
                tokens.insert(
                    to_ident_key(&method_name_binding.inner.easy_name()),
                    Token::from_parsed(
                        AstToken::Expression(expression.clone()),
                        SymbolKind::Struct,
                    ),
                );
            }

            for exp in arguments {
                handle_expression(type_engine, exp, tokens);
            }

            for field in contract_call_params {
                tokens.insert(
                    to_ident_key(&field.name),
                    Token::from_parsed(
                        AstToken::Expression(field.value.clone()),
                        SymbolKind::Field,
                    ),
                );
                handle_expression(type_engine, &field.value, tokens);
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
            handle_expression(type_engine, prefix, tokens);
        }
        ExpressionKind::AmbiguousPathExpression(path_expr) => {
            let AmbiguousPathExpression {
                call_path_binding,
                args,
            } = &**path_expr;

            for ident in call_path_binding
                .inner
                .prefixes
                .iter()
                .chain(iter::once(&call_path_binding.inner.suffix.before.inner))
            {
                tokens.insert(
                    to_ident_key(ident),
                    Token::from_parsed(AstToken::Expression(expression.clone()), SymbolKind::Enum),
                );
            }

            let token = Token::from_parsed(
                AstToken::Expression(expression.clone()),
                SymbolKind::Variant,
            );

            tokens.insert(
                to_ident_key(&call_path_binding.inner.suffix.suffix),
                token.clone(),
            );

            for type_arg in &call_path_binding.type_arguments {
                collect_type_arg(type_engine, type_arg, &token, tokens);
            }

            for exp in args {
                handle_expression(type_engine, exp, tokens);
            }
        }
        ExpressionKind::DelineatedPath(delineated_path_expression) => {
            let DelineatedPathExpression {
                call_path_binding,
                args,
            } = &**delineated_path_expression;
            for ident in &call_path_binding.inner.prefixes {
                tokens.insert(
                    to_ident_key(ident),
                    Token::from_parsed(AstToken::Expression(expression.clone()), SymbolKind::Enum),
                );
            }

            let token = Token::from_parsed(
                AstToken::Expression(expression.clone()),
                SymbolKind::Variant,
            );

            tokens.insert(to_ident_key(&call_path_binding.inner.suffix), token.clone());

            for type_arg in &call_path_binding.type_arguments {
                collect_type_arg(type_engine, type_arg, &token, tokens);
            }

            for exp in args {
                handle_expression(type_engine, exp, tokens);
            }
        }
        ExpressionKind::AbiCast(abi_cast_expression) => {
            let AbiCastExpression { abi_name, address } = &**abi_cast_expression;
            for ident in &abi_name.prefixes {
                tokens.insert(
                    to_ident_key(ident),
                    Token::from_parsed(
                        AstToken::Expression(expression.clone()),
                        SymbolKind::Module,
                    ),
                );
            }
            tokens.insert(
                to_ident_key(&abi_name.suffix),
                Token::from_parsed(AstToken::Expression(expression.clone()), SymbolKind::Trait),
            );
            handle_expression(type_engine, address, tokens);
        }
        ExpressionKind::ArrayIndex(ArrayIndexExpression { prefix, index, .. }) => {
            handle_expression(type_engine, prefix, tokens);
            handle_expression(type_engine, index, tokens);
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
                handle_expression(type_engine, argument, tokens);
            }
        }
        ExpressionKind::WhileLoop(WhileLoopExpression {
            body, condition, ..
        }) => handle_while_loop(type_engine, body, condition, tokens),
        // TODO: collect these tokens as keywords once the compiler returns the span
        ExpressionKind::Break | ExpressionKind::Continue => {}
        ExpressionKind::Reassignment(reassignment) => {
            handle_expression(type_engine, &reassignment.rhs, tokens);

            match &reassignment.lhs {
                ReassignmentTarget::VariableExpression(exp) => {
                    handle_expression(type_engine, exp, tokens);
                }
                ReassignmentTarget::StorageField(idents) => {
                    for ident in idents {
                        tokens.insert(
                            to_ident_key(ident),
                            Token::from_parsed(
                                AstToken::Reassignment(reassignment.clone()),
                                SymbolKind::Field,
                            ),
                        );
                    }
                }
            }
        }
        ExpressionKind::Return(expr) => handle_expression(type_engine, expr, tokens),
    }
}

fn handle_while_loop(
    type_engine: &TypeEngine,
    body: &CodeBlock,
    condition: &Expression,
    tokens: &TokenMap,
) {
    handle_expression(type_engine, condition, tokens);
    for node in &body.contents {
        traverse_node(type_engine, node, tokens);
    }
}

fn literal_to_symbol_kind(value: &Literal) -> SymbolKind {
    match value {
        Literal::U8(..)
        | Literal::U16(..)
        | Literal::U32(..)
        | Literal::U64(..)
        | Literal::Numeric(..) => SymbolKind::NumericLiteral,
        Literal::String(..) => SymbolKind::StringLiteral,
        Literal::B256(..) => SymbolKind::ByteLiteral,
        Literal::Boolean(..) => SymbolKind::BoolLiteral,
    }
}

fn collect_type_arg(
    type_engine: &TypeEngine,
    type_argument: &TypeArgument,
    token: &Token,
    tokens: &TokenMap,
) {
    let mut token = token.clone();
    let type_info = type_engine.look_up_type_id(type_argument.type_id);
    match &type_info {
        TypeInfo::Array(type_arg, length) => {
            token.kind = SymbolKind::NumericLiteral;
            tokens.insert(to_ident_key(&Ident::new(length.span())), token.clone());
            collect_type_arg(type_engine, &type_arg, &token, tokens);
        }
        TypeInfo::Tuple(type_arguments) => {
            for type_arg in type_arguments {
                collect_type_arg(type_engine, type_arg, &token, tokens);
            }
        }
        _ => {
            let symbol_kind = type_info_to_symbol_kind(type_engine, &type_info);
            token.kind = symbol_kind;
            token.type_def = Some(TypeDefinition::TypeId(type_argument.type_id));
            tokens.insert(to_ident_key(&Ident::new(type_argument.span.clone())), token);
        }
    }
}

fn collect_scrutinee(scrutinee: &Scrutinee, tokens: &TokenMap) {
    match scrutinee {
        Scrutinee::CatchAll { .. } => (),
        Scrutinee::Literal { ref value, span } => {
            let token = Token::from_parsed(
                AstToken::Scrutinee(scrutinee.clone()),
                literal_to_symbol_kind(value),
            );
            tokens.insert(to_ident_key(&Ident::new(span.clone())), token);
        }
        Scrutinee::Variable { name, .. } => {
            let token =
                Token::from_parsed(AstToken::Scrutinee(scrutinee.clone()), SymbolKind::Variable);
            tokens.insert(to_ident_key(name), token);
        }
        Scrutinee::StructScrutinee {
            struct_name,
            fields,
            ..
        } => {
            let token =
                Token::from_parsed(AstToken::Scrutinee(scrutinee.clone()), SymbolKind::Struct);
            tokens.insert(to_ident_key(struct_name), token);

            for field in fields {
                let token =
                    Token::from_parsed(AstToken::Scrutinee(scrutinee.clone()), SymbolKind::Field);
                if let StructScrutineeField::Field {
                    field, scrutinee, ..
                } = field
                {
                    tokens.insert(to_ident_key(field), token);

                    if let Some(scrutinee) = scrutinee {
                        collect_scrutinee(scrutinee, tokens);
                    }
                }
            }
        }
        Scrutinee::EnumScrutinee {
            call_path, value, ..
        } => {
            for ident in &call_path.prefixes {
                let token =
                    Token::from_parsed(AstToken::Scrutinee(scrutinee.clone()), SymbolKind::Enum);
                tokens.insert(to_ident_key(ident), token);
            }

            let token =
                Token::from_parsed(AstToken::Scrutinee(scrutinee.clone()), SymbolKind::Variant);
            tokens.insert(to_ident_key(&call_path.suffix), token);

            collect_scrutinee(value, tokens);
        }
        Scrutinee::Tuple { elems, .. } => {
            for elem in elems {
                collect_scrutinee(elem, tokens);
            }
        }
    }
}

fn collect_type_info_token(
    type_engine: &TypeEngine,
    tokens: &TokenMap,
    token: &Token,
    type_info: &TypeInfo,
    type_span: Option<Span>,
    symbol_kind: Option<SymbolKind>,
) {
    let mut token = token.clone();
    match symbol_kind {
        Some(kind) => token.kind = kind,
        None => token.kind = type_info_to_symbol_kind(type_engine, type_info),
    }

    match type_info {
        TypeInfo::UnsignedInteger(..) | TypeInfo::Boolean | TypeInfo::B256 => {
            if let Some(type_span) = type_span {
                tokens.insert(to_ident_key(&Ident::new(type_span)), token);
            }
        }
        TypeInfo::Str(length) => {
            tokens.insert(to_ident_key(&Ident::new(length.span())), token);
        }
        TypeInfo::Array(type_arg, length) => {
            token.kind = SymbolKind::NumericLiteral;
            tokens.insert(to_ident_key(&Ident::new(length.span())), token.clone());
            collect_type_arg(type_engine, type_arg, &token, tokens);
        }
        TypeInfo::Tuple(type_arguments) => {
            for type_arg in type_arguments {
                collect_type_arg(type_engine, type_arg, &token, tokens);
            }
        }
        TypeInfo::Custom {
            name,
            type_arguments,
        } => {
            token.type_def = Some(TypeDefinition::Ident(name.clone()));
            tokens.insert(to_ident_key(name), token.clone());
            if let Some(type_arguments) = type_arguments {
                for type_arg in type_arguments {
                    collect_type_arg(type_engine, type_arg, &token, tokens);
                }
            }
        }
        _ => (),
    }
}

fn collect_function_parameter(
    type_engine: &TypeEngine,
    parameter: &FunctionParameter,
    tokens: &TokenMap,
) {
    let token = Token::from_parsed(
        AstToken::FunctionParameter(parameter.clone()),
        SymbolKind::ValueParam,
    );
    tokens.insert(to_ident_key(&parameter.name), token.clone());

    collect_type_info_token(
        type_engine,
        tokens,
        &token,
        &parameter.type_info,
        Some(parameter.type_span.clone()),
        None,
    );
}

fn collect_trait_fn(type_engine: &TypeEngine, trait_fn: &TraitFn, tokens: &TokenMap) {
    let token = Token::from_parsed(AstToken::TraitFn(trait_fn.clone()), SymbolKind::Function);
    tokens.insert(to_ident_key(&trait_fn.name), token.clone());

    for parameter in &trait_fn.parameters {
        collect_function_parameter(type_engine, parameter, tokens);
    }

    collect_type_info_token(
        type_engine,
        tokens,
        &token,
        &trait_fn.return_type,
        Some(trait_fn.return_type_span.clone()),
        None,
    );
}

fn collect_type_parameter(type_param: &TypeParameter, tokens: &TokenMap, token: AstToken) {
    tokens.insert(
        to_ident_key(&type_param.name_ident),
        Token::from_parsed(token, SymbolKind::TypeParameter),
    );
}
