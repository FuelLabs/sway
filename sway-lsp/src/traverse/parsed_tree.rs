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
    traverse::{Parse, ParseContext},
};

use sway_core::{
    language::{
        parsed::{
            AbiCastExpression, AbiDeclaration, AmbiguousPathExpression, ArrayExpression,
            ArrayIndexExpression, AstNode, AstNodeContent, CodeBlock, ConstantDeclaration,
            Declaration, DelineatedPathExpression, EnumDeclaration, EnumVariant, Expression,
            ExpressionKind, FunctionApplicationExpression, FunctionDeclaration, FunctionParameter,
            IfExpression, ImplItem, ImplSelf, ImplTrait, ImportType, IntrinsicFunctionExpression,
            LazyOperatorExpression, MatchExpression, MethodApplicationExpression, MethodName,
            ParseModule, ParseProgram, ParseSubmodule, ReassignmentExpression, ReassignmentTarget,
            Scrutinee, StorageAccessExpression, StorageDeclaration, StorageField,
            StructDeclaration, StructExpression, StructExpressionField, StructField,
            StructScrutineeField, SubfieldExpression, Supertrait, TraitDeclaration, TraitFn,
            TraitItem, TreeType, TupleIndexExpression, UseStatement, VariableDeclaration,
            WhileLoopExpression,
        },
        CallPathTree, Literal,
    },
    transform::{AttributeKind, AttributesMap},
    type_system::{TypeArgument, TypeParameter},
    TraitConstraint, TypeInfo,
};
use sway_types::constants::{DESTRUCTURE_PREFIX, MATCH_RETURN_VAR_NAME_PREFIX, TUPLE_NAME_PREFIX};
use sway_types::{Ident, Span, Spanned};

pub struct ParsedTree<'a> {
    ctx: &'a ParseContext<'a>,
}

