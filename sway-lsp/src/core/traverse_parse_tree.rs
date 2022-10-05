#![allow(dead_code)]
use crate::{
    core::token::{AstToken, SymbolKind, Token, TokenMap, TypeDefinition},
    utils::token::{desugared_op, to_ident_key, type_info_to_symbol_kind},
};
use sway_core::{
    constants::{DESTRUCTURE_PREFIX, MATCH_RETURN_VAR_NAME_PREFIX, TUPLE_NAME_PREFIX},
    language::parsed::{
        AbiCastExpression, ArrayIndexExpression, AstNode, AstNodeContent, CodeBlock, Declaration,
        DelineatedPathExpression, Expression, ExpressionKind, FunctionApplicationExpression,
        FunctionDeclaration, FunctionParameter, IfExpression, IntrinsicFunctionExpression,
        LazyOperatorExpression, Literal, MatchExpression, MethodApplicationExpression, MethodName,
        ReassignmentTarget, Scrutinee, StorageAccessExpression, StructExpression,
        StructScrutineeField, SubfieldExpression, TraitFn, TupleIndexExpression,
        WhileLoopExpression,
    },
    type_system::{TypeArgument, TypeParameter},
    TypeInfo,
};
use sway_types::{Ident, Span, Spanned};

pub fn traverse_node(node: &AstNode, tokens: &TokenMap) {
    match &node.content {
        AstNodeContent::Declaration(declaration) => handle_declaration(declaration, tokens),
        AstNodeContent::Expression(expression) => handle_expression(expression, tokens),
        AstNodeContent::ImplicitReturnExpression(expression) => {
            handle_expression(expression, tokens)
        }

        // TODO
        // handle other content types
        _ => {}
    };
}

fn handle_function_declation(func: &FunctionDeclaration, tokens: &TokenMap) {
    let token = Token::from_parsed(
        AstToken::FunctionDeclaration(func.clone()),
        SymbolKind::Function,
    );
    tokens.insert(to_ident_key(&func.name), token.clone());
    for node in &func.body.contents {
        traverse_node(node, tokens);
    }

    for parameter in &func.parameters {
        collect_function_parameter(parameter, tokens);
    }

    for type_param in &func.type_parameters {
        collect_type_parameter(
            type_param,
            tokens,
            AstToken::FunctionDeclaration(func.clone()),
        );
    }

    collect_type_info_token(
        tokens,
        &token,
        &func.return_type,
        Some(func.return_type_span.clone()),
        None,
    );
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
                        tokens,
                        &token,
                        &variable.type_ascription,
                        Some(type_ascription_span.clone()),
                        None,
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
                collect_trait_fn(trait_fn, tokens);
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
                    type_info_to_symbol_kind(&impl_trait.type_implementing_for),
                ),
            );

            for type_param in &impl_trait.type_parameters {
                collect_type_parameter(
                    type_param,
                    tokens,
                    AstToken::Declaration(declaration.clone()),
                );
            }

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

            for type_param in &impl_self.type_parameters {
                collect_type_parameter(
                    type_param,
                    tokens,
                    AstToken::Declaration(declaration.clone()),
                );
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
                    SymbolKind::Trait,
                ),
            );

            for trait_fn in &abi_decl.interface_surface {
                collect_trait_fn(trait_fn, tokens);
            }
        }
        Declaration::ConstantDeclaration(const_decl) => {
            let token = Token::from_parsed(
                AstToken::Declaration(declaration.clone()),
                SymbolKind::Const,
            );
            tokens.insert(to_ident_key(&const_decl.name), token.clone());

            collect_type_info_token(
                tokens,
                &token,
                &const_decl.type_ascription,
                const_decl.type_ascription_span.clone(),
                None,
            );
            handle_expression(&const_decl.value, tokens);
        }
        Declaration::StorageDeclaration(storage_decl) => {
            for field in &storage_decl.fields {
                let token =
                    Token::from_parsed(AstToken::StorageField(field.clone()), SymbolKind::Field);
                tokens.insert(to_ident_key(&field.name), token.clone());

                collect_type_info_token(
                    tokens,
                    &token,
                    &field.type_info,
                    Some(field.type_info_span.clone()),
                    None,
                );
                handle_expression(&field.initializer, tokens);
            }
        }
    }
}

