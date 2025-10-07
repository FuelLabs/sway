#![allow(dead_code)]

use crate::{
    core::{
        token::{
            desugared_op, type_info_to_symbol_kind, ParsedAstToken, SymbolKind, Token,
            TypeDefinition,
        },
        token_map::TokenMap,
    },
    traverse::{adaptive_iter, Parse, ParseContext},
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use sway_core::{
    compiler_generated::{
        is_generated_any_match_expression_var_name, is_generated_destructured_struct_var_name,
        is_generated_tuple_var_name,
    },
    decl_engine::parsed_id::ParsedDeclId,
    language::{
        parsed::{
            AbiCastExpression, AbiDeclaration, AmbiguousPathExpression, ArrayExpression,
            ArrayIndexExpression, AstNode, AstNodeContent, ConfigurableDeclaration,
            ConstGenericDeclaration, ConstantDeclaration, Declaration, DelineatedPathExpression,
            EnumDeclaration, EnumVariant, Expression, ExpressionKind, ForLoopExpression,
            FunctionApplicationExpression, FunctionDeclaration, FunctionParameter, IfExpression,
            ImplItem, ImplSelfOrTrait, ImportType, IntrinsicFunctionExpression,
            LazyOperatorExpression, MatchExpression, MethodApplicationExpression, MethodName,
            ModStatement, ParseModule, ParseProgram, ParseSubmodule, QualifiedPathType,
            ReassignmentExpression, ReassignmentTarget, RefExpression, Scrutinee, Statement,
            StorageAccessExpression, StorageDeclaration, StorageEntry, StorageField,
            StorageNamespace, StructDeclaration, StructExpression, StructExpressionField,
            StructField, StructScrutineeField, SubfieldExpression, Supertrait, TraitDeclaration,
            TraitFn, TraitItem, TraitTypeDeclaration, TupleIndexExpression, TypeAliasDeclaration,
            UseStatement, VariableDeclaration, WhileLoopExpression,
        },
        CallPathTree, HasSubmodules, Literal,
    },
    transform::Attributes,
    type_system::{GenericArgument, TypeParameter},
    TraitConstraint, TypeInfo,
};
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
            self.ctx
                .ident(&Ident::new(parse_module.module_kind_span.clone())),
            Token::from_parsed(
                ParsedAstToken::LibrarySpan(parse_module.module_kind_span.clone()),
                SymbolKind::Keyword,
            ),
        );
        for (
            _,
            ParseSubmodule {
                module,
                mod_name_span,
                ..
            },
        ) in parse_module.submodules_recursive()
        {
            self.ctx.tokens.insert(
                self.ctx.ident(&Ident::new(mod_name_span.clone())),
                Token::from_parsed(ParsedAstToken::ModuleName, SymbolKind::Module),
            );
            self.collect_parse_module(module);
        }
    }
}

impl Parse for Attributes {
    fn parse(&self, ctx: &ParseContext) {
        self.all_as_slice()
            .par_iter()
            .filter(|attribute| !attribute.is_doc_comment())
            .for_each_with(ctx, |ctx, attribute| {
                ctx.tokens.insert(
                    ctx.ident(&attribute.name),
                    Token::from_parsed(
                        ParsedAstToken::Attribute(attribute.clone()),
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
            AstNodeContent::Expression(expression) => {
                expression.parse(ctx);
            }
            AstNodeContent::Statement(statement) => statement.parse(ctx),
            AstNodeContent::Error(_, _) => {}
        }
    }
}

impl Parse for Statement {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            Statement::Use(use_statement) => use_statement.parse(ctx),
            Statement::Mod(mod_statement) => mod_statement.parse(ctx),
        }
    }
}

impl Parse for Declaration {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            Declaration::VariableDeclaration(decl_id) => decl_id.parse(ctx),
            Declaration::FunctionDeclaration(decl_id) => decl_id.parse(ctx),
            Declaration::TraitDeclaration(decl_id) => decl_id.parse(ctx),
            Declaration::StructDeclaration(decl_id) => decl_id.parse(ctx),
            Declaration::EnumDeclaration(decl_id) => decl_id.parse(ctx),
            Declaration::EnumVariantDeclaration(_decl) => unreachable!(),
            Declaration::ImplSelfOrTrait(decl_id) => decl_id.parse(ctx),
            Declaration::AbiDeclaration(decl_id) => decl_id.parse(ctx),
            Declaration::ConstantDeclaration(decl_id) => decl_id.parse(ctx),
            Declaration::ConfigurableDeclaration(decl_id) => decl_id.parse(ctx),
            Declaration::ConstGenericDeclaration(decl_id) => decl_id.parse(ctx),
            Declaration::StorageDeclaration(decl_id) => decl_id.parse(ctx),
            Declaration::TypeAliasDeclaration(decl_id) => decl_id.parse(ctx),
            Declaration::TraitTypeDeclaration(decl_id) => decl_id.parse(ctx),
            Declaration::TraitFnDeclaration(decl_id) => decl_id.parse(ctx),
        }
    }
}