impl<'a> ParsedTree<'a> {
    pub fn new(ctx: &'a ParseContext<'a>) -> Self {
        Self { ctx }
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
            self.ctx.tokens.insert(
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
                ..
            },
        ) in &parse_module.submodules
        {
            self.ctx.tokens.insert(
                to_ident_key(&Ident::new(dependency_path_span.clone())),
                Token::from_parsed(AstToken::IncludeStatement, SymbolKind::Module),
            );
            self.ctx.tokens.insert(
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
        self.ctx.tokens.insert(
            to_ident_key(&const_decl.name),
            Token::from_parsed(
                AstToken::Declaration(Declaration::ConstantDeclaration(const_decl.clone())),
                SymbolKind::Const,
            ),
        );

        self.collect_type_arg(&const_decl.type_ascription);
        if let Some(value) = &const_decl.value {
            self.handle_expression(value);
        }

        const_decl.attributes.parse(&self.ctx);
    }

    fn handle_function_declaration(&self, func: &FunctionDeclaration) {
        let token = Token::from_parsed(
            AstToken::FunctionDeclaration(func.clone()),
            SymbolKind::Function,
        );
        self.ctx
            .tokens
            .insert(to_ident_key(&func.name), token.clone());
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
            self.ctx.tokens.insert(to_ident_key(ident), token.clone());
            for constraint in constraints {
                self.collect_trait_constraint(constraint);
            }
        }

        self.collect_type_arg(&func.return_type);

        func.attributes.parse(&self.ctx);
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
                    self.ctx.tokens.insert(
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
                self.ctx.tokens.insert(
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
                    self.ctx.tokens.insert(
                        to_ident_key(&supertrait.name.suffix),
                        Token::from_parsed(
                            AstToken::Supertrait(supertrait.clone()),
                            SymbolKind::Trait,
                        ),
                    );
                }
            }
            Declaration::StructDeclaration(struct_dec) => {
                self.ctx.tokens.insert(
                    to_ident_key(&struct_dec.name),
                    Token::from_parsed(
                        AstToken::Declaration(declaration.clone()),
                        SymbolKind::Struct,
                    ),
                );
                for field in &struct_dec.fields {
                    self.ctx.tokens.insert(
                        to_ident_key(&field.name),
                        Token::from_parsed(AstToken::StructField(field.clone()), SymbolKind::Field),
                    );

                    self.collect_type_arg(&field.type_argument);
                    field.attributes.parse(&self.ctx);
                }

                for type_param in &struct_dec.type_parameters {
                    self.collect_type_parameter(type_param);
                }

                struct_dec.attributes.parse(&self.ctx);
            }
            Declaration::EnumDeclaration(enum_decl) => {
                self.ctx.tokens.insert(
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
                    self.ctx.tokens.insert(
                        to_ident_key(&variant.name),
                        Token::from_parsed(
                            AstToken::EnumVariant(variant.clone()),
                            SymbolKind::Variant,
                        ),
                    );

                    self.collect_type_arg(&variant.type_argument);
                    variant.attributes.parse(&self.ctx);
                }

                enum_decl.attributes.parse(&self.ctx);
            }
            Declaration::ImplTrait(impl_trait) => {
                for ident in &impl_trait.trait_name.prefixes {
                    self.ctx.tokens.insert(
                        to_ident_key(ident),
                        Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Module),
                    );
                }

                self.ctx.tokens.insert(
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
                } = &self
                    .ctx
                    .engines
                    .te()
                    .get(impl_self.implementing_for.type_id)
                {
                    self.ctx.tokens.insert(
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
                self.ctx.tokens.insert(
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
                    self.ctx.tokens.insert(
                        to_ident_key(&supertrait.name.suffix),
                        Token::from_parsed(
                            AstToken::Supertrait(supertrait.clone()),
                            SymbolKind::Trait,
                        ),
                    );
                }

                abi_decl.attributes.parse(&self.ctx);
            }
            Declaration::ConstantDeclaration(const_decl) => {
                self.handle_const_declaration(const_decl);
            }
            Declaration::StorageDeclaration(storage_decl) => {
                for field in &storage_decl.fields {
                    self.ctx.tokens.insert(
                        to_ident_key(&field.name),
                        Token::from_parsed(
                            AstToken::StorageField(field.clone()),
                            SymbolKind::Field,
                        ),
                    );

                    self.collect_type_arg(&field.type_argument);
                    self.handle_expression(&field.initializer);

                    field.attributes.parse(&self.ctx);
                }
                storage_decl.attributes.parse(&self.ctx);
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
            self.ctx.tokens.insert(
                to_ident_key(alias),
                Token::from_parsed(
                    AstToken::UseStatement(use_statement.clone()),
                    SymbolKind::Unknown,
                ),
            );
        }

        for prefix in call_path {
            self.ctx.tokens.insert(
                to_ident_key(prefix),
                Token::from_parsed(
                    AstToken::UseStatement(use_statement.clone()),
                    SymbolKind::Module,
                ),
            );
        }

        match &import_type {
            ImportType::Item(item) => {
                self.ctx.tokens.insert(
                    to_ident_key(item),
                    Token::from_parsed(
                        AstToken::UseStatement(use_statement.clone()),
                        SymbolKind::Unknown,
                    ),
                );
            }
            ImportType::SelfImport(span) => {
                self.ctx.tokens.insert(
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

                self.ctx.tokens.insert(
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
                        self.ctx.tokens.insert(
                            to_ident_key(ident),
                            Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Module),
                        );
                    }

                    self.ctx.tokens.insert(
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

                    self.ctx.tokens.insert(
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

                self.ctx.tokens.insert(
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
                    self.ctx.tokens.insert(
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
                    self.ctx.tokens.insert(
                        to_ident_key(ident),
                        Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Struct),
                    );
                }

                let name = &call_path_binding.inner.suffix;
                let type_arguments = &call_path_binding.type_arguments.to_vec();

                self.ctx.tokens.insert(
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
                    self.ctx.tokens.insert(
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
                    collect_type_info_token(self.ctx, type_info, Some(ident.span()));
                }

                for type_arg in &method_name_binding.type_arguments.to_vec() {
                    self.collect_type_arg(type_arg);
                }

                // Don't collect applications of desugared operators due to mismatched ident lengths.
                if !desugared_op(&prefixes) {
                    self.ctx.tokens.insert(
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
                    self.ctx.tokens.insert(
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
                self.ctx.tokens.insert(
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
                    self.ctx.tokens.insert(
                        to_ident_key(ident),
                        Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Enum),
                    );
                }

                self.ctx.tokens.insert(
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
                    self.ctx.tokens.insert(
                        to_ident_key(ident),
                        Token::from_parsed(
                            AstToken::Expression(expression.clone()),
                            SymbolKind::Enum,
                        ),
                    );
                }

                self.ctx.tokens.insert(
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
                    self.ctx.tokens.insert(
                        to_ident_key(ident),
                        Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Module),
                    );
                }
                self.ctx.tokens.insert(
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
                    self.ctx.tokens.insert(
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
                self.ctx.tokens.insert(
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
                            self.ctx.tokens.insert(
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
            self.ctx.tokens.insert(
                to_ident_key(prefix),
                Token::from_parsed(AstToken::Ident(prefix.clone()), SymbolKind::Function),
            );
        }
        self.ctx.tokens.insert(
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
        let type_info = self.ctx.engines.te().get(type_argument.type_id);
        match &type_info {
            TypeInfo::Array(type_arg, length) => {
                let ident = Ident::new(length.span());
                self.ctx.tokens.insert(
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
                let symbol_kind = type_info_to_symbol_kind(self.ctx.engines.te(), &type_info);
                let token =
                    Token::from_parsed(AstToken::TypeArgument(type_argument.clone()), symbol_kind);

                if let Some(tree) = &type_argument.call_path_tree {
                    collect_call_path_tree(tree, &token, &self.ctx.tokens);
                }
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
                self.ctx
                    .tokens
                    .insert(to_ident_key(&Ident::new(span.clone())), token);
            }
            Scrutinee::Variable { name, .. } => {
                self.ctx.tokens.insert(
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
                    self.ctx.tokens.insert(to_ident_key(ident), token);
                }
                self.ctx.tokens.insert(
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
                        self.ctx.tokens.insert(to_ident_key(field), token);

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
                    self.ctx.tokens.insert(
                        to_ident_key(ident),
                        Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Enum),
                    );
                }

                let token =
                    Token::from_parsed(AstToken::Scrutinee(scrutinee.clone()), SymbolKind::Variant);
                self.ctx
                    .tokens
                    .insert(to_ident_key(&call_path.suffix), token);

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

    fn collect_function_parameter(&self, parameter: &FunctionParameter) {
        self.ctx.tokens.insert(
            to_ident_key(&parameter.name),
            Token::from_parsed(
                AstToken::FunctionParameter(parameter.clone()),
                SymbolKind::ValueParam,
            ),
        );
        self.collect_type_arg(&parameter.type_argument);
    }

    fn collect_trait_fn(&self, trait_fn: &TraitFn) {
        self.ctx.tokens.insert(
            to_ident_key(&trait_fn.name),
            Token::from_parsed(AstToken::TraitFn(trait_fn.clone()), SymbolKind::Function),
        );

        for parameter in &trait_fn.parameters {
            self.collect_function_parameter(parameter);
        }

        collect_type_info_token(
            self.ctx,
            &trait_fn.return_type,
            Some(trait_fn.return_type_span.clone()),
        );

        trait_fn.attributes.parse(&self.ctx);
    }

    fn collect_type_parameter(&self, type_param: &TypeParameter) {
        self.ctx.tokens.insert(
            to_ident_key(&type_param.name_ident),
            Token::from_parsed(
                AstToken::TypeParameter(type_param.clone()),
                SymbolKind::TypeParameter,
            ),
        );
    }
}

impl Parse for AttributesMap {
    fn parse(&self, ctx: &ParseContext) {
        self.iter()
            .filter(|(kind, ..)| **kind != AttributeKind::DocComment)
            .flat_map(|(.., attrs)| attrs)
            .for_each(|attribute| {
                ctx.tokens.insert(
                    to_ident_key(&attribute.name),
                    Token::from_parsed(
                        AstToken::Attribute(attribute.clone()),
                        SymbolKind::DeriveHelper,
                    ),
                );
            });
    }
}

impl Parse for AstNode {
    fn parse(&self, ctx: &ParseContext) {
        match &self.content {
            AstNodeContent::Declaration(declaration) => declaration.parse(ctx),
            AstNodeContent::Expression(expression)
            | AstNodeContent::ImplicitReturnExpression(expression) => {
                expression.parse(ctx);
            }
            AstNodeContent::UseStatement(use_statement) => use_statement.parse(ctx),
            // include statements are handled throught [`collect_module_spans`]
            AstNodeContent::IncludeStatement(_) => {}
        }
    }
}

impl Parse for Declaration {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            Declaration::VariableDeclaration(decl) => decl.parse(ctx),
            Declaration::FunctionDeclaration(decl) => decl.parse(ctx),
            Declaration::TraitDeclaration(decl) => decl.parse(ctx),
            Declaration::StructDeclaration(decl) => decl.parse(ctx),
            Declaration::EnumDeclaration(decl) => decl.parse(ctx),
            Declaration::ImplTrait(decl) => decl.parse(ctx),
            Declaration::ImplSelf(decl) => decl.parse(ctx),
            Declaration::AbiDeclaration(decl) => decl.parse(ctx),
            Declaration::ConstantDeclaration(decl) => decl.parse(ctx),
            Declaration::StorageDeclaration(decl) => decl.parse(ctx),
        }
    }
}

impl Parse for UseStatement {
    fn parse(&self, ctx: &ParseContext) {
        if let Some(alias) = &self.alias {
            ctx.tokens.insert(
                to_ident_key(alias),
                Token::from_parsed(AstToken::UseStatement(self.clone()), SymbolKind::Unknown),
            );
        }
        for prefix in &self.call_path {
            ctx.tokens.insert(
                to_ident_key(prefix),
                Token::from_parsed(AstToken::UseStatement(self.clone()), SymbolKind::Module),
            );
        }
        match &self.import_type {
            ImportType::Item(item) => {
                ctx.tokens.insert(
                    to_ident_key(item),
                    Token::from_parsed(AstToken::UseStatement(self.clone()), SymbolKind::Unknown),
                );
            }
            ImportType::SelfImport(span) => {
                ctx.tokens.insert(
                    to_ident_key(&Ident::new(span.clone())),
                    Token::from_parsed(AstToken::UseStatement(self.clone()), SymbolKind::Unknown),
                );
            }
            ImportType::Star => {}
        }
    }
}

impl Parse for Expression {
    fn parse(&self, ctx: &ParseContext) {
        match &self.kind {
            ExpressionKind::Error(_part_spans) => {
                // FIXME(Centril): Left for @JoshuaBatty to use.
            }
            ExpressionKind::Literal(value) => {
                let symbol_kind = literal_to_symbol_kind(value);
                ctx.tokens.insert(
                    to_ident_key(&Ident::new(self.span.clone())),
                    Token::from_parsed(AstToken::Expression(self.clone()), symbol_kind),
                );
            }
            ExpressionKind::FunctionApplication(function_application_expression) => {
                function_application_expression.parse(ctx);
            }
            ExpressionKind::LazyOperator(LazyOperatorExpression { lhs, rhs, .. }) => {
                lhs.parse(ctx);
                rhs.parse(ctx);
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
                    ctx.tokens.insert(
                        to_ident_key(name),
                        Token::from_parsed(AstToken::Expression(self.clone()), symbol_kind),
                    );
                }
            }
            ExpressionKind::Tuple(fields) => {
                fields.iter().for_each(|field| field.parse(ctx));
            }
            ExpressionKind::TupleIndex(TupleIndexExpression {
                prefix, index_span, ..
            }) => {
                prefix.parse(ctx);
                ctx.tokens.insert(
                    to_ident_key(&Ident::new(index_span.clone())),
                    Token::from_parsed(
                        AstToken::Expression(self.clone()),
                        SymbolKind::NumericLiteral,
                    ),
                );
            }
            ExpressionKind::Array(array_expression) => {
                array_expression.parse(ctx);
            }
            ExpressionKind::Struct(struct_expression) => {
                struct_expression.parse(ctx);
            }
            ExpressionKind::CodeBlock(code_block) => {
                code_block.contents.iter().for_each(|node| node.parse(ctx));
            }
            ExpressionKind::If(IfExpression {
                condition,
                then,
                r#else,
                ..
            }) => {
                condition.parse(ctx);
                then.parse(ctx);
                if let Some(r#else) = r#else {
                    r#else.parse(ctx);
                }
            }
            ExpressionKind::Match(MatchExpression {
                value, branches, ..
            }) => {
                value.parse(ctx);
                branches.iter().for_each(|branch| {
                    branch.scrutinee.parse(ctx);
                    branch.result.parse(ctx);
                });
            }
            ExpressionKind::Asm(asm) => {
                asm.registers.iter().for_each(|register| {
                    if let Some(initializer) = &register.initializer {
                        initializer.parse(ctx);
                    }
                });
            }
            ExpressionKind::MethodApplication(method_application_expression) => {
                method_application_expression.parse(ctx);
            }
            ExpressionKind::Subfield(SubfieldExpression {
                prefix,
                field_to_access,
                ..
            }) => {
                prefix.parse(ctx);
                ctx.tokens.insert(
                    to_ident_key(field_to_access),
                    Token::from_parsed(AstToken::Expression(self.clone()), SymbolKind::Field),
                );
            }
            ExpressionKind::AmbiguousPathExpression(path_expr) => {
                path_expr.parse(ctx);
            }
            ExpressionKind::DelineatedPath(delineated_path_expression) => {
                delineated_path_expression.parse(ctx);
            }
            ExpressionKind::AbiCast(abi_cast_expression) => {
                abi_cast_expression.parse(ctx);
            }
            ExpressionKind::ArrayIndex(ArrayIndexExpression { prefix, index, .. }) => {
                prefix.parse(ctx);
                index.parse(ctx);
            }
            ExpressionKind::StorageAccess(StorageAccessExpression { field_names }) => {
                field_names.iter().for_each(|field_name| {
                    ctx.tokens.insert(
                        to_ident_key(field_name),
                        Token::from_parsed(AstToken::Ident(field_name.clone()), SymbolKind::Field),
                    );
                });
            }
            ExpressionKind::IntrinsicFunction(intrinsic_function_expression) => {
                intrinsic_function_expression.parse(ctx);
            }
            ExpressionKind::WhileLoop(WhileLoopExpression {
                body, condition, ..
            }) => {
                body.contents.iter().for_each(|node| node.parse(ctx));
                condition.parse(ctx);
            }
            ExpressionKind::Reassignment(reassignment) => {
                reassignment.parse(ctx);
            }
            ExpressionKind::Return(expr) => {
                expr.parse(ctx);
            }
            // We are collecting these tokens in the lexed phase.
            ExpressionKind::Break | ExpressionKind::Continue => {}
        }
    }
}

impl Parse for ReassignmentExpression {
    fn parse(&self, ctx: &ParseContext) {
        self.rhs.parse(ctx);
        match &self.lhs {
            ReassignmentTarget::VariableExpression(exp) => {
                exp.parse(ctx);
            }
            ReassignmentTarget::StorageField(idents) => {
                for ident in idents {
                    ctx.tokens.insert(
                        to_ident_key(ident),
                        Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Field),
                    );
                }
            }
        }
    }
}

impl Parse for IntrinsicFunctionExpression {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            to_ident_key(&self.name),
            Token::from_parsed(
                AstToken::Intrinsic(self.kind_binding.inner.clone()),
                SymbolKind::Function,
            ),
        );
        self.arguments.iter().for_each(|arg| arg.parse(ctx));
        self.kind_binding
            .type_arguments
            .to_vec()
            .iter()
            .for_each(|type_arg| type_arg.parse(ctx));
    }
}

impl Parse for AbiCastExpression {
    fn parse(&self, ctx: &ParseContext) {
        for ident in &self.abi_name.prefixes {
            ctx.tokens.insert(
                to_ident_key(ident),
                Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Module),
            );
        }
        ctx.tokens.insert(
            to_ident_key(&self.abi_name.suffix),
            Token::from_parsed(AstToken::AbiCastExpression(self.clone()), SymbolKind::Trait),
        );
        self.address.parse(ctx);
    }
}

impl Parse for DelineatedPathExpression {
    fn parse(&self, ctx: &ParseContext) {
        let DelineatedPathExpression {
            call_path_binding,
            args,
        } = &*self;
        for ident in &call_path_binding.inner.prefixes {
            ctx.tokens.insert(
                to_ident_key(ident),
                Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Enum),
            );
        }
        ctx.tokens.insert(
            to_ident_key(&call_path_binding.inner.suffix),
            Token::from_parsed(
                AstToken::DelineatedPathExpression(self.clone()),
                SymbolKind::Variant,
            ),
        );
        call_path_binding
            .type_arguments
            .to_vec()
            .iter()
            .for_each(|type_arg| {
                type_arg.parse(ctx);
            });
        if let Some(args_vec) = args.as_ref() {
            args_vec.iter().for_each(|exp| {
                exp.parse(ctx);
            });
        }
    }
}

impl Parse for AmbiguousPathExpression {
    fn parse(&self, ctx: &ParseContext) {
        let AmbiguousPathExpression {
            call_path_binding,
            args,
        } = &*self;
        for ident in call_path_binding
            .inner
            .prefixes
            .iter()
            .chain(iter::once(&call_path_binding.inner.suffix.before.inner))
        {
            ctx.tokens.insert(
                to_ident_key(ident),
                Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Enum),
            );
        }
        ctx.tokens.insert(
            to_ident_key(&call_path_binding.inner.suffix.suffix),
            Token::from_parsed(
                AstToken::AmbiguousPathExpression(self.clone()),
                SymbolKind::Variant,
            ),
        );
        call_path_binding
            .type_arguments
            .to_vec()
            .iter()
            .for_each(|type_arg| {
                type_arg.parse(ctx);
            });
        args.iter().for_each(|arg| arg.parse(ctx));
    }
}

impl Parse for MethodApplicationExpression {
    fn parse(&self, ctx: &ParseContext) {
        let prefixes = match &self.method_name_binding.inner {
            MethodName::FromType {
                call_path_binding, ..
            } => call_path_binding.inner.prefixes.clone(),
            MethodName::FromTrait { call_path, .. } => call_path.prefixes.clone(),
            _ => vec![],
        };
        if let MethodName::FromType {
            call_path_binding, ..
        } = &self.method_name_binding.inner
        {
            let (type_info, ident) = &call_path_binding.inner.suffix;
            collect_type_info_token(ctx, type_info, Some(ident.span()));
        }
        self.method_name_binding
            .type_arguments
            .to_vec()
            .iter()
            .for_each(|type_arg| {
                type_arg.parse(ctx);
            });
        // Don't collect applications of desugared operators due to mismatched ident lengths.
        if !desugared_op(&prefixes) {
            ctx.tokens.insert(
                to_ident_key(&self.method_name_binding.inner.easy_name()),
                Token::from_parsed(
                    AstToken::MethodApplicationExpression(self.clone()),
                    SymbolKind::Struct,
                ),
            );
        }
        self.arguments.iter().for_each(|arg| arg.parse(ctx));
        self.contract_call_params
            .iter()
            .for_each(|param| param.parse(ctx));
    }
}

impl Parse for Scrutinee {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            Scrutinee::CatchAll { .. } => (),
            Scrutinee::Literal { ref value, span } => {
                let token = Token::from_parsed(
                    AstToken::Scrutinee(self.clone()),
                    literal_to_symbol_kind(value),
                );
                ctx.tokens
                    .insert(to_ident_key(&Ident::new(span.clone())), token);
            }
            Scrutinee::Variable { name, .. } => {
                ctx.tokens.insert(
                    to_ident_key(name),
                    // it could either be a variable or a constant
                    Token::from_parsed(AstToken::Scrutinee(self.clone()), SymbolKind::Unknown),
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
                    ctx.tokens.insert(to_ident_key(ident), token);
                }
                ctx.tokens.insert(
                    to_ident_key(&struct_name.suffix),
                    Token::from_parsed(AstToken::Scrutinee(self.clone()), SymbolKind::Struct),
                );
                fields.iter().for_each(|field| field.parse(ctx));
            }
            Scrutinee::EnumScrutinee {
                call_path, value, ..
            } => {
                for ident in &call_path.prefixes {
                    ctx.tokens.insert(
                        to_ident_key(ident),
                        Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Enum),
                    );
                }
                let token =
                    Token::from_parsed(AstToken::Scrutinee(self.clone()), SymbolKind::Variant);
                ctx.tokens.insert(to_ident_key(&call_path.suffix), token);
                value.parse(ctx);
            }
            Scrutinee::Tuple { elems, .. } => {
                elems.iter().for_each(|elem| elem.parse(ctx));
            }
            Scrutinee::Error { .. } => {
                // FIXME: Left for @JoshuaBatty to use.
            }
        }
    }
}

impl Parse for StructScrutineeField {
    fn parse(&self, ctx: &ParseContext) {
        let token = Token::from_parsed(
            AstToken::StructScrutineeField(self.clone()),
            SymbolKind::Field,
        );
        if let StructScrutineeField::Field {
            field, scrutinee, ..
        } = self
        {
            ctx.tokens.insert(to_ident_key(field), token);
            if let Some(scrutinee) = scrutinee {
                scrutinee.parse(ctx);
            }
        }
    }
}

impl Parse for StructExpression {
    fn parse(&self, ctx: &ParseContext) {
        for ident in &self.call_path_binding.inner.prefixes {
            ctx.tokens.insert(
                to_ident_key(ident),
                Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Struct),
            );
        }
        let name = &self.call_path_binding.inner.suffix;
        ctx.tokens.insert(
            to_ident_key(name),
            Token::from_parsed(AstToken::StructExpression(self.clone()), SymbolKind::Struct),
        );
        let type_arguments = &self.call_path_binding.type_arguments.to_vec();
        type_arguments.iter().for_each(|type_arg| {
            type_arg.parse(ctx);
        });
        self.fields.iter().for_each(|field| field.parse(ctx));
    }
}

impl Parse for StructExpressionField {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            to_ident_key(&self.name),
            Token::from_parsed(
                AstToken::StructExpressionField(self.clone()),
                SymbolKind::Field,
            ),
        );
        self.value.parse(ctx);
    }
}

