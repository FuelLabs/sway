use crate::{
    core::token::{desugared_op, type_info_to_symbol_kind, AstToken, SymbolKind, TypeDefinition},
    traverse::{Parse, ParseContext},
};
use sway_core::{
    language::{
        parsed::{
            AbiCastExpression, AbiDeclaration, AmbiguousPathExpression, ArrayExpression,
            ArrayIndexExpression, AstNode, AstNodeContent, CodeBlock, ConstantDeclaration,
            Declaration, DelineatedPathExpression, EnumDeclaration, EnumVariant, Expression,
            ExpressionKind, FunctionApplicationExpression, FunctionDeclaration, FunctionParameter,
            ImplSelf, ImplTrait, IntrinsicFunctionExpression, LazyOperatorExpression, MatchBranch,
            MethodApplicationExpression, MethodName, ReassignmentExpression, ReassignmentTarget,
            Scrutinee, StorageDeclaration, StorageField, StructDeclaration, StructExpression,
            StructExpressionField, StructField, StructScrutineeField, SubfieldExpression,
            Supertrait, TraitDeclaration, TraitFn, VariableDeclaration,
        },
        Literal,
    },
    transform::{AttributeKind, AttributesMap},
    type_system::{TraitConstraint, TypeArgument, TypeParameter},
    AbiName, TypeInfo,
};
use sway_types::constants::{DESTRUCTURE_PREFIX, MATCH_RETURN_VAR_NAME_PREFIX, TUPLE_NAME_PREFIX};
use sway_types::{Ident, Spanned};

pub fn parse(node: &AstNode, ctx: &ParseContext) {
    node.parse(ctx);
}

impl Parse for AstNode {
    fn parse(&self, ctx: &ParseContext) {
        match &self.content {
            AstNodeContent::Declaration(declaration) => declaration.parse(ctx),
            AstNodeContent::Expression(expression)
            | AstNodeContent::ImplicitReturnExpression(expression) => expression.parse(ctx),
            // TODO
            // handle other content types
            _ => {}
        };
    }
}