impl Parse for UseStatement {
    fn parse(&self, ctx: &ParseContext) {
        if let Some(alias) = &self.alias {
            ctx.tokens.insert(
                ctx.ident(alias),
                Token::from_parsed(
                    ParsedAstToken::UseStatement(self.clone()),
                    SymbolKind::Unknown,
                ),
            );
        }
        adaptive_iter(&self.call_path, |prefix| {
            ctx.tokens.insert(
                ctx.ident(prefix),
                Token::from_parsed(
                    ParsedAstToken::UseStatement(self.clone()),
                    SymbolKind::Module,
                ),
            );
        });
        match &self.import_type {
            ImportType::Item(item) => {
                ctx.tokens.insert(
                    ctx.ident(item),
                    Token::from_parsed(
                        ParsedAstToken::UseStatement(self.clone()),
                        SymbolKind::Unknown,
                    ),
                );
            }
            ImportType::SelfImport(span) => {
                ctx.tokens.insert(
                    ctx.ident(&Ident::new(span.clone())),
                    Token::from_parsed(
                        ParsedAstToken::UseStatement(self.clone()),
                        SymbolKind::Unknown,
                    ),
                );
            }
            ImportType::Star => {}
        }
    }
}

impl Parse for ModStatement {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            ctx.ident(&self.mod_name),
            Token::from_parsed(
                ParsedAstToken::ModStatement(self.clone()),
                SymbolKind::Unknown,
            ),
        );
    }
}