impl Parse for ArrayExpression {
    fn parse(&self, ctx: &ParseContext) {
        self.contents.iter().for_each(|exp| exp.parse(ctx));
        if let Some(length_span) = &self.length_span {
            let ident = Ident::new(length_span.clone());
            ctx.tokens.insert(
                to_ident_key(&ident),
                Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::NumericLiteral),
            );
        }
    }
}

impl Parse for FunctionApplicationExpression {
    fn parse(&self, ctx: &ParseContext) {
        // Don't collect applications of desugared operators due to mismatched ident lengths.
        if !desugared_op(&self.call_path_binding.inner.prefixes) {
            for ident in &self.call_path_binding.inner.prefixes {
                ctx.tokens.insert(
                    to_ident_key(ident),
                    Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Module),
                );
            }
            ctx.tokens.insert(
                to_ident_key(&self.call_path_binding.inner.suffix),
                Token::from_parsed(
                    AstToken::FunctionApplicationExpression(self.clone()),
                    SymbolKind::Function,
                ),
            );
            self.call_path_binding
                .type_arguments
                .to_vec()
                .iter()
                .for_each(|type_arg| {
                    type_arg.parse(ctx);
                });
        }
        self.arguments.iter().for_each(|exp| {
            exp.parse(ctx);
        });
    }
}