impl Parse for Declaration {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            Declaration::VariableDeclaration(variable) => variable.parse(ctx),
            Declaration::FunctionDeclaration(func_decl) => func_decl.parse(ctx),
            Declaration::TraitDeclaration(trait_decl) => trait_decl.parse(ctx),
            Declaration::StructDeclaration(struct_dec) => struct_dec.parse(ctx),
            Declaration::EnumDeclaration(enum_decl) => enum_decl.parse(ctx),
            Declaration::ImplTrait(impl_trait) => impl_trait.parse(ctx),
            Declaration::ImplSelf(impl_self) => impl_self.parse(ctx),
            Declaration::AbiDeclaration(abi_decl) => abi_decl.parse(ctx),
            Declaration::ConstantDeclaration(const_decl) => const_decl.parse(ctx),
            Declaration::StorageDeclaration(storage_decl) => storage_decl.parse(ctx),
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
                let symbol_kind = literal_to_symbol_kind(&value);
                ctx.tokens.insert_parsed(
                    Ident::new(self.span()),
                    AstToken::Literal(value.clone()),
                    symbol_kind,
                );
            }
            ExpressionKind::FunctionApplication(function_application_expression) => {
                function_application_expression.parse(ctx)
            }
            ExpressionKind::LazyOperator(LazyOperatorExpression { lhs, rhs, .. }) => {
                lhs.parse(ctx);
                rhs.parse(ctx);
            }
            ExpressionKind::Variable(name) => {
                if !name.as_str().contains(TUPLE_NAME_PREFIX)
                    && !name.as_str().contains(MATCH_RETURN_VAR_NAME_PREFIX)
                {
                    let symbol_kind = symbol_kind_from_ident(&name);
                    ctx.tokens.insert_parsed(
                        name.clone(),
                        AstToken::Ident(name.clone()),
                        symbol_kind,
                    );
                }
            }
            ExpressionKind::Tuple(fields) => {
                fields.iter().for_each(|field| field.parse(ctx));
            }
            ExpressionKind::TupleIndex(tuple_index_exp) => {
                tuple_index_exp.prefix.parse(ctx);
                let ident = Ident::new(tuple_index_exp.index_span.clone());
                ctx.tokens.insert_parsed(
                    ident.clone(),
                    AstToken::Ident(ident.clone()),
                    SymbolKind::NumericLiteral,
                );
            }
            ExpressionKind::Array(array_expression) => {
                array_expression.parse(ctx);
            }
            ExpressionKind::Struct(struct_expression) => {
                struct_expression.parse(ctx);
            }
            ExpressionKind::CodeBlock(code_block) => {
                code_block.parse(ctx);
            }
            ExpressionKind::If(if_exp) => {
                if_exp.condition.parse(ctx);
                if_exp.then.parse(ctx);
                if let Some(r#else) = &if_exp.r#else {
                    r#else.parse(ctx);
                }
            }
            ExpressionKind::Match(match_exp) => {
                match_exp.value.parse(ctx);
                match_exp
                    .branches
                    .iter()
                    .for_each(|branch| branch.parse(ctx));
            }
            ExpressionKind::Asm(_) => {
                //TODO handle asm expressions
            }
            ExpressionKind::MethodApplication(method_application_expression) => {
                method_application_expression.parse(ctx);
            }
            ExpressionKind::Subfield(SubfieldExpression {
                prefix,
                field_to_access,
                ..
            }) => {
                ctx.tokens.insert_parsed(
                    field_to_access.clone(),
                    AstToken::Ident(field_to_access.clone()),
                    SymbolKind::Field,
                );
                prefix.parse(ctx);
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
            ExpressionKind::StorageAccess(storage_access_exp) => {
                storage_access_exp
                    .field_names
                    .iter()
                    .for_each(|field_name| {
                        ctx.tokens.insert_parsed(
                            field_name.clone(),
                            AstToken::Ident(field_name.clone()),
                            SymbolKind::Field,
                        );
                    });
            }
            ExpressionKind::IntrinsicFunction(IntrinsicFunctionExpression {
                name,
                kind_binding,
                arguments,
            }) => {
                ctx.tokens.insert_parsed(
                    name.clone(),
                    AstToken::Intrinsic(kind_binding.inner.clone()),
                    SymbolKind::Function,
                );
                arguments.iter().for_each(|argument| argument.parse(ctx));
            }
            ExpressionKind::WhileLoop(while_exp) => {
                while_exp.body.parse(ctx);
                while_exp.condition.parse(ctx);
            }
            // These keyword tokens are already collected in during lexing traversal.
            ExpressionKind::Break | ExpressionKind::Continue => {}
            ExpressionKind::Reassignment(reassignment) => {
                reassignment.parse(ctx);
            }
            ExpressionKind::Return(expr) => expr.parse(ctx),
        }
    }
}

impl Parse for VariableDeclaration {
    fn parse(&self, ctx: &ParseContext) {
        // Don't collect tokens if the ident's name contains __tuple_ || __match_return_var_name_
        // The individual elements are handled in the subsequent VariableDeclaration's
        if !self.name.as_str().contains(TUPLE_NAME_PREFIX)
            && !self.name.as_str().contains(MATCH_RETURN_VAR_NAME_PREFIX)
        {
            let symbol_kind = symbol_kind_from_ident(&self.name);

            // We want to use the span from variable.name to construct a
            // new Ident as the name_override_opt can be set to one of the
            // const prefixes and not the actual token name.
            ctx.tokens.insert_parsed(
                Ident::new(self.name.span()),
                AstToken::VariableDeclaration(self.clone()),
                symbol_kind,
            );

            if self.type_ascription_span.is_some() {
                self.type_ascription.parse(ctx);
            }
        }
        self.body.parse(ctx);
    }
}

impl Parse for FunctionDeclaration {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert_parsed(
            self.name.clone(),
            AstToken::FunctionDeclaration(self.clone()),
            SymbolKind::Function,
        );
        self.body.contents.iter().for_each(|node| node.parse(ctx));
        self.parameters
            .iter()
            .for_each(|func_param| func_param.parse(ctx));
        self.type_parameters
            .iter()
            .for_each(|type_param| type_param.parse(ctx));
        self.return_type.parse(ctx);
        self.attributes.parse(ctx);
    }
}

