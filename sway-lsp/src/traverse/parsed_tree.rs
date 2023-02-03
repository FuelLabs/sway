#![allow(dead_code)]
use std::iter;

use crate::{
    core::{
        token::{
            desugared_op, to_ident_key, type_info_to_symbol_kind, AstToken, SymbolKind, Token,
            TypeDefinition,
        },
        token_map::TokenMap,
    },
    traverse::Parse,
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
    transform::{AttributeKind, AttributesMap},
    type_system::{TypeArgument, TypeParameter},
    TypeEngine, TypeInfo,
};
use sway_types::constants::{DESTRUCTURE_PREFIX, MATCH_RETURN_VAR_NAME_PREFIX, TUPLE_NAME_PREFIX};
use sway_types::{Ident, Span, Spanned};

pub struct ParsedTree<'a> {
    type_engine: &'a TypeEngine,
    tokens: &'a TokenMap,
}

impl<'a> ParsedTree<'a> {
    pub fn new(type_engine: &'a TypeEngine, tokens: &'a TokenMap) -> Self {
        Self {
            type_engine,
            tokens,
        }
    }

    pub fn traverse_node(&self, node: &AstNode) {
        match &node.content {
            AstNodeContent::Declaration(declaration) => self.handle_declaration(declaration),
            AstNodeContent::Expression(expression)
            | AstNodeContent::ImplicitReturnExpression(expression) => {
                self.handle_expression(expression)
            }
            // TODO
            // handle other content types
            _ => {}
        };
    }

    fn handle_function_declation(&self, func: &FunctionDeclaration) {
        let token = Token::from_parsed(
            AstToken::FunctionDeclaration(func.clone()),
            SymbolKind::Function,
        );
        self.tokens.insert(to_ident_key(&func.name), token.clone());
        for node in &func.body.contents {
            self.traverse_node(node);
        }

        for parameter in &func.parameters {
            self.collect_function_parameter(parameter);
        }

        for type_param in &func.type_parameters {
            self.collect_type_parameter(type_param, AstToken::FunctionDeclaration(func.clone()));
        }

        self.collect_type_info_token(
            &token,
            &func.return_type,
            Some(func.return_type_span.clone()),
            None,
        );

        func.attributes.parse(self.tokens);
    }