impl Parse for VariableDeclaration {
    fn parse(&self, ctx: &ParseContext) {
        // Don't collect tokens if the ident's name contains __tuple_ || __match_return_var_name_
        // The individual elements are handled in the subsequent VariableDeclaration's
        if !self.name.as_str().contains(TUPLE_NAME_PREFIX)
            && !self.name.as_str().contains(MATCH_RETURN_VAR_NAME_PREFIX)
        {
            let symbol_kind = if self.name.as_str().contains(DESTRUCTURE_PREFIX) {
                SymbolKind::Struct
            } else {
                SymbolKind::Variable
            };
            // We want to use the span from variable.name to construct a
            // new Ident as the name_override_opt can be set to one of the
            // const prefixes and not the actual token name.
            ctx.tokens.insert(
                to_ident_key(&Ident::new(self.name.span())),
                Token::from_parsed(
                    AstToken::Declaration(Declaration::VariableDeclaration(self.clone())),
                    symbol_kind,
                ),
            );
            self.type_ascription.parse(ctx);
        }
        self.body.parse(ctx);
    }
}

impl Parse for FunctionDeclaration {
    fn parse(&self, ctx: &ParseContext) {
        let token = Token::from_parsed(
            AstToken::FunctionDeclaration(self.clone()),
            SymbolKind::Function,
        );
        ctx.tokens.insert(to_ident_key(&self.name), token.clone());
        self.body.contents.iter().for_each(|node| {
            node.parse(ctx);
        });
        self.parameters.iter().for_each(|param| {
            param.parse(ctx);
        });
        self.type_parameters.iter().for_each(|type_param| {
            type_param.parse(ctx);
        });
        for (ident, constraints) in &self.where_clause {
            ctx.tokens.insert(to_ident_key(ident), token.clone());
            constraints.iter().for_each(|constraint| {
                constraint.parse(ctx);
            });
        }
        self.return_type.parse(ctx);
        self.attributes.parse(ctx);
    }
}