impl Parse for Expression {
    fn parse(&self, ctx: &ParseContext) {
        match &self.kind {
            ExpressionKind::Error(part_spans, _) => {
                adaptive_iter(part_spans, |span| {
                    ctx.tokens.insert(
                        ctx.ident(&Ident::new(span.clone())),
                        Token::from_parsed(
                            ParsedAstToken::ErrorRecovery(span.clone()),
                            SymbolKind::Unknown,
                        ),
                    );
                });
            }
            ExpressionKind::Literal(value) => {
                let symbol_kind = literal_to_symbol_kind(value);
                ctx.tokens.insert(
                    ctx.ident(&Ident::new(self.span.clone())),
                    Token::from_parsed(ParsedAstToken::Expression(self.clone()), symbol_kind),
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
                if !(is_generated_tuple_var_name(name.as_str())
                    || is_generated_any_match_expression_var_name(name.as_str()))
                {
                    let symbol_kind = if is_generated_destructured_struct_var_name(name.as_str()) {
                        SymbolKind::Struct
                    } else if name.as_str() == "self" {
                        SymbolKind::SelfKeyword
                    } else {
                        SymbolKind::Variable
                    };
                    ctx.tokens.insert(
                        ctx.ident(name),
                        Token::from_parsed(ParsedAstToken::Expression(self.clone()), symbol_kind),
                    );
                }
            }
            ExpressionKind::Tuple(fields) => {
                adaptive_iter(fields, |field| field.parse(ctx));
            }
            ExpressionKind::TupleIndex(TupleIndexExpression {
                prefix, index_span, ..
            }) => {
                prefix.parse(ctx);
                ctx.tokens.insert(
                    ctx.ident(&Ident::new(index_span.clone())),
                    Token::from_parsed(
                        ParsedAstToken::Expression(self.clone()),
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
                adaptive_iter(&code_block.contents, |node| node.parse(ctx));
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
                adaptive_iter(branches, |branch| {
                    branch.scrutinee.parse(ctx);
                    branch.result.parse(ctx);
                });
            }
            ExpressionKind::Asm(asm) => {
                adaptive_iter(&asm.registers, |register| {
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
                    ctx.ident(field_to_access),
                    Token::from_parsed(ParsedAstToken::Expression(self.clone()), SymbolKind::Field),
                );
            }
            ExpressionKind::AmbiguousVariableExpression(ident) => {
                ctx.tokens.insert(
                    ctx.ident(ident),
                    Token::from_parsed(ParsedAstToken::Ident(ident.clone()), SymbolKind::Unknown),
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
            ExpressionKind::StorageAccess(StorageAccessExpression {
                field_names,
                namespace_names,
                storage_keyword_span,
            }) => {
                let storage_ident = Ident::new(storage_keyword_span.clone());
                ctx.tokens.insert(
                    ctx.ident(&storage_ident),
                    Token::from_parsed(ParsedAstToken::Ident(storage_ident), SymbolKind::Unknown),
                );
                adaptive_iter(namespace_names, |namespace_name| {
                    ctx.tokens.insert(
                        ctx.ident(namespace_name),
                        Token::from_parsed(
                            ParsedAstToken::Ident(namespace_name.clone()),
                            SymbolKind::Field,
                        ),
                    );
                });
                adaptive_iter(field_names, |field_name| {
                    ctx.tokens.insert(
                        ctx.ident(field_name),
                        Token::from_parsed(
                            ParsedAstToken::Ident(field_name.clone()),
                            SymbolKind::Field,
                        ),
                    );
                });
            }
            ExpressionKind::IntrinsicFunction(intrinsic_function_expression) => {
                intrinsic_function_expression.parse(ctx);
            }
            ExpressionKind::WhileLoop(WhileLoopExpression {
                body, condition, ..
            }) => {
                adaptive_iter(&body.contents, |node| node.parse(ctx));
                condition.parse(ctx);
            }
            ExpressionKind::ForLoop(ForLoopExpression { desugared }) => {
                desugared.parse(ctx);
            }
            ExpressionKind::Reassignment(reassignment) => {
                reassignment.parse(ctx);
            }
            ExpressionKind::ImplicitReturn(expr)
            | ExpressionKind::Return(expr)
            | ExpressionKind::Panic(expr)
            | ExpressionKind::Ref(RefExpression { value: expr, .. })
            | ExpressionKind::Deref(expr) => {
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
            ReassignmentTarget::ElementAccess(exp) | ReassignmentTarget::Deref(exp) => {
                exp.parse(ctx);
            }
        }
    }
}

impl Parse for IntrinsicFunctionExpression {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            ctx.ident(&self.name),
            Token::from_parsed(
                ParsedAstToken::Intrinsic(self.kind_binding.inner.clone()),
                SymbolKind::Intrinsic,
            ),
        );
        adaptive_iter(&self.arguments, |arg| arg.parse(ctx));
        adaptive_iter(&self.kind_binding.type_arguments.to_vec(), |type_arg| {
            type_arg.parse(ctx);
        });
    }
}

impl Parse for AbiCastExpression {
    fn parse(&self, ctx: &ParseContext) {
        adaptive_iter(&self.abi_name.prefixes, |ident| {
            ctx.tokens.insert(
                ctx.ident(ident),
                Token::from_parsed(ParsedAstToken::Ident(ident.clone()), SymbolKind::Module),
            );
        });
        ctx.tokens.insert(
            ctx.ident(&self.abi_name.suffix),
            Token::from_parsed(
                ParsedAstToken::AbiCastExpression(self.clone()),
                SymbolKind::Trait,
            ),
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
        adaptive_iter(&call_path_binding.inner.call_path.prefixes, |ident| {
            ctx.tokens.insert(
                ctx.ident(ident),
                Token::from_parsed(ParsedAstToken::Ident(ident.clone()), SymbolKind::Enum),
            );
        });
        ctx.tokens.insert(
            ctx.ident(&call_path_binding.inner.call_path.suffix),
            Token::from_parsed(
                ParsedAstToken::DelineatedPathExpression(self.clone()),
                SymbolKind::Variant,
            ),
        );
        adaptive_iter(&call_path_binding.type_arguments.to_vec(), |type_arg| {
            type_arg.parse(ctx);
        });
        if let Some(args_vec) = args.as_ref() {
            adaptive_iter(args_vec, |exp| exp.parse(ctx));
        }
        collect_qualified_path_root(ctx, call_path_binding.inner.qualified_path_root.clone());
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
                ctx.ident(ident),
                Token::from_parsed(ParsedAstToken::Ident(ident.clone()), SymbolKind::Enum),
            );
        }
        ctx.tokens.insert(
            ctx.ident(&call_path_binding.inner.suffix.suffix),
            Token::from_parsed(
                ParsedAstToken::AmbiguousPathExpression(self.clone()),
                SymbolKind::Variant,
            ),
        );
        adaptive_iter(&call_path_binding.type_arguments.to_vec(), |type_arg| {
            type_arg.parse(ctx);
        });
        adaptive_iter(args, |exp| exp.parse(ctx));
        collect_qualified_path_root(ctx, qualified_path_root.clone().map(Box::new));
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
        adaptive_iter(
            &self.method_name_binding.type_arguments.to_vec(),
            |type_arg| {
                type_arg.parse(ctx);
            },
        );
        // Don't collect applications of desugared operators due to mismatched ident lengths.
        if !desugared_op(&prefixes) {
            ctx.tokens.insert(
                ctx.ident(&self.method_name_binding.inner.easy_name()),
                Token::from_parsed(
                    ParsedAstToken::MethodApplicationExpression(self.clone()),
                    SymbolKind::Struct,
                ),
            );
        }
        adaptive_iter(&self.arguments, |arg| arg.parse(ctx));
        adaptive_iter(&self.contract_call_params, |param| param.parse(ctx));
    }
}

impl Parse for Scrutinee {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            Scrutinee::CatchAll { .. } => (),
            Scrutinee::Literal { ref value, span } => {
                let token = Token::from_parsed(
                    ParsedAstToken::Scrutinee(self.clone()),
                    literal_to_symbol_kind(value),
                );
                ctx.tokens
                    .insert(ctx.ident(&Ident::new(span.clone())), token);
            }
            Scrutinee::Variable { name, .. } => {
                ctx.tokens.insert(
                    ctx.ident(name),
                    // it could either be a variable or a constant
                    Token::from_parsed(
                        ParsedAstToken::Scrutinee(self.clone()),
                        SymbolKind::Unknown,
                    ),
                );
            }
            Scrutinee::StructScrutinee {
                struct_name,
                fields,
                ..
            } => {
                adaptive_iter(&struct_name.prefixes, |ident| {
                    let token = Token::from_parsed(
                        ParsedAstToken::Ident(ident.clone()),
                        SymbolKind::Struct,
                    );
                    ctx.tokens.insert(ctx.ident(ident), token);
                });
                ctx.tokens.insert(
                    ctx.ident(&struct_name.suffix),
                    Token::from_parsed(ParsedAstToken::Scrutinee(self.clone()), SymbolKind::Struct),
                );
                adaptive_iter(fields, |field| field.parse(ctx));
            }
            Scrutinee::EnumScrutinee {
                call_path, value, ..
            } => {
                adaptive_iter(&call_path.prefixes, |ident| {
                    ctx.tokens.insert(
                        ctx.ident(ident),
                        Token::from_parsed(ParsedAstToken::Ident(ident.clone()), SymbolKind::Enum),
                    );
                });
                let token = Token::from_parsed(
                    ParsedAstToken::Scrutinee(self.clone()),
                    SymbolKind::Variant,
                );
                ctx.tokens.insert(ctx.ident(&call_path.suffix), token);
                value.parse(ctx);
            }
            Scrutinee::AmbiguousSingleIdent(ident) => {
                let token =
                    Token::from_parsed(ParsedAstToken::Ident(ident.clone()), SymbolKind::Unknown);
                ctx.tokens.insert(ctx.ident(ident), token);
            }
            Scrutinee::Tuple { elems, .. } | Scrutinee::Or { elems, .. } => {
                adaptive_iter(elems, |elem| elem.parse(ctx));
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
            ParsedAstToken::StructScrutineeField(self.clone()),
            SymbolKind::Field,
        );
        if let StructScrutineeField::Field {
            field, scrutinee, ..
        } = self
        {
            ctx.tokens.insert(ctx.ident(field), token);
            if let Some(scrutinee) = scrutinee {
                scrutinee.parse(ctx);
            }
        }
    }
}

impl Parse for StructExpression {
    fn parse(&self, ctx: &ParseContext) {
        adaptive_iter(&self.call_path_binding.inner.prefixes, |ident| {
            ctx.tokens.insert(
                ctx.ident(ident),
                Token::from_parsed(ParsedAstToken::Ident(ident.clone()), SymbolKind::Struct),
            );
        });
        let name = &self.call_path_binding.inner.suffix;
        let symbol_kind = if name.as_str() == "Self" {
            SymbolKind::SelfKeyword
        } else {
            SymbolKind::Struct
        };
        ctx.tokens.insert(
            ctx.ident(name),
            Token::from_parsed(ParsedAstToken::StructExpression(self.clone()), symbol_kind),
        );
        let type_arguments = &self.call_path_binding.type_arguments.to_vec();
        adaptive_iter(type_arguments, |type_arg| type_arg.parse(ctx));
        adaptive_iter(&self.fields, |field| field.parse(ctx));
    }
}

impl Parse for StructExpressionField {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            ctx.ident(&self.name),
            Token::from_parsed(
                ParsedAstToken::StructExpressionField(self.clone()),
                SymbolKind::Field,
            ),
        );
        self.value.parse(ctx);
    }
}

impl Parse for ArrayExpression {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            ArrayExpression::Explicit {
                contents,
                length_span,
            } => {
                adaptive_iter(contents, |exp| exp.parse(ctx));
                if let Some(length_span) = &length_span {
                    let ident = Ident::new(length_span.clone());
                    ctx.tokens.insert(
                        ctx.ident(&ident),
                        Token::from_parsed(
                            ParsedAstToken::Ident(ident.clone()),
                            SymbolKind::NumericLiteral,
                        ),
                    );
                }
            }
            ArrayExpression::Repeat { value, length } => {
                value.parse(ctx);
                length.parse(ctx);
            }
        }
    }
}

