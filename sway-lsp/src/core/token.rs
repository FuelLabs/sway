use crate::{
    core::token_type::{
        get_const_details, get_enum_details, get_function_details, get_struct_details,
        get_struct_field_details, get_trait_details, TokenType, VariableDetails,
    },
    utils::common::{extract_var_body, get_range_from_span},
};
use sway_core::{
    constants::TUPLE_NAME_PREFIX, parse_tree::MethodName, type_engine::TypeInfo, AstNode,
    AstNodeContent, Declaration, Expression, FunctionDeclaration, FunctionParameter,
    IntrinsicFunctionKind, VariableDeclaration, WhileLoop,
};
use sway_types::{ident::Ident, span::Span, Spanned};
use tower_lsp::lsp_types::Range;

#[derive(Debug, Clone)]
pub struct Token {
    pub range: Range,
    pub token_type: TokenType,
    pub name: String,
    pub line_start: u32,
    pub length: u32,
}

impl Token {
    pub fn new(span: &Span, name: String, token_type: TokenType) -> Self {
        let range = get_range_from_span(span);

        Self {
            range,
            name,
            token_type,
            line_start: range.start.line,
            length: range.end.character - range.start.character,
        }
    }

    pub fn is_within_character_range(&self, character: u32) -> bool {
        let range = self.range;
        character >= range.start.character && character <= range.end.character
    }

    pub fn is_same_type(&self, other_token: &Token) -> bool {
        if other_token.token_type == self.token_type {
            true
        } else {
            matches!(
                (&other_token.token_type, &self.token_type),
                (
                    TokenType::FunctionApplication,
                    TokenType::FunctionDeclaration(_)
                ) | (
                    TokenType::FunctionDeclaration(_),
                    TokenType::FunctionApplication
                ),
            )
        }
    }

    pub fn get_line_start(&self) -> u32 {
        self.line_start
    }

    pub fn from_variable(var_dec: &VariableDeclaration) -> Self {
        let ident = &var_dec.name;
        let name = ident.as_str();
        let var_body = extract_var_body(var_dec);

        Token::new(
            &ident.span(),
            name.into(),
            TokenType::VariableDeclaration(VariableDetails {
                is_mutable: var_dec.is_mutable,
                var_body,
            }),
        )
    }

    pub fn from_ident(ident: &Ident, token_type: TokenType) -> Self {
        Token::new(&ident.span(), ident.as_str().into(), token_type)
    }

    pub fn from_span(span: Span, token_type: TokenType) -> Self {
        Token::new(&span, span.as_str().into(), token_type)
    }

    pub fn is_initial_declaration(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::VariableDeclaration(_)
                | TokenType::FunctionDeclaration(_)
                | TokenType::TraitDeclaration(_)
                | TokenType::StructDeclaration(_)
                | TokenType::EnumDeclaration(_)
                | TokenType::AbiDeclaration
                | TokenType::ConstantDeclaration(_)
                | TokenType::StorageFieldDeclaration
        )
    }
}

pub fn traverse_node(node: AstNode, tokens: &mut Vec<Token>) {
    match node.content {
        AstNodeContent::Declaration(dec) => handle_declaration(dec, tokens),
        AstNodeContent::Expression(exp) => handle_expression(exp, tokens),
        AstNodeContent::ImplicitReturnExpression(exp) => handle_expression(exp, tokens),
        AstNodeContent::ReturnStatement(return_statement) => {
            handle_expression(return_statement.expr, tokens)
        }
        AstNodeContent::WhileLoop(while_loop) => handle_while_loop(while_loop, tokens),
        // TODO
        // handle other content types
        _ => {}
    };
}

fn handle_function_parameter(parameter: &FunctionParameter, tokens: &mut Vec<Token>) {
    let ident = &parameter.name;
    let name = ident.as_str();

    tokens.push(Token::new(
        &ident.span(),
        name.into(),
        TokenType::FunctionParameter,
    ));
}

fn handle_function_declation(function_declaration: FunctionDeclaration, tokens: &mut Vec<Token>) {
    let ident = &function_declaration.name;
    let token = Token::from_ident(
        ident,
        TokenType::FunctionDeclaration(get_function_details(
            &function_declaration.span,
            function_declaration.visibility,
        )),
    );
    tokens.push(token);

    for param in function_declaration.parameters {
        handle_function_parameter(&param, tokens);
    }

    handle_custom_type(&function_declaration.return_type, tokens);

    for node in function_declaration.body.contents {
        traverse_node(node, tokens);
    }
}

fn handle_custom_type(type_info: &TypeInfo, tokens: &mut Vec<Token>) {
    if let TypeInfo::Custom { name, .. } = type_info {
        //Iterate through the tokens and find the first token that has the same name as the custom type.
        //Extract the token type of the found token, this should help determine if the custom type
        //is a struct or an enum.
        let found_token = tokens.iter().find(|token| token.name == name.as_str());
        if let Some(token_type) = found_token.map(|token| &token.token_type) {
            if let TokenType::StructDeclaration(_) = token_type {
                let token = Token::from_ident(name, TokenType::Struct);
                tokens.push(token);
            } else if let TokenType::EnumDeclaration(_) = token_type {
                let token = Token::from_ident(name, TokenType::EnumApplication);
                tokens.push(token);
            }
        }
    }
}

