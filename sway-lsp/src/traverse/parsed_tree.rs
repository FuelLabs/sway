#![allow(dead_code)]

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
            ArrayIndexExpression, AstNode, AstNodeContent, ConstantDeclaration, Declaration,
            DelineatedPathExpression, EnumDeclaration, EnumVariant, Expression, ExpressionKind,
            FunctionApplicationExpression, FunctionDeclaration, FunctionParameter, IfExpression,
            ImplItem, ImplSelf, ImplTrait, ImportType, IntrinsicFunctionExpression,
            LazyOperatorExpression, MatchExpression, MethodApplicationExpression, MethodName,
            ParseModule, ParseProgram, ParseSubmodule, ReassignmentExpression, ReassignmentTarget,
            Scrutinee, StorageAccessExpression, StorageDeclaration, StorageField,
            StructDeclaration, StructExpression, StructExpressionField, StructField,
            StructScrutineeField, SubfieldExpression, Supertrait, TraitDeclaration, TraitFn,
            TraitItem, TupleIndexExpression, TypeAliasDeclaration, UseStatement,
            VariableDeclaration, WhileLoopExpression,
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
        node.parse(self.ctx);
    }

    /// Collects module names from the mod statements
    pub fn collect_module_spans(&self, parse_program: &ParseProgram) {
        self.collect_parse_module(&parse_program.root);
    }

    fn collect_parse_module(&self, parse_module: &ParseModule) {
        self.ctx.tokens.insert(
            to_ident_key(&Ident::new(parse_module.span.clone())),
            Token::from_parsed(
                AstToken::LibrarySpan(parse_module.span.clone()),
                SymbolKind::Module,
            ),
        );
        for (
            _,
            ParseSubmodule {
                module,
                mod_name_span,
                ..
            },
        ) in &parse_module.submodules
        {
            self.ctx.tokens.insert(
                to_ident_key(&Ident::new(mod_name_span.clone())),
                Token::from_parsed(AstToken::IncludeStatement, SymbolKind::Module),
            );
            self.collect_parse_module(module);
        }
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
            Declaration::TypeAliasDeclaration(decl) => decl.parse(ctx),
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
            ExpressionKind::Error(part_spans) => {
                for span in part_spans.iter() {
                    ctx.tokens.insert(
                        to_ident_key(&Ident::new(span.clone())),
                        Token::from_parsed(
                            AstToken::ErrorRecovery(span.clone()),
                            SymbolKind::Unknown,
                        ),
                    );
                }
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
                    } else if name.as_str() == "self" {
                        SymbolKind::SelfKeyword
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
            ExpressionKind::AmbiguousVariableExpression(ident) => {
                ctx.tokens.insert(
                    to_ident_key(ident),
                    Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Unknown),
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
        }
    }
}

impl Parse for IntrinsicFunctionExpression {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            to_ident_key(&self.name),
            Token::from_parsed(
                AstToken::Intrinsic(self.kind_binding.inner.clone()),
                SymbolKind::Intrinsic,
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
        } = self;
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
            qualified_path_root,
        } = self;
        for ident in call_path_binding.inner.prefixes.iter().chain(
            call_path_binding
                .inner
                .suffix
                .before
                .iter()
                .map(|before| &before.inner),
        ) {
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

        if let Some(qualified_path_root) = qualified_path_root {
            qualified_path_root.ty.parse(ctx);
            collect_type_info_token(
                ctx,
                &qualified_path_root.as_trait,
                qualified_path_root.as_trait_span,
            );
        }
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
            collect_type_info_token(ctx, type_info, Some(&ident.span()));
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
            Scrutinee::AmbiguousSingleIdent(ident) => {
                let token = Token::from_parsed(AstToken::Ident(ident.clone()), SymbolKind::Unknown);
                ctx.tokens.insert(to_ident_key(ident), token);
            }
            Scrutinee::Tuple { elems, .. } | Scrutinee::Or { elems, .. } => {
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
            let ident = if self.name.is_raw_ident() {
                Ident::new_with_raw(self.name.span(), true)
            } else {
                Ident::new(self.name.span())
            };
            ctx.tokens.insert(
                to_ident_key(&ident),
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
            AstToken::Declaration(Declaration::FunctionDeclaration(self.clone())),
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
            TraitItem::Constant(const_decl) => const_decl.parse(ctx),
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
            ImplItem::Constant(const_decl) => const_decl.parse(ctx),
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
            ImplItem::Constant(const_decl) => const_decl.parse(ctx),
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
            TraitItem::Constant(const_decl) => const_decl.parse(ctx),
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
        collect_type_info_token(ctx, &self.return_type, Some(&self.return_type_span));
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
                let symbol_kind = type_info_to_symbol_kind(ctx.engines.te(), &type_info, None);
                if let Some(tree) = &self.call_path_tree {
                    let token =
                        Token::from_parsed(AstToken::TypeArgument(self.clone()), symbol_kind);
                    collect_call_path_tree(tree, &token, ctx.tokens);
                }
            }
        }
    }
}

impl Parse for TypeAliasDeclaration {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            to_ident_key(&self.name),
            Token::from_parsed(
                AstToken::Declaration(Declaration::TypeAliasDeclaration(self.clone())),
                SymbolKind::TypeAlias,
            ),
        );
        self.ty.parse(ctx);
        self.attributes.parse(ctx);
    }
}

fn collect_type_info_token(ctx: &ParseContext, type_info: &TypeInfo, type_span: Option<&Span>) {
    let symbol_kind = type_info_to_symbol_kind(ctx.engines.te(), type_info, type_span);
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
                let ident = Ident::new(type_span.clone());
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