impl Parse for TraitDeclaration {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            to_ident_key(&self.name),
            Token::from_parsed(
                AstToken::Declaration(Declaration::TraitDeclaration(self.clone())),
                SymbolKind::Trait,
            ),
        );
        self.interface_surface.iter().for_each(|item| match item {
            TraitItem::TraitFn(trait_fn) => trait_fn.parse(ctx),
        });
        self.methods.iter().for_each(|func_dec| {
            func_dec.parse(ctx);
        });
        self.supertraits.iter().for_each(|supertrait| {
            supertrait.parse(ctx);
        });
    }
}

impl Parse for StructDeclaration {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            to_ident_key(&self.name),
            Token::from_parsed(
                AstToken::Declaration(Declaration::StructDeclaration(self.clone())),
                SymbolKind::Struct,
            ),
        );
        self.fields.iter().for_each(|field| {
            field.parse(ctx);
        });
        self.type_parameters.iter().for_each(|type_param| {
            type_param.parse(ctx);
        });
        self.attributes.parse(ctx);
    }
}

impl Parse for EnumDeclaration {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            to_ident_key(&self.name),
            Token::from_parsed(
                AstToken::Declaration(Declaration::EnumDeclaration(self.clone())),
                SymbolKind::Enum,
            ),
        );
        self.type_parameters.iter().for_each(|type_param| {
            type_param.parse(ctx);
        });
        self.variants.iter().for_each(|variant| {
            variant.parse(ctx);
        });
        self.attributes.parse(ctx);
    }
}