impl Parse for FunctionApplicationExpression {
    fn parse(&self, ctx: &ParseContext) {
        // Don't collect applications of desugared operators due to mismatched ident lengths.
        if !desugared_op(&self.call_path_binding.inner.prefixes) {
            adaptive_iter(&self.call_path_binding.inner.prefixes, |ident| {
                ctx.tokens.insert(
                    ctx.ident(ident),
                    Token::from_parsed(ParsedAstToken::Ident(ident.clone()), SymbolKind::Module),
                );
            });
            ctx.tokens.insert(
                ctx.ident(&self.call_path_binding.inner.suffix),
                Token::from_parsed(
                    ParsedAstToken::FunctionApplicationExpression(self.clone()),
                    SymbolKind::Function,
                ),
            );
            adaptive_iter(
                &self.call_path_binding.type_arguments.to_vec(),
                |type_arg| {
                    type_arg.parse(ctx);
                },
            );
        }
        adaptive_iter(&self.arguments, |exp| exp.parse(ctx));
    }
}

impl Parse for ParsedDeclId<VariableDeclaration> {
    fn parse(&self, ctx: &ParseContext) {
        let var_decl = ctx.engines.pe().get_variable(self);
        // Don't collect tokens if the idents are generated tuple or match desugaring names.
        // The individual elements are handled in the subsequent VariableDeclaration's.
        if !(is_generated_tuple_var_name(var_decl.name.as_str())
            || is_generated_any_match_expression_var_name(var_decl.name.as_str()))
        {
            let symbol_kind = if is_generated_destructured_struct_var_name(var_decl.name.as_str()) {
                SymbolKind::Struct
            } else {
                SymbolKind::Variable
            };
            // We want to use the span from variable.name to construct a
            // new Ident as the name_override_opt can be set to one of the
            // const prefixes and not the actual token name.
            let ident = if var_decl.name.is_raw_ident() {
                Ident::new_with_raw(var_decl.name.span(), true)
            } else {
                Ident::new(var_decl.name.span())
            };
            ctx.tokens.insert(
                ctx.ident(&ident),
                Token::from_parsed(
                    ParsedAstToken::Declaration(Declaration::VariableDeclaration(*self)),
                    symbol_kind,
                ),
            );
            var_decl.type_ascription.parse(ctx);
        }
        var_decl.body.parse(ctx);
    }
}