fn handle_expression(expression: &Expression, tokens: &TokenMap) {
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

                collect_type_args(&call_path_binding.type_arguments, &token, tokens);
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
                collect_scrutinee(&branch.scrutinee, tokens);
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
                let token = Token::from_parsed(
                    AstToken::Expression(expression.clone()),
                    SymbolKind::Struct,
                );
                let (type_info, span) = &call_path_binding.inner.suffix;
                collect_type_info_token(tokens, &token, type_info, Some(span.clone()), None);
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
                    Token::from_parsed(AstToken::Expression(expression.clone()), SymbolKind::Enum),
                );
            }

            let token = Token::from_parsed(
                AstToken::Expression(expression.clone()),
                SymbolKind::Variant,
            );

            tokens.insert(to_ident_key(&call_path_binding.inner.suffix), token.clone());

            collect_type_args(&call_path_binding.type_arguments, &token, tokens);

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
                        SymbolKind::Module,
                    ),
                );
            }
            tokens.insert(
                to_ident_key(&abi_name.suffix),
                Token::from_parsed(AstToken::Expression(expression.clone()), SymbolKind::Trait),
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
        ExpressionKind::Break | ExpressionKind::Continue => {}
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
                            Token::from_parsed(
                                AstToken::Reassignment(reassignment.clone()),
                                SymbolKind::Field,
                            ),
                        );
                    }
                }
            }
        }
        ExpressionKind::Return(expr) => handle_expression(expr, tokens),
    }
}

fn handle_while_loop(body: &CodeBlock, condition: &Expression, tokens: &TokenMap) {
    handle_expression(condition, tokens);
    for node in &body.contents {
        traverse_node(node, tokens);
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
        Literal::Byte(..) | Literal::B256(..) => SymbolKind::ByteLiteral,
        Literal::Boolean(..) => SymbolKind::BoolLiteral,
    }
}

fn collect_type_args(type_arguments: &Vec<TypeArgument>, token: &Token, tokens: &TokenMap) {
    for arg in type_arguments {
        let mut token = token.clone();
        let type_info = sway_core::type_system::look_up_type_id(arg.type_id);
        // TODO handle tuple and arrays in type_arguments - https://github.com/FuelLabs/sway/issues/2486
        if let TypeInfo::Tuple(_) | TypeInfo::Array(_, _, _) = type_info {
            continue;
        }
        let symbol_kind = type_info_to_symbol_kind(&type_info);
        token.kind = symbol_kind;
        token.type_def = Some(TypeDefinition::TypeId(arg.type_id));
        tokens.insert(to_ident_key(&Ident::new(arg.span.clone())), token);
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
    tokens: &TokenMap,
    token: &Token,
    type_info: &TypeInfo,
    type_span: Option<Span>,
    symbol_kind: Option<SymbolKind>,
) {
    let mut token = token.clone();
    match symbol_kind {
        Some(kind) => token.kind = kind,
        None => token.kind = type_info_to_symbol_kind(type_info),
    }

    match type_info {
        TypeInfo::UnsignedInteger(..) | TypeInfo::Boolean | TypeInfo::Byte | TypeInfo::B256 => {
            if let Some(type_span) = type_span {
                tokens.insert(to_ident_key(&Ident::new(type_span)), token);
            }
        }
        TypeInfo::Tuple(args) => {
            collect_type_args(args, &token, tokens);
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

    collect_type_info_token(
        tokens,
        &token,
        &parameter.type_info,
        Some(parameter.type_span.clone()),
        None,
    );
}

fn collect_trait_fn(trait_fn: &TraitFn, tokens: &TokenMap) {
    let token = Token::from_parsed(AstToken::TraitFn(trait_fn.clone()), SymbolKind::Function);
    tokens.insert(to_ident_key(&trait_fn.name), token.clone());

    for parameter in &trait_fn.parameters {
        collect_function_parameter(parameter, tokens);
    }

    collect_type_info_token(
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
