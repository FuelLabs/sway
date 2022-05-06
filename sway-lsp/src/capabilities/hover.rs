use crate::{
    core::{
        session::{Documents, Session},
        token::Token,
        token_type::{TokenType, VarBody},
    },
    utils::common::extract_visibility,
};
use std::sync::Arc;
use tower_lsp::lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind};

pub fn get_hover_data(session: Arc<Session>, params: HoverParams) -> Option<Hover> {
    let position = params.text_document_position_params.position;
    let url = &params.text_document_position_params.text_document.uri;

    match session.documents.get(url.path()) {
        Some(ref document) => {
            if let Some(token) = document.get_token_at_position(position) {
                if token.is_initial_declaration() {
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

fn get_hover_format(token: &Token, documents: &Documents) -> Hover {
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
        TokenType::FunctionDeclaration(func_details) => func_details.signature.clone(),
        TokenType::StructDeclaration(struct_details) => format!(
            "{}struct {}",
            extract_visibility(&struct_details.visibility),
            &token.name
        ),
        TokenType::TraitDeclaration(trait_details) => format!(
            "{}trait {}",
            extract_visibility(&trait_details.visibility),
            &token.name
        ),
        TokenType::EnumDeclaration(enum_details) => format!(
            "{}enum {}",
            extract_visibility(&enum_details.visibility),
            &token.name
        ),
        _ => token.name.clone(),
    };

    Hover {
        contents: HoverContents::Markup(MarkupContent {
            value: format!("```sway\n{}\n```", value),
            kind: MarkupKind::Markdown,
        }),
        range: Some(token.range),
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