impl Parse for ParsedDeclId<FunctionDeclaration> {
    fn parse(&self, ctx: &ParseContext) {
        let token = Token::from_parsed(
            ParsedAstToken::Declaration(Declaration::FunctionDeclaration(*self)),
            SymbolKind::Function,
        );
        let fn_decl = ctx.engines.pe().get_function(self);
        ctx.tokens.insert(ctx.ident(&fn_decl.name), token.clone());
        adaptive_iter(&fn_decl.body.contents, |node| node.parse(ctx));
        adaptive_iter(&fn_decl.parameters, |param| param.parse(ctx));
        adaptive_iter(&fn_decl.type_parameters, |type_param| type_param.parse(ctx));
        for (ident, constraints) in &fn_decl.where_clause {
            ctx.tokens.insert(ctx.ident(ident), token.clone());
            adaptive_iter(constraints, |constraint| constraint.parse(ctx));
        }
        fn_decl.return_type.parse(ctx);
        fn_decl.attributes.parse(ctx);
    }
}

impl Parse for ParsedDeclId<TraitDeclaration> {
    fn parse(&self, ctx: &ParseContext) {
        let trait_decl = ctx.engines.pe().get_trait(self);
        ctx.tokens.insert(
            ctx.ident(&trait_decl.name),
            Token::from_parsed(
                ParsedAstToken::Declaration(Declaration::TraitDeclaration(*self)),
                SymbolKind::Trait,
            ),
        );
        adaptive_iter(&trait_decl.interface_surface, |item| match item {
            TraitItem::TraitFn(trait_fn) => trait_fn.parse(ctx),
            TraitItem::Constant(const_decl) => const_decl.parse(ctx),
            TraitItem::Type(trait_type) => trait_type.parse(ctx),
            TraitItem::Error(_, _) => {}
        });
        adaptive_iter(&trait_decl.methods, |func_dec| func_dec.parse(ctx));
        adaptive_iter(&trait_decl.supertraits, |supertrait| supertrait.parse(ctx));
    }
}

impl Parse for ParsedDeclId<StructDeclaration> {
    fn parse(&self, ctx: &ParseContext) {
        let struct_decl = ctx.engines.pe().get_struct(self);
        ctx.tokens.insert(
            ctx.ident(&struct_decl.name),
            Token::from_parsed(
                ParsedAstToken::Declaration(Declaration::StructDeclaration(*self)),
                SymbolKind::Struct,
            ),
        );
        adaptive_iter(&struct_decl.fields, |field| field.parse(ctx));
        adaptive_iter(&struct_decl.type_parameters, |type_param| {
            type_param.parse(ctx);
        });
        struct_decl.attributes.parse(ctx);
    }
}

impl Parse for ParsedDeclId<EnumDeclaration> {
    fn parse(&self, ctx: &ParseContext) {
        let enum_decl = ctx.engines.pe().get_enum(self);
        ctx.tokens.insert(
            ctx.ident(&enum_decl.name),
            Token::from_parsed(
                ParsedAstToken::Declaration(Declaration::EnumDeclaration(*self)),
                SymbolKind::Enum,
            ),
        );
        adaptive_iter(&enum_decl.type_parameters, |type_param| {
            type_param.parse(ctx);
        });
        adaptive_iter(&enum_decl.variants, |variant| variant.parse(ctx));
        enum_decl.attributes.parse(ctx);
    }
}