impl Parse for ImplTrait {
    fn parse(&self, ctx: &ParseContext) {
        for ident in &self.trait_name.prefixes {
            ctx.tokens.insert(
                to_ident_key(ident),
                Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Module),
            );
        }
        ctx.tokens.insert(
            to_ident_key(&self.trait_name.suffix),
            Token::from_parsed(
                AstToken::Declaration(Declaration::ImplTrait(self.clone())),
                SymbolKind::Trait,
            ),
        );
        self.implementing_for.parse(ctx);
        self.impl_type_parameters.iter().for_each(|type_param| {
            type_param.parse(ctx);
        });
        self.items.iter().for_each(|item| match item {
            ImplItem::Fn(fn_decl) => fn_decl.parse(ctx),
        });
    }
}

impl Parse for ImplSelf {
    fn parse(&self, ctx: &ParseContext) {
        if let TypeInfo::Custom {
            call_path,
            type_arguments,
        } = &ctx.engines.te().get(self.implementing_for.type_id)
        {
            ctx.tokens.insert(
                to_ident_key(&call_path.suffix),
                Token::from_parsed(
                    AstToken::Declaration(Declaration::ImplSelf(self.clone())),
                    SymbolKind::Struct,
                ),
            );
            if let Some(type_arguments) = type_arguments {
                for type_arg in type_arguments {
                    type_arg.parse(ctx);
                }
            }
        }
        self.impl_type_parameters.iter().for_each(|type_param| {
            type_param.parse(ctx);
        });
        self.items.iter().for_each(|item| match item {
            ImplItem::Fn(fn_decl) => fn_decl.parse(ctx),
        });
    }
}

