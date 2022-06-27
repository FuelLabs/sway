use crate::{
    core::{
        session::{Documents, Session},
        token::{AstToken, TokenType, TypedAstToken},
    },
    utils::{
        common::{extract_visibility, get_range_from_span},
        function::extract_fn_signature,
        token::is_initial_declaration,
    },
};

use std::sync::Arc;
use tower_lsp::lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind};

use sway_core::{semantic_analysis::ast_node::TypedDeclaration, Declaration, Visibility};
use sway_types::{Ident, Spanned};

pub fn get_hover_data(session: Arc<Session>, params: HoverParams) -> Option<Hover> {
    let position = params.text_document_position_params.position;
    let url = &params.text_document_position_params.text_document.uri;

    match session.documents.get(url.path()) {
        Some(ref document) => {
            if let Some(token) = document.get_token_at_position(position) {
                if is_initial_declaration(token) {
                    Some(get_hover_format(token, &session.documents))
                } else {
                    // todo: this logic is flawed at the moment
                    // if there are multiple tokens with the same name and type in different files
                    // there is no way for us to know which one is currently used in here
                    for document_ref in &session.documents {
                        if let Some(declared_token) = document_ref.get_declared_token(&token.name) {
                            if declared_token.is_same_type(token) {
                                return Some(get_hover_format(declared_token, &session.documents));
                            }
                        }
                    }
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_hover_format(token: &TokenType, ident: &Ident, documents: &Documents) -> Hover {
    let token_name: String = ident.as_str().into();
    let range = get_range_from_span(&ident.span());

    let format_visibility_hover = |visibility: Visibility, decl_name: &str| -> String {
        format!(
            "{}{} {}",
            extract_visibility(&visibility),
            decl_name,
            token_name
        )
    };

    let value = match token.typed {
        Some(typed_token) => match typed_token {
            TypedAstToken::TypedDeclaration(decl) => match decl {
                TypedDeclaration::VariableDeclaration(var) => {
                    format!(
                        "let{} {}: {}",
                        if var_details.is_mutable { " mut" } else { "" },
                        token_name,
                        var_type
                    )
                }
                TypedDeclaration::FunctionDeclaration(func) => extract_fn_signature(&func.span()),
                TypedDeclaration::StructDeclaration(struct_decl) => {
                    format_visibility_hover(struct_decl.visibility, decl.friendly_name())
                }
                TypedDeclaration::TraitDeclaration(trait_decl) => {
                    format_visibility_hover(trait_decl.visibility, decl.friendly_name())
                }
                TypedDeclaration::EnumDeclaration(enum_decl) => {
                    format_visibility_hover(enum_decl.visibility, decl.friendly_name())
                }
                _ => token_name,
            },
            _ => token_name,
        },
        None => match token.parsed {
            AstToken::Declaration(decl) => match decl {
                Declaration::FunctionDeclaration(func) => extract_fn_signature(&func.span),
                Declaration::StructDeclaration(struct_decl) => {
                    format_visibility_hover(struct_decl.visibility, "struct")
                }
                Declaration::TraitDeclaration(trait_decl) => {
                    format_visibility_hover(trait_decl.visibility, "trait")
                }
                Declaration::EnumDeclaration(enum_decl) => {
                    format_visibility_hover(enum_decl.visibility, "enum")
                }
                _ => token_name,
            },
            _ => token_name,
        },
    };

    let value = match &token.token_type {
        TokenType::VariableDeclaration(var_details) => {
            let var_type = match &var_details.var_body {
                VarBody::FunctionCall(fn_name) => get_var_type_from_fn(fn_name, documents),
                VarBody::Type(var_type) => var_type.clone(),
                _ => "".into(),
            };

            format!(
                "let{} {}: {}",
                if var_details.is_mutable { " mut" } else { "" },
                token.name,
                var_type
            )
        }
        //TokenType::FunctionDeclaration(func_details) => func_details.signature.clone(),
        // TokenType::StructDeclaration(struct_details) => format!(
        //     "{}struct {}",
        //     extract_visibility(&struct_details.visibility),
        //     &token.name
        // ),
        // TokenType::TraitDeclaration(trait_details) => format!(
        //     "{}trait {}",
        //     extract_visibility(&trait_details.visibility),
        //     &token.name
        // ),
        // TokenType::EnumDeclaration(enum_details) => format!(
        //     "{}enum {}",
        //     extract_visibility(&enum_details.visibility),
        //     &token.name
        // ),
        //_ => token.name.clone(),
    };

    Hover {
        contents: HoverContents::Markup(MarkupContent {
            value: format!("```sway\n{}\n```", value),
            kind: MarkupKind::Markdown,
        }),
        range: Some(range),
    }
}

fn get_var_type_from_fn(fn_name: &str, documents: &Documents) -> String {
    for document_ref in documents {
        if let Some(declared_token) = document_ref.get_declared_token(fn_name) {
            if let TokenType::FunctionDeclaration(func_details) = &declared_token.token_type {
                return func_details
                    .get_return_type_from_signature()
                    .unwrap_or_default();
            }
        }
    }

    "".into()
}