impl Parse for ParsedDeclId<ImplSelfOrTrait> {
    fn parse(&self, ctx: &ParseContext) {
        let impl_self_or_trait = ctx.engines.pe().get_impl_self_or_trait(self);
        adaptive_iter(&impl_self_or_trait.trait_name.prefixes, |ident| {
            ctx.tokens.insert(
                ctx.ident(ident),
                Token::from_parsed(ParsedAstToken::Ident(ident.clone()), SymbolKind::Module),
            );
        });
        ctx.tokens.insert(
            ctx.ident(&impl_self_or_trait.trait_name.suffix),
            Token::from_parsed(
                ParsedAstToken::Declaration(Declaration::ImplSelfOrTrait(*self)),
                SymbolKind::Trait,
            ),
        );
        if let TypeInfo::Custom {
            qualified_call_path,
            type_arguments,
        } = &&*ctx
            .engines
            .te()
            .get(impl_self_or_trait.implementing_for.type_id())
        {
            ctx.tokens.insert(
                ctx.ident(&qualified_call_path.call_path.suffix),
                Token::from_parsed(
                    ParsedAstToken::Declaration(Declaration::ImplSelfOrTrait(*self)),
                    SymbolKind::Struct,
                ),
            );
            if let Some(type_arguments) = type_arguments {
                adaptive_iter(type_arguments, |type_arg| type_arg.parse(ctx));
            }
        } else {
            impl_self_or_trait.implementing_for.parse(ctx);
        }

        adaptive_iter(&impl_self_or_trait.impl_type_parameters, |type_param| {
            type_param.parse(ctx);
        });
        adaptive_iter(&impl_self_or_trait.items, |item| match item {
            ImplItem::Fn(fn_decl) => fn_decl.parse(ctx),
            ImplItem::Constant(const_decl) => const_decl.parse(ctx),
            ImplItem::Type(type_decl) => type_decl.parse(ctx),
        });
    }
}

impl Parse for ParsedDeclId<AbiDeclaration> {
    fn parse(&self, ctx: &ParseContext) {
        let abi_decl = ctx.engines.pe().get_abi(self);
        ctx.tokens.insert(
            ctx.ident(&abi_decl.name),
            Token::from_parsed(
                ParsedAstToken::Declaration(Declaration::AbiDeclaration(*self)),
                SymbolKind::Trait,
            ),
        );
        adaptive_iter(&abi_decl.interface_surface, |item| match item {
            TraitItem::TraitFn(trait_fn) => trait_fn.parse(ctx),
            TraitItem::Constant(const_decl) => const_decl.parse(ctx),
            TraitItem::Type(type_decl) => type_decl.parse(ctx),
            TraitItem::Error(_, _) => {}
        });
        adaptive_iter(&abi_decl.supertraits, |supertrait| supertrait.parse(ctx));
        abi_decl.attributes.parse(ctx);
    }
}

impl Parse for ParsedDeclId<ConstantDeclaration> {
    fn parse(&self, ctx: &ParseContext) {
        let const_decl = ctx.engines.pe().get_constant(self);
        ctx.tokens.insert(
            ctx.ident(&const_decl.name),
            Token::from_parsed(
                ParsedAstToken::Declaration(Declaration::ConstantDeclaration(*self)),
                SymbolKind::Const,
            ),
        );
        const_decl.type_ascription.parse(ctx);
        if let Some(value) = &const_decl.value {
            value.parse(ctx);
        }
        const_decl.attributes.parse(ctx);
    }
}

impl Parse for ParsedDeclId<ConfigurableDeclaration> {
    fn parse(&self, ctx: &ParseContext) {
        let const_decl = ctx.engines.pe().get_configurable(self);
        ctx.tokens.insert(
            ctx.ident(&const_decl.name),
            Token::from_parsed(
                ParsedAstToken::Declaration(Declaration::ConfigurableDeclaration(*self)),
                SymbolKind::Const,
            ),
        );
        const_decl.type_ascription.parse(ctx);
        if let Some(value) = &const_decl.value {
            value.parse(ctx);
        }
        const_decl.attributes.parse(ctx);
    }
}

impl Parse for ParsedDeclId<ConstGenericDeclaration> {
    fn parse(&self, ctx: &ParseContext) {
        let const_decl = ctx.engines.pe().get_const_generic(self);
        ctx.tokens.insert(
            ctx.ident(&const_decl.name),
            Token::from_parsed(
                ParsedAstToken::Declaration(Declaration::ConstGenericDeclaration(*self)),
                SymbolKind::Const,
            ),
        );
    }
}