impl Parse for AbiDeclaration {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            to_ident_key(&self.name),
            Token::from_parsed(
                AstToken::Declaration(Declaration::AbiDeclaration(self.clone())),
                SymbolKind::Trait,
            ),
        );
        self.interface_surface.iter().for_each(|item| match item {
            TraitItem::TraitFn(trait_fn) => trait_fn.parse(ctx),
        });
        self.supertraits.iter().for_each(|supertrait| {
            supertrait.parse(ctx);
        });
        self.attributes.parse(ctx);
    }
}

impl Parse for ConstantDeclaration {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            to_ident_key(&self.name),
            Token::from_parsed(
                AstToken::Declaration(Declaration::ConstantDeclaration(self.clone())),
                SymbolKind::Const,
            ),
        );
        self.type_ascription.parse(ctx);
        if let Some(value) = &self.value {
            value.parse(ctx);
        }
        self.attributes.parse(ctx);
    }
}

impl Parse for StorageDeclaration {
    fn parse(&self, ctx: &ParseContext) {
        self.fields.iter().for_each(|field| {
            field.parse(ctx);
        });
        self.attributes.parse(ctx);
    }
}

impl Parse for StorageField {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            to_ident_key(&self.name),
            Token::from_parsed(AstToken::StorageField(self.clone()), SymbolKind::Field),
        );
        self.type_argument.parse(ctx);
        self.initializer.parse(ctx);
        self.attributes.parse(ctx);
    }
}

impl Parse for Supertrait {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            to_ident_key(&self.name.suffix),
            Token::from_parsed(AstToken::Supertrait(self.clone()), SymbolKind::Trait),
        );
    }
}