fn handle_declaration(declaration: Declaration, tokens: &mut Vec<Token>) {
    match declaration {
        Declaration::VariableDeclaration(variable) => {
            let name = variable.name.as_str();
            // Don't collect tokens if the ident's name contains __tuple_
            // The individual tuple elements are handled in the subsequent VariableDeclaration's
            if !name.contains(TUPLE_NAME_PREFIX) {
                tokens.push(Token::from_variable(&variable));
            }

            handle_expression(variable.body, tokens);
        }
        Declaration::FunctionDeclaration(func_dec) => {
            handle_function_declation(func_dec, tokens);
        }
        Declaration::TraitDeclaration(trait_dec) => {
            let ident = &trait_dec.name;
            let token = Token::from_ident(
                ident,
                TokenType::TraitDeclaration(get_trait_details(&trait_dec)),
            );
            tokens.push(token);

            for func_dec in trait_dec.methods {
                handle_function_declation(func_dec, tokens);
            }
        }
        Declaration::StructDeclaration(struct_dec) => {
            let ident = &struct_dec.name;
            let token = Token::from_ident(
                ident,
                TokenType::StructDeclaration(get_struct_details(&struct_dec)),
            );
            tokens.push(token);

            for field in struct_dec.fields {
                let token = Token::from_ident(
                    &field.name,
                    TokenType::StructField(get_struct_field_details(ident)),
                );
                tokens.push(token);
            }
        }
        Declaration::EnumDeclaration(enum_dec) => {
            let ident = &enum_dec.name;
            let token = Token::from_ident(
                ident,
                TokenType::EnumDeclaration(get_enum_details(&enum_dec)),
            );
            tokens.push(token);

            for variant in enum_dec.variants {
                let ident = &variant.name;
                let token = Token::from_ident(ident, TokenType::EnumVariant);
                tokens.push(token);
            }
        }
        Declaration::Reassignment(reassignment) => {
            let token_type = TokenType::Reassignment;
            let token = Token::from_span(reassignment.lhs_span(), token_type);
            tokens.push(token);
            handle_expression(reassignment.rhs, tokens);
        }
        Declaration::ImplTrait(impl_trait) => {
            let ident = impl_trait.trait_name.suffix;
            let token = Token::from_ident(&ident, TokenType::ImplTrait);
            tokens.push(token);

            for func_dec in impl_trait.functions {
                handle_function_declation(func_dec, tokens);
            }
        }
        Declaration::ImplSelf(impl_self) => {
            handle_custom_type(&impl_self.type_implementing_for, tokens);

            for func_dec in impl_self.functions {
                handle_function_declation(func_dec, tokens);
            }
        }
        Declaration::AbiDeclaration(abi_dec) => {
            let ident = &abi_dec.name;
            let token = Token::from_ident(ident, TokenType::AbiDeclaration);

            tokens.push(token);

            for func_dec in abi_dec.methods {
                handle_function_declation(func_dec, tokens);
            }

            for train_fn in abi_dec.interface_surface {
                let ident = &train_fn.name;
                let token = Token::from_ident(ident, TokenType::TraitFunction);
                tokens.push(token);

                for param in train_fn.parameters {
                    handle_function_parameter(&param, tokens);
                }

                handle_custom_type(&train_fn.return_type, tokens);
            }
        }
        Declaration::ConstantDeclaration(const_dec) => {
            let ident = &const_dec.name;
            let token = Token::from_ident(
                ident,
                TokenType::ConstantDeclaration(get_const_details(&const_dec)),
            );
            tokens.push(token);
        }
        Declaration::StorageDeclaration(storage_dec) => {
            for field in storage_dec.fields {
                let ident = &field.name;
                let token = Token::from_ident(ident, TokenType::StorageFieldDeclaration);
                tokens.push(token);
            }
        }
    };
}