impl Parse for ParsedDeclId<TraitTypeDeclaration> {
    fn parse(&self, ctx: &ParseContext) {
        let trait_type_decl = ctx.engines.pe().get_trait_type(self);
        ctx.tokens.insert(
            ctx.ident(&trait_type_decl.name),
            Token::from_parsed(
                ParsedAstToken::Declaration(Declaration::TraitTypeDeclaration(*self)),
                SymbolKind::TraitType,
            ),
        );
        if let Some(ty) = &trait_type_decl.ty_opt {
            ty.parse(ctx);
        }
        trait_type_decl.attributes.parse(ctx);
    }
}

impl Parse for ParsedDeclId<StorageDeclaration> {
    fn parse(&self, ctx: &ParseContext) {
        let storage_decl = ctx.engines.pe().get_storage(self);
        adaptive_iter(&storage_decl.entries, |entry| entry.parse(ctx));
        storage_decl.attributes.parse(ctx);
    }
}

impl Parse for StorageEntry {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            StorageEntry::Namespace(namespace) => namespace.parse(ctx),
            StorageEntry::Field(field) => field.parse(ctx),
        }
    }
}

impl Parse for StorageNamespace {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            ctx.ident(&self.name),
            Token::from_parsed(
                ParsedAstToken::StorageNamespace(self.clone()),
                SymbolKind::Field,
            ),
        );
        self.entries.iter().for_each(|entry| entry.parse(ctx));
    }
}

impl Parse for StorageField {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            ctx.ident(&self.name),
            Token::from_parsed(
                ParsedAstToken::StorageField(self.clone()),
                SymbolKind::Field,
            ),
        );
        self.type_argument.parse(ctx);
        self.initializer.parse(ctx);
        self.attributes.parse(ctx);
    }
}

impl Parse for Supertrait {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            ctx.ident(&self.name.suffix),
            Token::from_parsed(ParsedAstToken::Supertrait(self.clone()), SymbolKind::Trait),
        );
    }
}

impl Parse for ParsedDeclId<TraitFn> {
    fn parse(&self, ctx: &ParseContext) {
        let trait_fn = ctx.engines.pe().get_trait_fn(self);
        ctx.tokens.insert(
            ctx.ident(&trait_fn.name),
            Token::from_parsed(ParsedAstToken::TraitFn(*self), SymbolKind::Function),
        );
        adaptive_iter(&trait_fn.parameters, |param| param.parse(ctx));
        trait_fn.return_type.parse(ctx);
        trait_fn.attributes.parse(ctx);
    }
}

impl Parse for TraitConstraint {
    fn parse(&self, ctx: &ParseContext) {
        adaptive_iter(&self.trait_name.prefixes, |prefix| {
            ctx.tokens.insert(
                ctx.ident(prefix),
                Token::from_parsed(ParsedAstToken::Ident(prefix.clone()), SymbolKind::Function),
            );
        });
        ctx.tokens.insert(
            ctx.ident(&self.trait_name.suffix),
            Token::from_parsed(
                ParsedAstToken::TraitConstraint(self.clone()),
                SymbolKind::Function,
            ),
        );
        adaptive_iter(&self.type_arguments, |type_arg| type_arg.parse(ctx));
    }
}

impl Parse for FunctionParameter {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            ctx.ident(&self.name),
            Token::from_parsed(
                ParsedAstToken::FunctionParameter(self.clone()),
                SymbolKind::ValueParam,
            ),
        );
        self.type_argument.parse(ctx);
    }
}

impl Parse for StructField {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            ctx.ident(&self.name),
            Token::from_parsed(ParsedAstToken::StructField(self.clone()), SymbolKind::Field),
        );
        self.type_argument.parse(ctx);
        self.attributes.parse(ctx);
    }
}

impl Parse for EnumVariant {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert(
            ctx.ident(&self.name),
            Token::from_parsed(
                ParsedAstToken::EnumVariant(self.clone()),
                SymbolKind::Variant,
            ),
        );
        self.type_argument.parse(ctx);
        self.attributes.parse(ctx);
    }
}

impl Parse for TypeParameter {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            TypeParameter::Type(p) => {
                ctx.tokens.insert(
                    ctx.ident(&p.name),
                    Token::from_parsed(
                        ParsedAstToken::TypeParameter(self.clone()),
                        SymbolKind::TypeParameter,
                    ),
                );
            }
            TypeParameter::Const(_) => {}
        }
    }
}

