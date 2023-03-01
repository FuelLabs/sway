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
            AstNodeContent, CodeBlock, ConstantDeclaration, Declaration, DelineatedPathExpression,
            Expression, ExpressionKind, FunctionApplicationExpression, FunctionDeclaration,
            FunctionParameter, IfExpression, ImplItem, ImportType, IntrinsicFunctionExpression,
            LazyOperatorExpression, MatchExpression, MethodApplicationExpression, MethodName,
            ParseModule, ParseProgram, ParseSubmodule, ReassignmentTarget, Scrutinee,
            StorageAccessExpression, StructExpression, StructScrutineeField, SubfieldExpression,
            TraitFn, TraitItem, TreeType, TupleIndexExpression, UseStatement, WhileLoopExpression,
        },
        CallPathTree, Literal,
    },
    transform::{AttributeKind, AttributesMap},
    type_system::{TypeArgument, TypeParameter},
    TraitConstraint, TypeEngine, TypeInfo,
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
            AstNodeContent::UseStatement(use_statement) => self.handle_use_statement(use_statement),
            // include statements are handled throught [`collect_module_spans`]
            AstNodeContent::IncludeStatement(_) => {}
        };
    }

    /// Collects the library name and the module name from the dep statement
    pub fn collect_module_spans(&self, parse_program: &ParseProgram) {
        if let TreeType::Library { name } = &parse_program.kind {
            self.tokens.insert(
                to_ident_key(name),
                Token::from_parsed(AstToken::LibraryName(name.clone()), SymbolKind::Module),
            );
        }
        self.collect_parse_module(&parse_program.root);
    }

    fn collect_parse_module(&self, parse_module: &ParseModule) {
        for (
            _,
            ParseSubmodule {
                library_name,
                module,
                dependency_path_span,
            },
        ) in &parse_module.submodules
        {
            self.tokens.insert(
                to_ident_key(&Ident::new(dependency_path_span.clone())),
                Token::from_parsed(AstToken::IncludeStatement, SymbolKind::Module),
            );
            self.tokens.insert(
                to_ident_key(library_name),
                Token::from_parsed(
                    AstToken::LibraryName(library_name.clone()),
                    SymbolKind::Module,
                ),
            );
            self.collect_parse_module(module);
        }
    }

    fn handle_const_declaration(&self, const_decl: &ConstantDeclaration) {
        self.tokens.insert(
            to_ident_key(&const_decl.name),
            Token::from_parsed(
                AstToken::ConstantDeclaration(const_decl.clone()),
                SymbolKind::Const,
            ),
        );

        self.collect_type_arg(&const_decl.type_ascription);
        if let Some(value) = &const_decl.value {
            self.handle_expression(value);
        }

        const_decl.attributes.parse(self.tokens);
    }

    fn handle_function_declaration(&self, func: &FunctionDeclaration) {
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
            self.collect_type_parameter(type_param);
        }

        for (ident, constraints) in &func.where_clause {
            self.tokens.insert(to_ident_key(ident), token.clone());
            for constraint in constraints {
                self.collect_trait_constraint(constraint);
            }
        }

        self.collect_type_arg(&func.return_type);

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

                    // We want to use the span from variable.name to construct a
                    // new Ident as the name_override_opt can be set to one of the
                    // const prefixes and not the actual token name.
                    self.tokens.insert(
                        to_ident_key(&Ident::new(variable.name.span())),
                        Token::from_parsed(AstToken::Declaration(declaration.clone()), symbol_kind),
                    );

                    self.collect_type_arg(&variable.type_ascription);
                }
                self.handle_expression(&variable.body);
            }
            Declaration::FunctionDeclaration(func) => {
                self.handle_function_declaration(func);
            }
            Declaration::TraitDeclaration(trait_decl) => {
                self.tokens.insert(
                    to_ident_key(&trait_decl.name),
                    Token::from_parsed(
                        AstToken::Declaration(declaration.clone()),
                        SymbolKind::Trait,
                    ),
                );

                for item in &trait_decl.interface_surface {
                    match item {
                        TraitItem::TraitFn(trait_fn) => self.collect_trait_fn(trait_fn),
                    }
                }

                for func_dec in &trait_decl.methods {
                    self.handle_function_declaration(func_dec);
                }

                for supertrait in &trait_decl.supertraits {
                    self.tokens.insert(
                        to_ident_key(&supertrait.name.suffix),
                        Token::from_parsed(
                            AstToken::Supertrait(supertrait.clone()),
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
                    self.tokens.insert(
                        to_ident_key(&field.name),
                        Token::from_parsed(AstToken::StructField(field.clone()), SymbolKind::Field),
                    );

                    self.collect_type_arg(&field.type_argument);
                    field.attributes.parse(self.tokens);
                }

                for type_param in &struct_dec.type_parameters {
                    self.collect_type_parameter(type_param);
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
                    self.collect_type_parameter(type_param);
                }

                for variant in &enum_decl.variants {
                    self.tokens.insert(
                        to_ident_key(&variant.name),
                        Token::from_parsed(
                            AstToken::EnumVariant(variant.clone()),
                            SymbolKind::Variant,
                        ),
                    );

                    self.collect_type_arg(&variant.type_argument);
                    variant.attributes.parse(self.tokens);
                }

                enum_decl.attributes.parse(self.tokens);
            }
            Declaration::ImplTrait(impl_trait) => {
                for ident in &impl_trait.trait_name.prefixes {
                    self.tokens.insert(
                        to_ident_key(ident),
                        Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Module),
                    );
                }

                self.tokens.insert(
                    to_ident_key(&impl_trait.trait_name.suffix),
                    Token::from_parsed(
                        AstToken::Declaration(declaration.clone()),
                        SymbolKind::Trait,
                    ),
                );

                self.collect_type_arg(&impl_trait.implementing_for);

                for type_param in &impl_trait.impl_type_parameters {
                    self.collect_type_parameter(type_param);
                }

                for item in &impl_trait.items {
                    match item {
                        ImplItem::Fn(fn_decl) => self.handle_function_declaration(fn_decl),
                    }
                }
            }
            Declaration::ImplSelf(impl_self) => {
                if let TypeInfo::Custom {
                    call_path,
                    type_arguments,
                } = &self.type_engine.get(impl_self.implementing_for.type_id)
                {
                    self.tokens.insert(
                        to_ident_key(&call_path.suffix),
                        Token::from_parsed(
                            AstToken::Declaration(declaration.clone()),
                            SymbolKind::Struct,
                        ),
                    );
                    if let Some(type_arguments) = type_arguments {
                        for type_arg in type_arguments {
                            self.collect_type_arg(type_arg);
                        }
                    }
                }

                for type_param in &impl_self.impl_type_parameters {
                    self.collect_type_parameter(type_param);
                }

                for item in &impl_self.items {
                    match item {
                        ImplItem::Fn(fn_decl) => self.handle_function_declaration(fn_decl),
                    }
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

                for item in &abi_decl.interface_surface {
                    match item {
                        TraitItem::TraitFn(trait_fn) => self.collect_trait_fn(trait_fn),
                    }
                }

                for supertrait in &abi_decl.supertraits {
                    self.tokens.insert(
                        to_ident_key(&supertrait.name.suffix),
                        Token::from_parsed(
                            AstToken::Supertrait(supertrait.clone()),
                            SymbolKind::Trait,
                        ),
                    );
                }

                abi_decl.attributes.parse(self.tokens);
            }
            Declaration::ConstantDeclaration(const_decl) => {
                self.handle_const_declaration(const_decl);
            }
            Declaration::StorageDeclaration(storage_decl) => {
                for field in &storage_decl.fields {
                    self.tokens.insert(
                        to_ident_key(&field.name),
                        Token::from_parsed(
                            AstToken::StorageField(field.clone()),
                            SymbolKind::Field,
                        ),
                    );

                    self.collect_type_arg(&field.type_argument);
                    self.handle_expression(&field.initializer);

                    field.attributes.parse(self.tokens);
                }
                storage_decl.attributes.parse(self.tokens);
            }
        }
    }

    fn handle_use_statement(
        &self,
        use_statement @ UseStatement {
            alias,
            call_path,
            is_absolute: _,
            import_type,
        }: &UseStatement,
    ) {
        if let Some(alias) = alias {
            self.tokens.insert(
                to_ident_key(alias),
                Token::from_parsed(
                    AstToken::UseStatement(use_statement.clone()),
                    SymbolKind::Unknown,
                ),
            );
        }

        for prefix in call_path {
            self.tokens.insert(
                to_ident_key(prefix),
                Token::from_parsed(
                    AstToken::UseStatement(use_statement.clone()),
                    SymbolKind::Module,
                ),
            );
        }

        match &import_type {
            ImportType::Item(item) => {
                self.tokens.insert(
                    to_ident_key(item),
                    Token::from_parsed(
                        AstToken::UseStatement(use_statement.clone()),
                        SymbolKind::Unknown,
                    ),
                );
            }
            ImportType::SelfImport(span) => {
                self.tokens.insert(
                    to_ident_key(&Ident::new(span.clone())),
                    Token::from_parsed(
                        AstToken::UseStatement(use_statement.clone()),
                        SymbolKind::Unknown,
                    ),
                );
            }
            ImportType::Star => {}
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
                            Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Module),
                        );
                    }

                    self.tokens.insert(
                        to_ident_key(&call_path_binding.inner.suffix),
                        Token::from_parsed(
                            AstToken::Expression(expression.clone()),
                            SymbolKind::Function,
                        ),
                    );

                    for type_arg in &call_path_binding.type_arguments.to_vec() {
                        self.collect_type_arg(type_arg);
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
                    let ident = Ident::new(length_span.clone());
                    self.tokens.insert(
                        to_ident_key(&ident),
                        Token::from_parsed(
                            AstToken::Ident(ident.clone()),
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
                        Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Struct),
                    );
                }

                let name = &call_path_binding.inner.suffix;
                let type_arguments = &call_path_binding.type_arguments.to_vec();

                self.tokens.insert(
                    to_ident_key(name),
                    Token::from_parsed(
                        AstToken::Expression(expression.clone()),
                        SymbolKind::Struct,
                    ),
                );
                for type_arg in type_arguments {
                    self.collect_type_arg(type_arg);
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
            ExpressionKind::Asm(asm) => {
                for register in &asm.registers {
                    if let Some(initializer) = &register.initializer {
                        self.handle_expression(initializer);
                    }
                }
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
                    let (type_info, ident) = &call_path_binding.inner.suffix;
                    self.collect_type_info_token(type_info, Some(ident.span()));
                }

                for type_arg in &method_name_binding.type_arguments.to_vec() {
                    self.collect_type_arg(type_arg);
                }

                // Don't collect applications of desugared operators due to mismatched ident lengths.
                if !desugared_op(&prefixes) {
                    self.tokens.insert(
                        to_ident_key(&method_name_binding.inner.easy_name()),
                        Token::from_parsed(
                            AstToken::Expression(expression.clone()),
                            SymbolKind::Struct,
                        ),
                    );
                }

                for exp in arguments {
                    self.handle_expression(exp);
                }

                for field in contract_call_params {
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
                        Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Enum),
                    );
                }

                self.tokens.insert(
                    to_ident_key(&call_path_binding.inner.suffix.suffix),
                    Token::from_parsed(
                        AstToken::Expression(expression.clone()),
                        SymbolKind::Variant,
                    ),
                );

                for type_arg in &call_path_binding.type_arguments.to_vec() {
                    self.collect_type_arg(type_arg);
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

                self.tokens.insert(
                    to_ident_key(&call_path_binding.inner.suffix),
                    Token::from_parsed(
                        AstToken::Expression(expression.clone()),
                        SymbolKind::Variant,
                    ),
                );

                for type_arg in &call_path_binding.type_arguments.to_vec() {
                    self.collect_type_arg(type_arg);
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
                        Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Module),
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
            ExpressionKind::StorageAccess(StorageAccessExpression { field_names }) => {
                for field in field_names {
                    self.tokens.insert(
                        to_ident_key(field),
                        Token::from_parsed(AstToken::Ident(field.clone()), SymbolKind::Field),
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

                for type_arg in &kind_binding.type_arguments.to_vec() {
                    self.collect_type_arg(type_arg);
                }
            }
            ExpressionKind::WhileLoop(WhileLoopExpression {
                body, condition, ..
            }) => self.handle_while_loop(body, condition),
            // We are collecting these tokens in the lexed phase.
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
                                    AstToken::Ident(ident.clone()),
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

    fn collect_trait_constraint(&self, constraint: &TraitConstraint) {
        for prefix in &constraint.trait_name.prefixes {
            self.tokens.insert(
                to_ident_key(prefix),
                Token::from_parsed(AstToken::Ident(prefix.clone()), SymbolKind::Function),
            );
        }
        self.tokens.insert(
            to_ident_key(&constraint.trait_name.suffix),
            Token::from_parsed(
                AstToken::TraitConstraint(constraint.clone()),
                SymbolKind::Function,
            ),
        );
        for type_arg in &constraint.type_arguments {
            self.collect_type_arg(type_arg)
        }
    }

    fn collect_type_arg(&self, type_argument: &TypeArgument) {
        let type_info = self.type_engine.get(type_argument.type_id);
        match &type_info {
            TypeInfo::Array(type_arg, length) => {
                let ident = Ident::new(length.span());
                self.tokens.insert(
                    to_ident_key(&ident),
                    Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::NumericLiteral),
                );
                self.collect_type_arg(type_arg);
            }
            TypeInfo::Tuple(type_arguments) => {
                for type_arg in type_arguments {
                    self.collect_type_arg(type_arg);
                }
            }
            _ => {
                let symbol_kind = type_info_to_symbol_kind(self.type_engine, &type_info);
                let token =
                    Token::from_parsed(AstToken::TypeArgument(type_argument.clone()), symbol_kind);

                if let Some(tree) = &type_argument.call_path_tree {
                    self.collect_call_path_tree(tree, &token);
                }
            }
        }
    }

    fn collect_call_path_tree(&self, tree: &CallPathTree, token: &Token) {
        for ident in &tree.call_path.prefixes {
            self.tokens.insert(to_ident_key(ident), token.clone());
        }
        self.tokens
            .insert(to_ident_key(&tree.call_path.suffix), token.clone());

        for child in &tree.children {
            self.collect_call_path_tree(child, token);
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
                self.tokens.insert(
                    to_ident_key(name),
                    // it could either be a variable or a constant
                    Token::from_parsed(AstToken::Scrutinee(scrutinee.clone()), SymbolKind::Unknown),
                );
            }
            Scrutinee::StructScrutinee {
                struct_name,
                fields,
                ..
            } => {
                for ident in &struct_name.prefixes {
                    let token =
                        Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Struct);
                    self.tokens.insert(to_ident_key(ident), token);
                }
                self.tokens.insert(
                    to_ident_key(&struct_name.suffix),
                    Token::from_parsed(AstToken::Scrutinee(scrutinee.clone()), SymbolKind::Struct),
                );

                for field in fields {
                    let token = Token::from_parsed(
                        AstToken::StructScrutineeField(field.clone()),
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
                    self.tokens.insert(
                        to_ident_key(ident),
                        Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Enum),
                    );
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

    fn collect_type_info_token(&self, type_info: &TypeInfo, type_span: Option<Span>) {
        let symbol_kind = type_info_to_symbol_kind(self.type_engine, type_info);
        match type_info {
            TypeInfo::Str(length) => {
                let ident = Ident::new(length.span());
                self.tokens.insert(
                    to_ident_key(&ident),
                    Token::from_parsed(AstToken::Ident(ident.clone()), symbol_kind),
                );
            }
            TypeInfo::Array(type_arg, length) => {
                let ident = Ident::new(length.span());
                self.tokens.insert(
                    to_ident_key(&ident),
                    Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::NumericLiteral),
                );
                self.collect_type_arg(type_arg);
            }
            TypeInfo::Tuple(type_arguments) => {
                for type_arg in type_arguments {
                    self.collect_type_arg(type_arg);
                }
            }
            TypeInfo::Custom {
                call_path,
                type_arguments,
            } => {
                let ident = call_path.suffix.clone();
                let mut token = Token::from_parsed(AstToken::Ident(ident.clone()), symbol_kind);
                token.type_def = Some(TypeDefinition::Ident(ident.clone()));
                self.tokens.insert(to_ident_key(&ident), token);
                if let Some(type_arguments) = type_arguments {
                    for type_arg in type_arguments {
                        self.collect_type_arg(type_arg);
                    }
                }
            }
            _ => {
                if let Some(type_span) = type_span {
                    let ident = Ident::new(type_span);
                    self.tokens.insert(
                        to_ident_key(&ident),
                        Token::from_parsed(AstToken::Ident(ident.clone()), symbol_kind),
                    );
                }
            }
        }
    }

    fn collect_function_parameter(&self, parameter: &FunctionParameter) {
        self.tokens.insert(
            to_ident_key(&parameter.name),
            Token::from_parsed(
                AstToken::FunctionParameter(parameter.clone()),
                SymbolKind::ValueParam,
            ),
        );
        self.collect_type_arg(&parameter.type_argument);
    }

    fn collect_trait_fn(&self, trait_fn: &TraitFn) {
        self.tokens.insert(
            to_ident_key(&trait_fn.name),
            Token::from_parsed(AstToken::TraitFn(trait_fn.clone()), SymbolKind::Function),
        );

        for parameter in &trait_fn.parameters {
            self.collect_function_parameter(parameter);
        }

        self.collect_type_info_token(
            &trait_fn.return_type,
            Some(trait_fn.return_type_span.clone()),
        );

        trait_fn.attributes.parse(self.tokens);
    }

    fn collect_type_parameter(&self, type_param: &TypeParameter) {
        self.tokens.insert(
            to_ident_key(&type_param.name_ident),
            Token::from_parsed(
                AstToken::TypeParameter(type_param.clone()),
                SymbolKind::TypeParameter,
            ),
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