    fn handle_declaration(&self, declaration: &Declaration) {
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
                    self.tokens.insert(
                        to_ident_key(&Ident::new(variable.name.span())),
                        token.clone(),
                    );

                    if let Some(type_ascription_span) = &variable.type_ascription_span {
                        self.collect_type_info_token(
                            &token,
                            &variable.type_ascription,
                            Some(type_ascription_span.clone()),
                            None,
                        );
                    }
                }
                self.handle_expression(&variable.body);
            }
            Declaration::FunctionDeclaration(func) => {
                self.handle_function_declation(func);
            }
            Declaration::TraitDeclaration(trait_decl) => {
                self.tokens.insert(
                    to_ident_key(&trait_decl.name),
                    Token::from_parsed(
                        AstToken::Declaration(declaration.clone()),
                        SymbolKind::Trait,
                    ),
                );

                for trait_fn in &trait_decl.interface_surface {
                    self.collect_trait_fn(trait_fn);
                }

                for func_dec in &trait_decl.methods {
                    self.handle_function_declation(func_dec);
                }

                for supertrait in &trait_decl.supertraits {
                    self.tokens.insert(
                        to_ident_key(&supertrait.name.suffix),
                        Token::from_parsed(
                            AstToken::Declaration(declaration.clone()),
                            SymbolKind::Trait,
                        ),
                    );
                }
            }
            Declaration::StructDeclaration(struct_dec) => {
                self.tokens.insert(
                    to_ident_key(&struct_dec.name),
                    Token::from_parsed(
                        AstToken::Declaration(declaration.clone()),
                        SymbolKind::Struct,
                    ),
                );
                for field in &struct_dec.fields {
                    let token =
                        Token::from_parsed(AstToken::StructField(field.clone()), SymbolKind::Field);
                    self.tokens.insert(to_ident_key(&field.name), token.clone());

                    self.collect_type_info_token(
                        &token,
                        &field.type_info,
                        Some(field.type_span.clone()),
                        None,
                    );

                    field.attributes.parse(self.tokens);
                }

                for type_param in &struct_dec.type_parameters {
                    self.collect_type_parameter(
                        type_param,
                        AstToken::Declaration(declaration.clone()),
                    );
                }

                struct_dec.attributes.parse(self.tokens);
            }
            Declaration::EnumDeclaration(enum_decl) => {
                self.tokens.insert(
                    to_ident_key(&enum_decl.name),
                    Token::from_parsed(
                        AstToken::Declaration(declaration.clone()),
                        SymbolKind::Enum,
                    ),
                );

                for type_param in &enum_decl.type_parameters {
                    self.collect_type_parameter(
                        type_param,
                        AstToken::Declaration(declaration.clone()),
                    );
                }

                for variant in &enum_decl.variants {
                    let token = Token::from_parsed(
                        AstToken::EnumVariant(variant.clone()),
                        SymbolKind::Variant,
                    );
                    self.tokens
                        .insert(to_ident_key(&variant.name), token.clone());

                    self.collect_type_info_token(
                        &token,
                        &variant.type_info,
                        Some(variant.type_span.clone()),
                        Some(SymbolKind::Variant),
                    );
                    variant.attributes.parse(self.tokens);
                }

                enum_decl.attributes.parse(self.tokens);
            }
            Declaration::ImplTrait(impl_trait) => {
                for ident in &impl_trait.trait_name.prefixes {
                    self.tokens.insert(
                        to_ident_key(ident),
                        Token::from_parsed(
                            AstToken::Declaration(declaration.clone()),
                            SymbolKind::Module,
                        ),
                    );
                }

                self.tokens.insert(
                    to_ident_key(&impl_trait.trait_name.suffix),
                    Token::from_parsed(
                        AstToken::Declaration(declaration.clone()),
                        SymbolKind::Trait,
                    ),
                );

                let token = Token::from_parsed(
                    AstToken::Declaration(declaration.clone()),
                    type_info_to_symbol_kind(self.type_engine, &impl_trait.type_implementing_for),
                );

                self.collect_type_info_token(
                    &token,
                    &impl_trait.type_implementing_for,
                    Some(impl_trait.type_implementing_for_span.clone()),
                    Some(SymbolKind::Variant),
                );

                for type_param in &impl_trait.impl_type_parameters {
                    self.collect_type_parameter(
                        type_param,
                        AstToken::Declaration(declaration.clone()),
                    );
                }

                for func_dec in &impl_trait.functions {
                    self.handle_function_declation(func_dec);
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
                    self.tokens.insert(to_ident_key(name), token.clone());
                    if let Some(type_arguments) = type_arguments {
                        for type_arg in type_arguments {
                            self.collect_type_arg(type_arg, &token);
                        }
                    }
                }

                for type_param in &impl_self.impl_type_parameters {
                    self.collect_type_parameter(
                        type_param,
                        AstToken::Declaration(declaration.clone()),
                    );
                }

                for func_dec in &impl_self.functions {
                    self.handle_function_declation(func_dec);
                }
            }
            Declaration::AbiDeclaration(abi_decl) => {
                self.tokens.insert(
                    to_ident_key(&abi_decl.name),
                    Token::from_parsed(
                        AstToken::Declaration(declaration.clone()),
                        SymbolKind::Trait,
                    ),
                );

                for trait_fn in &abi_decl.interface_surface {
                    self.collect_trait_fn(trait_fn);
                }

                abi_decl.attributes.parse(self.tokens);
            }
            Declaration::ConstantDeclaration(const_decl) => {
                let token = Token::from_parsed(
                    AstToken::Declaration(declaration.clone()),
                    SymbolKind::Const,
                );
                self.tokens
                    .insert(to_ident_key(&const_decl.name), token.clone());

                self.collect_type_info_token(
                    &token,
                    &const_decl.type_ascription,
                    const_decl.type_ascription_span.clone(),
                    None,
                );
                self.handle_expression(&const_decl.value);

                const_decl.attributes.parse(self.tokens);
            }
            Declaration::StorageDeclaration(storage_decl) => {
                for field in &storage_decl.fields {
                    let token = Token::from_parsed(
                        AstToken::StorageField(field.clone()),
                        SymbolKind::Field,
                    );
                    self.tokens.insert(to_ident_key(&field.name), token.clone());

                    self.collect_type_info_token(
                        &token,
                        &field.type_info,
                        Some(field.type_info_span.clone()),
                        None,
                    );
                    self.handle_expression(&field.initializer);

                    field.attributes.parse(self.tokens);
                }
                storage_decl.attributes.parse(self.tokens);
            }
        }
    }

    fn handle_expression(&self, expression: &Expression) {
        let span = &expression.span;
        match &expression.kind {
            ExpressionKind::Error(_part_spans) => {
                // FIXME(Centril): Left for @JoshuaBatty to use.
            }
            ExpressionKind::Literal(value) => {
                let symbol_kind = literal_to_symbol_kind(value);

                self.tokens.insert(
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
                        self.tokens.insert(
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

                    self.tokens
                        .insert(to_ident_key(&call_path_binding.inner.suffix), token.clone());

                    for type_arg in &call_path_binding.type_arguments {
                        self.collect_type_arg(type_arg, &token);
                    }
                }

                for exp in arguments {
                    self.handle_expression(exp);
                }
            }
            ExpressionKind::LazyOperator(LazyOperatorExpression { lhs, rhs, .. }) => {
                self.handle_expression(lhs);
                self.handle_expression(rhs);
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

                    self.tokens.insert(
                        to_ident_key(name),
                        Token::from_parsed(AstToken::Expression(expression.clone()), symbol_kind),
                    );
                }
            }
            ExpressionKind::Tuple(fields) => {
                for exp in fields {
                    self.handle_expression(exp);
                }
            }
            ExpressionKind::TupleIndex(TupleIndexExpression {
                prefix, index_span, ..
            }) => {
                self.handle_expression(prefix);

                self.tokens.insert(
                    to_ident_key(&Ident::new(index_span.clone())),
                    Token::from_parsed(
                        AstToken::Expression(expression.clone()),
                        SymbolKind::NumericLiteral,
                    ),
                );
            }
            ExpressionKind::Array(array_expression) => {
                for exp in &array_expression.contents {
                    self.handle_expression(exp);
                }

                if let Some(length_span) = &array_expression.length_span {
                    self.tokens.insert(
                        to_ident_key(&Ident::new(length_span.clone())),
                        Token::from_parsed(
                            AstToken::Expression(expression.clone()),
                            SymbolKind::NumericLiteral,
                        ),
                    );
                }
            }
            ExpressionKind::Struct(struct_expression) => {
                let StructExpression {
                    call_path_binding,
                    fields,
                } = &**struct_expression;
                for ident in &call_path_binding.inner.prefixes {
                    self.tokens.insert(
                        to_ident_key(ident),
                        Token::from_parsed(
                            AstToken::Expression(expression.clone()),
                            SymbolKind::Struct,
                        ),
                    );
                }

                let name = &call_path_binding.inner.suffix;
                let type_arguments = &call_path_binding.type_arguments;

                let token = Token::from_parsed(
                    AstToken::Expression(expression.clone()),
                    SymbolKind::Struct,
                );
                self.tokens.insert(to_ident_key(name), token.clone());
                for type_arg in type_arguments {
                    self.collect_type_arg(type_arg, &token);
                }

                for field in fields {
                    self.tokens.insert(
                        to_ident_key(&field.name),
                        Token::from_parsed(
                            AstToken::StructExpressionField(field.clone()),
                            SymbolKind::Field,
                        ),
                    );
                    self.handle_expression(&field.value);
                }
            }
            ExpressionKind::CodeBlock(contents) => {
                for node in &contents.contents {
                    self.traverse_node(node);
                }
            }
            ExpressionKind::If(IfExpression {
                condition,
                then,
                r#else,
                ..
            }) => {
                self.handle_expression(condition);
                self.handle_expression(then);
                if let Some(r#else) = r#else {
                    self.handle_expression(r#else);
                }
            }
            ExpressionKind::Match(MatchExpression {
                value, branches, ..
            }) => {
                self.handle_expression(value);
                for branch in branches {
                    self.collect_scrutinee(&branch.scrutinee);
                    self.handle_expression(&branch.result);
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
                    let (type_info, ident) = &call_path_binding.inner.suffix;
                    self.collect_type_info_token(&token, type_info, Some(ident.span()), None);
                }

                let token = Token::from_parsed(
                    AstToken::Expression(expression.clone()),
                    SymbolKind::Struct,
                );

                for type_arg in &method_name_binding.type_arguments {
                    self.collect_type_arg(type_arg, &token);
                }

                // Don't collect applications of desugared operators due to mismatched ident lengths.
                if !desugared_op(&prefixes) {
                    self.tokens
                        .insert(to_ident_key(&method_name_binding.inner.easy_name()), token);
                }

                for exp in arguments {
                    self.handle_expression(exp);
                }

                for field in contract_call_params {
                    self.tokens.insert(
                        to_ident_key(&field.name),
                        Token::from_parsed(
                            AstToken::Expression(field.value.clone()),
                            SymbolKind::Field,
                        ),
                    );
                    self.handle_expression(&field.value);
                }
            }
            ExpressionKind::Subfield(SubfieldExpression {
                prefix,
                field_to_access,
                ..
            }) => {
                self.tokens.insert(
                    to_ident_key(field_to_access),
                    Token::from_parsed(AstToken::Expression(expression.clone()), SymbolKind::Field),
                );
                self.handle_expression(prefix);
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
                    self.tokens.insert(
                        to_ident_key(ident),
                        Token::from_parsed(
                            AstToken::Expression(expression.clone()),
                            SymbolKind::Enum,
                        ),
                    );
                }

                let token = Token::from_parsed(
                    AstToken::Expression(expression.clone()),
                    SymbolKind::Variant,
                );

                self.tokens.insert(
                    to_ident_key(&call_path_binding.inner.suffix.suffix),
                    token.clone(),
                );

                for type_arg in &call_path_binding.type_arguments {
                    self.collect_type_arg(type_arg, &token);
                }

                for exp in args {
                    self.handle_expression(exp);
                }
            }
            ExpressionKind::DelineatedPath(delineated_path_expression) => {
                let DelineatedPathExpression {
                    call_path_binding,
                    args,
                } = &**delineated_path_expression;
                for ident in &call_path_binding.inner.prefixes {
                    self.tokens.insert(
                        to_ident_key(ident),
                        Token::from_parsed(
                            AstToken::Expression(expression.clone()),
                            SymbolKind::Enum,
                        ),
                    );
                }

                let token = Token::from_parsed(
                    AstToken::Expression(expression.clone()),
                    SymbolKind::Variant,
                );

                self.tokens
                    .insert(to_ident_key(&call_path_binding.inner.suffix), token.clone());

                for type_arg in &call_path_binding.type_arguments {
                    self.collect_type_arg(type_arg, &token);
                }

                if let Some(args_vec) = args.as_ref() {
                    args_vec.iter().for_each(|exp| {
                        self.handle_expression(exp);
                    });
                }
            }
            ExpressionKind::AbiCast(abi_cast_expression) => {
                let AbiCastExpression { abi_name, address } = &**abi_cast_expression;
                for ident in &abi_name.prefixes {
                    self.tokens.insert(
                        to_ident_key(ident),
                        Token::from_parsed(
                            AstToken::Expression(expression.clone()),
                            SymbolKind::Module,
                        ),
                    );
                }
                self.tokens.insert(
                    to_ident_key(&abi_name.suffix),
                    Token::from_parsed(AstToken::Expression(expression.clone()), SymbolKind::Trait),
                );
                self.handle_expression(address);
            }
            ExpressionKind::ArrayIndex(ArrayIndexExpression { prefix, index, .. }) => {
                self.handle_expression(prefix);
                self.handle_expression(index);
            }
            ExpressionKind::StorageAccess(StorageAccessExpression { field_names, .. }) => {
                for field in field_names {
                    self.tokens.insert(
                        to_ident_key(field),
                        Token::from_parsed(
                            AstToken::Expression(expression.clone()),
                            SymbolKind::Field,
                        ),
                    );
                }
            }
            ExpressionKind::IntrinsicFunction(IntrinsicFunctionExpression {
                name,
                kind_binding,
                arguments,
            }) => {
                self.tokens.insert(
                    to_ident_key(name),
                    Token::from_parsed(
                        AstToken::Intrinsic(kind_binding.inner.clone()),
                        SymbolKind::Function,
                    ),
                );

                for argument in arguments {
                    self.handle_expression(argument);
                }
            }
            ExpressionKind::WhileLoop(WhileLoopExpression {
                body, condition, ..
            }) => self.handle_while_loop(body, condition),
            // TODO: collect these tokens as keywords once the compiler returns the span
            ExpressionKind::Break | ExpressionKind::Continue => {}
            ExpressionKind::Reassignment(reassignment) => {
                self.handle_expression(&reassignment.rhs);

                match &reassignment.lhs {
                    ReassignmentTarget::VariableExpression(exp) => {
                        self.handle_expression(exp);
                    }
                    ReassignmentTarget::StorageField(idents) => {
                        for ident in idents {
                            self.tokens.insert(
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
            ExpressionKind::Return(expr) => self.handle_expression(expr),
        }
    }

    fn handle_while_loop(&self, body: &CodeBlock, condition: &Expression) {
        self.handle_expression(condition);
        for node in &body.contents {
            self.traverse_node(node);
        }
    }

    fn collect_type_arg(&self, type_argument: &TypeArgument, token: &Token) {
        let mut token = token.clone();
        let type_info = self.type_engine.get(type_argument.type_id);
        match &type_info {
            TypeInfo::Array(type_arg, length) => {
                token.kind = SymbolKind::NumericLiteral;
                self.tokens
                    .insert(to_ident_key(&Ident::new(length.span())), token.clone());
                self.collect_type_arg(type_arg, &token);
            }
            TypeInfo::Tuple(type_arguments) => {
                for type_arg in type_arguments {
                    self.collect_type_arg(type_arg, &token);
                }
            }
            TypeInfo::Custom {
                name,
                type_arguments,
            } => {
                if let Some(type_args) = type_arguments {
                    for type_arg in type_args {
                        self.collect_type_arg(type_arg, &token);
                    }
                }

                let symbol_kind = type_info_to_symbol_kind(self.type_engine, &type_info);
                token.kind = symbol_kind;
                token.type_def = Some(TypeDefinition::TypeId(type_argument.type_id));
                self.tokens
                    .insert(to_ident_key(&Ident::new(name.span())), token);
            }
            _ => {
                let symbol_kind = type_info_to_symbol_kind(self.type_engine, &type_info);
                token.kind = symbol_kind;
                token.type_def = Some(TypeDefinition::TypeId(type_argument.type_id));
                self.tokens
                    .insert(to_ident_key(&Ident::new(type_argument.span.clone())), token);
            }
        }
    }

    fn collect_scrutinee(&self, scrutinee: &Scrutinee) {
        match scrutinee {
            Scrutinee::CatchAll { .. } => (),
            Scrutinee::Literal { ref value, span } => {
                let token = Token::from_parsed(
                    AstToken::Scrutinee(scrutinee.clone()),
                    literal_to_symbol_kind(value),
                );
                self.tokens
                    .insert(to_ident_key(&Ident::new(span.clone())), token);
            }
            Scrutinee::Variable { name, .. } => {
                let token = Token::from_parsed(
                    AstToken::Scrutinee(scrutinee.clone()),
                    SymbolKind::Variable,
                );
                self.tokens.insert(to_ident_key(name), token);
            }
            Scrutinee::StructScrutinee {
                struct_name,
                fields,
                ..
            } => {
                let token =
                    Token::from_parsed(AstToken::Scrutinee(scrutinee.clone()), SymbolKind::Struct);
                self.tokens.insert(to_ident_key(&struct_name.suffix), token);

                for field in fields {
                    let token = Token::from_parsed(
                        AstToken::Scrutinee(scrutinee.clone()),
                        SymbolKind::Field,
                    );
                    if let StructScrutineeField::Field {
                        field, scrutinee, ..
                    } = field
                    {
                        self.tokens.insert(to_ident_key(field), token);

                        if let Some(scrutinee) = scrutinee {
                            self.collect_scrutinee(scrutinee);
                        }
                    }
                }
            }
            Scrutinee::EnumScrutinee {
                call_path, value, ..
            } => {
                for ident in &call_path.prefixes {
                    let token = Token::from_parsed(
                        AstToken::Scrutinee(scrutinee.clone()),
                        SymbolKind::Enum,
                    );
                    self.tokens.insert(to_ident_key(ident), token);
                }

                let token =
                    Token::from_parsed(AstToken::Scrutinee(scrutinee.clone()), SymbolKind::Variant);
                self.tokens.insert(to_ident_key(&call_path.suffix), token);

                self.collect_scrutinee(value);
            }
            Scrutinee::Tuple { elems, .. } => {
                for elem in elems {
                    self.collect_scrutinee(elem);
                }
            }
            Scrutinee::Error { .. } => {
                // FIXME: Left for @JoshuaBatty to use.
            }
        }
    }

    fn collect_type_info_token(
        &self,
        token: &Token,
        type_info: &TypeInfo,
        type_span: Option<Span>,
        symbol_kind: Option<SymbolKind>,
    ) {
        let mut token = token.clone();
        match symbol_kind {
            Some(kind) => token.kind = kind,
            None => token.kind = type_info_to_symbol_kind(self.type_engine, type_info),
        }

        match type_info {
            TypeInfo::Str(length) => {
                self.tokens
                    .insert(to_ident_key(&Ident::new(length.span())), token);
            }
            TypeInfo::Array(type_arg, length) => {
                token.kind = SymbolKind::NumericLiteral;
                self.tokens
                    .insert(to_ident_key(&Ident::new(length.span())), token.clone());
                self.collect_type_arg(type_arg, &token);
            }
            TypeInfo::Tuple(type_arguments) => {
                for type_arg in type_arguments {
                    self.collect_type_arg(type_arg, &token);
                }
            }
            TypeInfo::Custom {
                name,
                type_arguments,
            } => {
                token.type_def = Some(TypeDefinition::Ident(name.clone()));
                self.tokens.insert(to_ident_key(name), token.clone());
                if let Some(type_arguments) = type_arguments {
                    for type_arg in type_arguments {
                        self.collect_type_arg(type_arg, &token);
                    }
                }
            }
            _ => {
                if let Some(type_span) = type_span {
                    self.tokens
                        .insert(to_ident_key(&Ident::new(type_span)), token);
                }
            }
        }
    }

    fn collect_function_parameter(&self, parameter: &FunctionParameter) {
        let token = Token::from_parsed(
            AstToken::FunctionParameter(parameter.clone()),
            SymbolKind::ValueParam,
        );
        self.tokens
            .insert(to_ident_key(&parameter.name), token.clone());

        self.collect_type_info_token(
            &token,
            &parameter.type_info,
            Some(parameter.type_span.clone()),
            None,
        );
    }

    fn collect_trait_fn(&self, trait_fn: &TraitFn) {
        let token = Token::from_parsed(AstToken::TraitFn(trait_fn.clone()), SymbolKind::Function);
        self.tokens
            .insert(to_ident_key(&trait_fn.name), token.clone());

        for parameter in &trait_fn.parameters {
            self.collect_function_parameter(parameter);
        }

        self.collect_type_info_token(
            &token,
            &trait_fn.return_type,
            Some(trait_fn.return_type_span.clone()),
            None,
        );

        trait_fn.attributes.parse(self.tokens);
    }

    fn collect_type_parameter(&self, type_param: &TypeParameter, token: AstToken) {
        self.tokens.insert(
            to_ident_key(&type_param.name_ident),
            Token::from_parsed(token, SymbolKind::TypeParameter),
        );
    }
}

impl Parse for AttributesMap {
    fn parse(&self, tokens: &TokenMap) {
        self.iter()
            .filter(|(kind, ..)| **kind != AttributeKind::DocComment)
            .flat_map(|(.., attrs)| attrs)
            .for_each(|attribute| {
                tokens.insert(
                    to_ident_key(&attribute.name),
                    Token::from_parsed(
                        AstToken::Attribute(attribute.clone()),
                        SymbolKind::DeriveHelper,
                    ),
                );
            });
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