impl Parse for GenericArgument {
    fn parse(&self, ctx: &ParseContext) {
        let type_info = ctx.engines.te().get(self.type_id());
        match &*type_info {
            TypeInfo::Array(type_arg, length) => {
                let ident = Ident::new(length.expr().span());
                ctx.tokens.insert(
                    ctx.ident(&ident),
                    Token::from_parsed(
                        ParsedAstToken::Ident(ident.clone()),
                        SymbolKind::NumericLiteral,
                    ),
                );
                type_arg.parse(ctx);
            }
            TypeInfo::Slice(type_arg) => {
                type_arg.parse(ctx);
            }
            TypeInfo::Tuple(type_arguments) => {
                adaptive_iter(type_arguments, |type_arg| type_arg.parse(ctx));
            }
            _ => {
                let symbol_kind = type_info_to_symbol_kind(ctx.engines.te(), &type_info, None);
                if let Some(tree) = &self.call_path_tree() {
                    let token =
                        Token::from_parsed(ParsedAstToken::TypeArgument(self.clone()), symbol_kind);
                    collect_call_path_tree(ctx, tree, &token, ctx.tokens);
                }
            }
        }
    }
}

impl Parse for ParsedDeclId<TypeAliasDeclaration> {
    fn parse(&self, ctx: &ParseContext) {
        let type_alias_decl = ctx.engines.pe().get_type_alias(self);
        ctx.tokens.insert(
            ctx.ident(&type_alias_decl.name),
            Token::from_parsed(
                ParsedAstToken::Declaration(Declaration::TypeAliasDeclaration(*self)),
                SymbolKind::TypeAlias,
            ),
        );
        type_alias_decl.ty.parse(ctx);
        type_alias_decl.attributes.parse(ctx);
    }
}

fn collect_type_info_token(ctx: &ParseContext, type_info: &TypeInfo, type_span: Option<&Span>) {
    let symbol_kind = type_info_to_symbol_kind(ctx.engines.te(), type_info, type_span);
    match type_info {
        TypeInfo::StringArray(length) => {
            let ident = Ident::new(length.expr().span());
            ctx.tokens.insert(
                ctx.ident(&ident),
                Token::from_parsed(ParsedAstToken::Ident(ident.clone()), symbol_kind),
            );
        }
        TypeInfo::Array(type_arg, length) => {
            let ident = Ident::new(length.expr().span());
            ctx.tokens.insert(
                ctx.ident(&ident),
                Token::from_parsed(
                    ParsedAstToken::Ident(ident.clone()),
                    SymbolKind::NumericLiteral,
                ),
            );
            type_arg.parse(ctx);
        }
        TypeInfo::Tuple(type_arguments) => {
            adaptive_iter(type_arguments, |type_arg| type_arg.parse(ctx));
        }
        TypeInfo::Custom {
            qualified_call_path,
            type_arguments,
        } => {
            collect_qualified_path_root(ctx, qualified_call_path.qualified_path_root.clone());
            let ident = qualified_call_path.call_path.suffix.clone();
            let mut token = Token::from_parsed(ParsedAstToken::Ident(ident.clone()), symbol_kind);
            token.type_def = Some(TypeDefinition::Ident(ident.clone()));
            ctx.tokens.insert(ctx.ident(&ident), token);
            if let Some(type_arguments) = type_arguments {
                adaptive_iter(type_arguments, |type_arg| type_arg.parse(ctx));
            }
        }
        _ => {
            if let Some(type_span) = type_span {
                let ident = Ident::new(type_span.clone());
                ctx.tokens.insert(
                    ctx.ident(&ident),
                    Token::from_parsed(ParsedAstToken::Ident(ident.clone()), symbol_kind),
                );
            }
        }
    }
}

fn collect_call_path_tree(
    ctx: &ParseContext,
    tree: &CallPathTree,
    token: &Token,
    tokens: &TokenMap,
) {
    collect_qualified_path_root(ctx, tree.qualified_call_path.qualified_path_root.clone());

    adaptive_iter(&tree.qualified_call_path.call_path.prefixes, |ident| {
        tokens.insert(ctx.ident(ident), token.clone());
    });
    tokens.insert(
        ctx.ident(&tree.qualified_call_path.call_path.suffix),
        token.clone(),
    );
    adaptive_iter(&tree.children, |child| {
        collect_call_path_tree(ctx, child, token, tokens);
    });
}

fn collect_qualified_path_root(
    ctx: &ParseContext,
    qualified_path_root: Option<Box<QualifiedPathType>>,
) {
    if let Some(qualified_path_root) = qualified_path_root {
        qualified_path_root.ty.parse(ctx);
        collect_type_info_token(
            ctx,
            &ctx.engines.te().get(qualified_path_root.as_trait),
            Some(&qualified_path_root.as_trait_span),
        );
    }
}

fn literal_to_symbol_kind(value: &Literal) -> SymbolKind {
    match value {
        Literal::U8(..)
        | Literal::U16(..)
        | Literal::U32(..)
        | Literal::U64(..)
        | Literal::U256(..)
        | Literal::Numeric(..) => SymbolKind::NumericLiteral,
        Literal::String(..) => SymbolKind::StringLiteral,
        Literal::B256(..) => SymbolKind::ByteLiteral,
        Literal::Boolean(..) => SymbolKind::BoolLiteral,
    }
}