impl Parse for TraitDeclaration {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert_parsed(
            self.name.clone(),
            AstToken::TraitDeclaration(self.clone()),
            SymbolKind::Trait,
        );
        self.interface_surface
            .iter()
            .for_each(|trait_fn| trait_fn.parse(ctx));
        self.methods
            .iter()
            .for_each(|func_decl| func_decl.parse(ctx));
        self.supertraits
            .iter()
            .for_each(|supertrait| supertrait.parse(ctx));
    }
}

impl Parse for StructDeclaration {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert_parsed(
            self.name.clone(),
            AstToken::StructDeclaration(self.clone()),
            SymbolKind::Struct,
        );
        self.fields.iter().for_each(|field| field.parse(ctx));
        self.type_parameters.iter().for_each(|type_param| {
            type_param.parse(ctx);
        });
        self.attributes.parse(ctx);
    }
}

impl Parse for EnumDeclaration {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert_parsed(
            self.name.clone(),
            AstToken::EnumDeclaration(self.clone()),
            SymbolKind::Enum,
        );
        self.type_parameters.iter().for_each(|type_param| {
            type_param.parse(ctx);
        });
        self.variants.iter().for_each(|variant| variant.parse(ctx));
        self.attributes.parse(ctx);
    }
}

impl Parse for ImplTrait {
    fn parse(&self, ctx: &ParseContext) {
        self.trait_name.prefixes.iter().for_each(|prefix| {
            ctx.tokens.insert_parsed(
                prefix.clone(),
                AstToken::Ident(prefix.clone()),
                SymbolKind::Module,
            );
        });
        ctx.tokens.insert_parsed(
            self.trait_name.suffix.clone(),
            AstToken::ImplTrait(self.clone()),
            SymbolKind::Trait,
        );

        self.type_implementing_for.parse(ctx);
        self.impl_type_parameters.iter().for_each(|type_param| {
            type_param.parse(ctx);
        });
        self.functions
            .iter()
            .for_each(|func_decl| func_decl.parse(ctx));
    }
}

impl Parse for ImplSelf {
    fn parse(&self, ctx: &ParseContext) {
        self.type_implementing_for.parse(ctx);
        self.impl_type_parameters.iter().for_each(|type_param| {
            type_param.parse(ctx);
        });
        self.functions
            .iter()
            .for_each(|func_decl| func_decl.parse(ctx));
    }
}

impl Parse for AbiDeclaration {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert_parsed(
            self.name.clone(),
            AstToken::AbiDeclaration(self.clone()),
            SymbolKind::Trait,
        );
        self.interface_surface
            .iter()
            .for_each(|trait_fn| trait_fn.parse(ctx));
        self.attributes.parse(ctx);
    }
}

impl Parse for ConstantDeclaration {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert_parsed(
            self.name.clone(),
            AstToken::ConstantDeclaration(self.clone()),
            SymbolKind::Const,
        );
        self.type_ascription.parse(ctx);
        self.value.parse(ctx);
        self.attributes.parse(ctx);
    }
}

impl Parse for StorageDeclaration {
    fn parse(&self, ctx: &ParseContext) {
        self.fields.iter().for_each(|field| field.parse(ctx));
        self.attributes.parse(ctx);
    }
}

impl Parse for FunctionApplicationExpression {
    fn parse(&self, ctx: &ParseContext) {
        // Don't collect applications of desugared operators due to mismatched ident lengths.
        if !desugared_op(&self.call_path_binding.inner.prefixes) {
            self.call_path_binding
                .inner
                .prefixes
                .iter()
                .for_each(|prefix| {
                    ctx.tokens.insert_parsed(
                        prefix.clone(),
                        AstToken::Ident(prefix.clone()),
                        SymbolKind::Module,
                    );
                });
            ctx.tokens.insert_parsed(
                self.call_path_binding.inner.suffix.clone(),
                AstToken::FunctionApplicationExpression(self.clone()),
                SymbolKind::Function,
            );
            self.call_path_binding
                .type_arguments
                .iter()
                .for_each(|type_arg| {
                    type_arg.parse(ctx);
                });
        }
        self.arguments.iter().for_each(|arg| arg.parse(ctx));
    }
}

impl Parse for CodeBlock {
    fn parse(&self, ctx: &ParseContext) {
        self.contents.iter().for_each(|node| node.parse(ctx));
    }
}

impl Parse for StorageField {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert_parsed(
            self.name.clone(),
            AstToken::StorageField(self.clone()),
            SymbolKind::Field,
        );
        self.type_info.parse(ctx);
        self.initializer.parse(ctx);
        self.attributes.parse(ctx);
    }
}