impl Parse for TraitFn {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            to_ident_key(&self.name),
            Token::from_parsed(AstToken::TraitFn(self.clone()), SymbolKind::Function),
        );
        self.parameters.iter().for_each(|param| {
            param.parse(ctx);
        });
        collect_type_info_token(ctx, &self.return_type, Some(self.return_type_span.clone()));
        self.attributes.parse(ctx);
    }
}

impl Parse for TraitConstraint {
    fn parse(&self, ctx: &ParseContext) {
        for prefix in &self.trait_name.prefixes {
            ctx.tokens.insert(
                to_ident_key(prefix),
                Token::from_parsed(AstToken::Ident(prefix.clone()), SymbolKind::Function),
            );
        }
        ctx.tokens.insert(
            to_ident_key(&self.trait_name.suffix),
            Token::from_parsed(
                AstToken::TraitConstraint(self.clone()),
                SymbolKind::Function,
            ),
        );
        self.type_arguments.iter().for_each(|type_arg| {
            type_arg.parse(ctx);
        });
    }
}

impl Parse for FunctionParameter {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            to_ident_key(&self.name),
            Token::from_parsed(
                AstToken::FunctionParameter(self.clone()),
                SymbolKind::ValueParam,
            ),
        );
        self.type_argument.parse(ctx);
    }
}

impl Parse for StructField {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            to_ident_key(&self.name),
            Token::from_parsed(AstToken::StructField(self.clone()), SymbolKind::Field),
        );
        self.type_argument.parse(ctx);
        self.attributes.parse(ctx);
    }
}

impl Parse for EnumVariant {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            to_ident_key(&self.name),
            Token::from_parsed(AstToken::EnumVariant(self.clone()), SymbolKind::Variant),
        );
        self.type_argument.parse(ctx);
        self.attributes.parse(ctx);
    }
}

impl Parse for TypeParameter {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            to_ident_key(&self.name_ident),
            Token::from_parsed(
                AstToken::TypeParameter(self.clone()),
                SymbolKind::TypeParameter,
            ),
        );
    }
}

impl Parse for TypeArgument {
    fn parse(&self, ctx: &ParseContext) {
        let type_info = ctx.engines.te().get(self.type_id);
        match &type_info {
            TypeInfo::Array(type_arg, length) => {
                let ident = Ident::new(length.span());
                ctx.tokens.insert(
                    to_ident_key(&ident),
                    Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::NumericLiteral),
                );
                type_arg.parse(ctx);
            }
            TypeInfo::Tuple(type_arguments) => {
                for type_arg in type_arguments {
                    type_arg.parse(ctx);
                }
            }
            _ => {
                let symbol_kind = type_info_to_symbol_kind(ctx.engines.te(), &type_info);
                if let Some(tree) = &self.call_path_tree {
                    let token =
                        Token::from_parsed(AstToken::TypeArgument(self.clone()), symbol_kind);
                    collect_call_path_tree(tree, &token, &ctx.tokens);
                }
            }
        }
    }
}

fn collect_type_info_token(ctx: &ParseContext, type_info: &TypeInfo, type_span: Option<Span>) {
    let symbol_kind = type_info_to_symbol_kind(ctx.engines.te(), type_info);
    match type_info {
        TypeInfo::Str(length) => {
            let ident = Ident::new(length.span());
            ctx.tokens.insert(
                to_ident_key(&ident),
                Token::from_parsed(AstToken::Ident(ident.clone()), symbol_kind),
            );
        }
        TypeInfo::Array(type_arg, length) => {
            let ident = Ident::new(length.span());
            ctx.tokens.insert(
                to_ident_key(&ident),
                Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::NumericLiteral),
            );
            type_arg.parse(ctx);
        }
        TypeInfo::Tuple(type_arguments) => {
            type_arguments.iter().for_each(|type_arg| {
                type_arg.parse(ctx);
            });
        }
        TypeInfo::Custom {
            call_path,
            type_arguments,
        } => {
            let ident = call_path.suffix.clone();
            let mut token = Token::from_parsed(AstToken::Ident(ident.clone()), symbol_kind);
            token.type_def = Some(TypeDefinition::Ident(ident.clone()));
            ctx.tokens.insert(to_ident_key(&ident), token);
            if let Some(type_arguments) = type_arguments {
                type_arguments.iter().for_each(|type_arg| {
                    type_arg.parse(ctx);
                });
            }
        }
        _ => {
            if let Some(type_span) = type_span {
                let ident = Ident::new(type_span);
                ctx.tokens.insert(
                    to_ident_key(&ident),
                    Token::from_parsed(AstToken::Ident(ident.clone()), symbol_kind),
                );
            }
        }
    }
}

fn collect_call_path_tree(tree: &CallPathTree, token: &Token, tokens: &TokenMap) {
    for ident in &tree.call_path.prefixes {
        tokens.insert(to_ident_key(ident), token.clone());
    }
    tokens.insert(to_ident_key(&tree.call_path.suffix), token.clone());
    for child in &tree.children {
        collect_call_path_tree(child, token, tokens);
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
