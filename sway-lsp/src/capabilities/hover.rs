use crate::{
    core::{
        session::Session,
        token::{AstToken, Token, TypedAstToken},
    },
    utils::{
        common::{extract_visibility, get_range_from_span},
        function::extract_fn_signature,
        token::to_ident_key,
    },
};
use sway_core::{
    declaration_engine, semantic_analysis::ast_node::TypedDeclaration, Declaration, Visibility,
};
use sway_types::{Ident, Spanned};
use tower_lsp::lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind};

pub fn hover_data(session: &Session, params: HoverParams) -> Option<Hover> {
    let position = params.text_document_position_params.position;
    let url = &params.text_document_position_params.text_document.uri;
    if let Some((_, token)) = session.token_at_position(url, position) {
        if let Some(decl_ident) = session.declared_token_ident(&token) {
            if let Some(decl_token) = session
                .token_map()
                .get(&to_ident_key(&decl_ident))
                .map(|item| item.value().clone())
            {
                let hover = hover_format(&decl_token, &decl_ident);
                return Some(hover);
            }
        }
    }
    None
}

fn hover_format(token: &Token, ident: &Ident) -> Hover {
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

    let format_variable_hover = |is_mutable: bool, type_name: String| -> String {
        let mutability = match is_mutable {
            false => "",
            true => " mut",
        };
        format!("let{} {}: {}", mutability, token_name, type_name,)
    };

    let value = match &token.typed {
        Some(typed_token) => match typed_token {
            TypedAstToken::TypedDeclaration(decl) => match decl {
                TypedDeclaration::VariableDeclaration(var_decl) => {
                    let type_name = format!("{}", var_decl.type_ascription);
                    format_variable_hover(var_decl.mutability.is_mutable(), type_name)
                }
                TypedDeclaration::FunctionDeclaration(func) => extract_fn_signature(&func.span()),
                TypedDeclaration::StructDeclaration(decl_id) => {
                    declaration_engine::de_get_struct(decl_id.clone(), &decl.span())
                        .map(|struct_decl| {
                            format_visibility_hover(struct_decl.visibility, decl.friendly_name())
                        })
                        .unwrap_or(token_name)
                }
                TypedDeclaration::TraitDeclaration(ref decl_id) => {
                    declaration_engine::de_get_trait(decl_id.clone(), &decl.span())
                        .map(|trait_decl| {
                            format_visibility_hover(trait_decl.visibility, decl.friendly_name())
                        })
                        .unwrap_or(token_name)
                }
                TypedDeclaration::EnumDeclaration(decl_id) => {
                    declaration_engine::de_get_enum(decl_id.clone(), &decl.span())
                        .map(|enum_decl| {
                            format_visibility_hover(enum_decl.visibility, decl.friendly_name())
                        })
                        .unwrap_or(token_name)
                }
                _ => token_name,
            },
            _ => token_name,
        },
        None => match &token.parsed {
            AstToken::Declaration(decl) => match decl {
                Declaration::VariableDeclaration(var_decl) => {
                    let type_name = format!("{}", var_decl.type_ascription);
                    format_variable_hover(var_decl.is_mutable, type_name)
                }
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

    Hover {
        contents: HoverContents::Markup(MarkupContent {
            value: format!("```sway\n{}\n```", value),
            kind: MarkupKind::Markdown,
        }),
        range: Some(range),
    }
}