fn handle_expression(exp: Expression, tokens: &mut Vec<Token>) {
    match exp {
        Expression::Literal { .. } => {}
        Expression::FunctionApplication {
            call_path_binding: type_binding,
            arguments,
            ..
        } => {
            let ident = type_binding.inner.suffix;
            let token = Token::from_ident(&ident, TokenType::FunctionApplication);
            tokens.push(token);

            for exp in arguments {
                handle_expression(exp, tokens);
            }
        }
        Expression::LazyOperator { lhs, rhs, .. } => {
            handle_expression(*lhs, tokens);
            handle_expression(*rhs, tokens);
        }
        Expression::VariableExpression { name, .. } => {
            if !name.as_str().contains(TUPLE_NAME_PREFIX) {
                let token = Token::from_ident(&name, TokenType::VariableExpression);
                tokens.push(token);
            }
        }
        Expression::Tuple { fields, .. } => {
            for exp in fields {
                handle_expression(exp, tokens);
            }
        }
        Expression::TupleIndex { prefix, .. } => {
            handle_expression(*prefix, tokens);
        }
        Expression::Array { contents, .. } => {
            for exp in contents {
                handle_expression(exp, tokens);
            }
        }
        Expression::StructExpression {
            call_path_binding: type_binding,
            fields,
            ..
        } => {
            let ident = type_binding.inner.suffix;
            let token = Token::from_ident(&ident, TokenType::Struct);
            tokens.push(token);

            for field in fields {
                let token = Token::from_ident(
                    &field.name,
                    TokenType::StructExpressionField(get_struct_field_details(&ident)),
                );
                tokens.push(token);
                handle_expression(field.value, tokens);
            }
        }
        Expression::CodeBlock { span: _, contents } => {
            let nodes = contents.contents;
            for node in nodes {
                traverse_node(node, tokens);
            }
        }
        Expression::IfExp {
            condition,
            then,
            r#else,
            ..
        } => {
            handle_expression(*condition, tokens);
            handle_expression(*then, tokens);
            if let Some(r#else) = r#else {
                handle_expression(*r#else, tokens);
            }
        }
        Expression::MatchExp {
            value, branches, ..
        } => {
            handle_expression(*value, tokens);
            for branch in branches {
                // TODO: handle_scrutinee(branch.scrutinee, tokens);
                handle_expression(branch.result, tokens);
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
            // Don't collect applications of desugared operators due to mismatched ident lengths.
            if !desugared_op(&method_name_binding.inner) {
                let ident = method_name_binding.inner.easy_name();
                let token = Token::from_ident(&ident, TokenType::MethodApplication);
                tokens.push(token);
            }

            for exp in arguments {
                handle_expression(exp, tokens);
            }

            //TODO handle methods from imported modules
            if let MethodName::FromType { type_name, .. } = &method_name_binding.inner {
                handle_custom_type(type_name, tokens);
            }

            for field in contract_call_params {
                let token = Token::from_ident(
                    &field.name,
                    TokenType::StructExpressionField(get_struct_field_details(
                        &method_name_binding.inner.easy_name(),
                    )),
                );
                tokens.push(token);
                handle_expression(field.value, tokens);
            }
        }
        Expression::SubfieldExpression { prefix, .. } => {
            handle_expression(*prefix, tokens);
            //TODO handle field_to_access?
        }
        Expression::DelineatedPath {
            call_path_binding: type_binding,
            arguments,
            ..
        } => {
            for prefix in type_binding.inner.prefixes {
                //TODO find the correct token type for this!
                let token = Token::from_ident(&prefix, TokenType::DelineatedPath);
                tokens.push(token);
            }

            let token = Token::from_ident(&type_binding.inner.suffix, TokenType::DelineatedPath);
            tokens.push(token);

            for exp in arguments {
                handle_expression(exp, tokens);
            }
        }
        Expression::AbiCast {
            abi_name, address, ..
        } => {
            let ident = abi_name.suffix;
            let token = Token::from_ident(&ident, TokenType::AbiCast);
            tokens.push(token);

            handle_expression(*address, tokens);
        }
        Expression::ArrayIndex { prefix, index, .. } => {
            handle_expression(*prefix, tokens);
            handle_expression(*index, tokens);
        }
        Expression::StorageAccess { field_names, .. } => {
            for field in field_names {
                let token = Token::from_ident(&field, TokenType::StorageAccess);
                tokens.push(token);
            }
        }
        Expression::IntrinsicFunction { kind, .. } => {
            handle_intrinsic_function(kind, tokens);
        }
    }
}

fn handle_intrinsic_function(kind: IntrinsicFunctionKind, tokens: &mut Vec<Token>) {
    match kind {
        IntrinsicFunctionKind::SizeOfVal { exp } => {
            handle_expression(*exp, tokens);
        }
        IntrinsicFunctionKind::SizeOfType { .. } => {}
        IntrinsicFunctionKind::IsRefType { .. } => {}
        IntrinsicFunctionKind::GetStorageKey => {}
    }
}

fn handle_while_loop(while_loop: WhileLoop, tokens: &mut Vec<Token>) {
    handle_expression(while_loop.condition, tokens);
    for node in while_loop.body.contents {
        traverse_node(node, tokens);
    }
}

// Check if the given method is a `core::ops` application desugared from short-hand syntax like / + * - etc.
fn desugared_op(method_name: &MethodName) -> bool {
    if let MethodName::FromType { ref call_path, .. } = method_name {
        let prefix0 = call_path.prefixes.get(0).map(|ident| ident.as_str());
        let prefix1 = call_path.prefixes.get(1).map(|ident| ident.as_str());
        if let (Some("core"), Some("ops")) = (prefix0, prefix1) {
            return true;
        }
    }
    false
}