impl Parse for FunctionParameter {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert_parsed(
            self.name.clone(),
            AstToken::FunctionParameter(self.clone()),
            SymbolKind::ValueParam,
        );
        self.type_info.parse(ctx);
    }
}

impl Parse for TraitFn {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert_parsed(
            self.name.clone(),
            AstToken::TraitFn(self.clone()),
            SymbolKind::Function,
        );
        self.parameters.iter().for_each(|param| param.parse(ctx));
        self.return_type.parse(ctx);
        self.attributes.parse(ctx);
    }
}

impl Parse for StructField {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert_parsed(
            self.name.clone(),
            AstToken::StructField(self.clone()),
            SymbolKind::Field,
        );
        self.type_info.parse(ctx);
        self.attributes.parse(ctx);
    }
}

impl Parse for StructExpression {
    fn parse(&self, ctx: &ParseContext) {
        self.call_path_binding
            .inner
            .prefixes
            .iter()
            .for_each(|prefix| {
                ctx.tokens.insert_parsed(
                    prefix.clone(),
                    AstToken::Ident(prefix.clone()),
                    SymbolKind::Struct,
                );
            });
        ctx.tokens.insert_parsed(
            self.call_path_binding.inner.suffix.clone(),
            AstToken::StructExpression(self.clone()),
            SymbolKind::Struct,
        );
        self.call_path_binding
            .type_arguments
            .iter()
            .for_each(|type_arg| {
                type_arg.parse(ctx);
            });
        self.fields.iter().for_each(|field| field.parse(ctx));
    }
}

impl Parse for StructExpressionField {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert_parsed(
            self.name.clone(),
            AstToken::StructExpressionField(self.clone()),
            SymbolKind::Field,
        );
        self.value.parse(ctx);
    }
}

impl Parse for ArrayExpression {
    fn parse(&self, ctx: &ParseContext) {
        self.contents.iter().for_each(|expr| expr.parse(ctx));
        if let Some(length_span) = &self.length_span {
            let ident = Ident::new(length_span.clone());
            ctx.tokens.insert_parsed(
                ident.clone(),
                AstToken::Ident(ident.clone()),
                SymbolKind::NumericLiteral,
            );
        }
    }
}

impl Parse for EnumVariant {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert_parsed(
            self.name.clone(),
            AstToken::EnumVariant(self.clone()),
            SymbolKind::Variant,
        );
        self.type_info.parse(ctx);
        self.attributes.parse(ctx);
    }
}

impl Parse for Scrutinee {
    fn parse(&self, ctx: &ParseContext) {
        match self {
            Scrutinee::CatchAll { .. } => (),
            Scrutinee::Literal { ref value, span } => {
                ctx.tokens.insert_parsed(
                    Ident::new(span.clone()),
                    AstToken::Scrutinee(self.clone()),
                    literal_to_symbol_kind(value),
                );
            }
            Scrutinee::Variable { name, .. } => {
                ctx.tokens.insert_parsed(
                    name.clone(),
                    AstToken::Scrutinee(self.clone()),
                    SymbolKind::Variable,
                );
            }
            Scrutinee::StructScrutinee {
                struct_name,
                fields,
                ..
            } => {
                ctx.tokens.insert_parsed(
                    struct_name.clone(),
                    AstToken::Scrutinee(self.clone()),
                    SymbolKind::Struct,
                );
                fields.iter().for_each(|field| field.parse(ctx));
            }
            Scrutinee::EnumScrutinee {
                call_path, value, ..
            } => {
                call_path.prefixes.iter().for_each(|prefix| {
                    ctx.tokens.insert_parsed(
                        prefix.clone(),
                        AstToken::Ident(prefix.clone()),
                        SymbolKind::Enum,
                    );
                });
                ctx.tokens.insert_parsed(
                    call_path.suffix.clone(),
                    AstToken::Scrutinee(self.clone()),
                    SymbolKind::Variant,
                );
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
        if let StructScrutineeField::Field {
            field, scrutinee, ..
        } = self
        {
            ctx.tokens.insert_parsed(
                field.clone(),
                AstToken::StructScrutineeField(self.clone()),
                SymbolKind::Field,
            );
            if let Some(scrutinee) = scrutinee {
                scrutinee.parse(ctx);
            }
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
            let (type_info, ..) = &call_path_binding.inner.suffix;
            type_info.parse(ctx);
        }

        // Don't collect applications of desugared operators due to mismatched ident lengths.
        if !desugared_op(&prefixes) {
            ctx.tokens.insert_parsed(
                self.method_name_binding.inner.easy_name(),
                AstToken::MethodApplicationExpression(self.clone()),
                SymbolKind::Struct,
            );
        }
        self.arguments.iter().for_each(|arg| arg.parse(ctx));
        self.contract_call_params
            .iter()
            .for_each(|param| param.parse(ctx));
    }
}

impl Parse for MatchBranch {
    fn parse(&self, ctx: &ParseContext) {
        self.scrutinee.parse(ctx);
        self.result.parse(ctx);
    }
}

impl Parse for AmbiguousPathExpression {
    fn parse(&self, ctx: &ParseContext) {
        self.call_path_binding
            .inner
            .prefixes
            .iter()
            .chain(std::iter::once(
                &self.call_path_binding.inner.suffix.before.inner,
            ))
            .for_each(|ident| {
                ctx.tokens.insert_parsed(
                    ident.clone(),
                    AstToken::Ident(ident.clone()),
                    SymbolKind::Enum,
                );
            });
        ctx.tokens.insert_parsed(
            self.call_path_binding.inner.suffix.suffix.clone(),
            AstToken::AmbiguousPathExpression(self.clone()),
            SymbolKind::Variant,
        );
        self.call_path_binding
            .type_arguments
            .iter()
            .for_each(|type_arg| {
                type_arg.parse(ctx);
            });
        self.args.iter().for_each(|arg| arg.parse(ctx));
    }
}

impl Parse for DelineatedPathExpression {
    fn parse(&self, ctx: &ParseContext) {
        self.call_path_binding
            .inner
            .prefixes
            .iter()
            .for_each(|ident| {
                ctx.tokens.insert_parsed(
                    ident.clone(),
                    AstToken::Ident(ident.clone()),
                    SymbolKind::Enum,
                );
            });
        ctx.tokens.insert_parsed(
            self.call_path_binding.inner.suffix.clone(),
            AstToken::DelineatedPathExpression(self.clone()),
            SymbolKind::Variant,
        );
        self.call_path_binding
            .type_arguments
            .iter()
            .for_each(|type_arg| {
                type_arg.parse(ctx);
            });
        self.args.iter().for_each(|arg| arg.parse(ctx));
    }
}

impl Parse for AbiCastExpression {
    fn parse(&self, ctx: &ParseContext) {
        self.abi_name.prefixes.iter().for_each(|ident| {
            ctx.tokens.insert_parsed(
                ident.clone(),
                AstToken::Ident(ident.clone()),
                SymbolKind::Module,
            );
        });
        ctx.tokens.insert_parsed(
            self.abi_name.suffix.clone(),
            AstToken::AbiCastExpression(self.clone()),
            SymbolKind::Trait,
        );
        self.address.parse(ctx);
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
                idents.iter().for_each(|ident| {
                    ctx.tokens.insert_parsed(
                        ident.clone(),
                        AstToken::ReassignmentExpression(self.clone()),
                        SymbolKind::Field,
                    );
                });
            }
        }
    }
}

impl Parse for Supertrait {
    fn parse(&self, ctx: &ParseContext) {
        ctx.tokens.insert_parsed(
            self.name.suffix.clone(),
            AstToken::Supertrait(self.clone()),
            SymbolKind::Trait,
        );
    }
}

impl Parse for AttributesMap {
    fn parse(&self, ctx: &ParseContext) {
        self.iter()
            .filter(|(kind, ..)| **kind != AttributeKind::DocComment)
            .flat_map(|(.., attrs)| attrs)
            .for_each(|attribute| {
                ctx.tokens.insert_parsed(
                    attribute.name.clone(),
                    AstToken::Attribute(attribute.clone()),
                    SymbolKind::DeriveHelper,
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

fn symbol_kind_from_ident(ident: &Ident) -> SymbolKind {
    if ident.as_str().contains(DESTRUCTURE_PREFIX) {
        SymbolKind::Struct
    } else {
        SymbolKind::Variable
    }
}
