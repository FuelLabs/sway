use crate::{
    core::{
        session::Session,
        token::{AstToken, Token, TypedAstToken},
    },
    utils::{common::get_range_from_span, token::to_ident_key},
};
use std::sync::Arc;
use sway_core::{
    declaration_engine,
    language::{parsed::Declaration, ty, Visibility},
};
use sway_types::{Ident, Span, Spanned};
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position, Url};

pub fn hover_data(session: Arc<Session>, url: Url, position: Position) -> Option<Hover> {
    if let Some((_, token)) = session.token_at_position(&url, position) {
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

fn visibility_as_str(visibility: &Visibility) -> &'static str {
    match visibility {
        Visibility::Private => "",
        Visibility::Public => "pub",
    }
}

/// Expects a span from either a `FunctionDeclaration` or a `TypedFunctionDeclaration`.
fn extract_fn_signature(span: &Span) -> String {
    let value = span.as_str();
    value.split('{').take(1).map(|v| v.trim()).collect()
}

fn hover_format(token: &Token, ident: &Ident) -> Hover {
    let token_name: String = ident.as_str().into();
    let range = get_range_from_span(&ident.span());

    let format_visibility_hover = |visibility: Visibility, decl_name: &str| -> String {
        format!(
            "{}{} {}",
            visibility_as_str(&visibility),
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
                ty::TyDeclaration::VariableDeclaration(var_decl) => {
                    let type_name = format!("{}", var_decl.type_ascription);
                    format_variable_hover(var_decl.mutability.is_mutable(), type_name)
                }
                ty::TyDeclaration::FunctionDeclaration(func) => extract_fn_signature(&func.span()),
                ty::TyDeclaration::StructDeclaration(decl_id) => {
                    declaration_engine::de_get_struct(decl_id.clone(), &decl.span())
                        .map(|struct_decl| {
                            format_visibility_hover(struct_decl.visibility, decl.friendly_name())
                        })
                        .unwrap_or(token_name)
                }
                ty::TyDeclaration::TraitDeclaration(ref decl_id) => {
                    declaration_engine::de_get_trait(decl_id.clone(), &decl.span())
                        .map(|trait_decl| {
                            format_visibility_hover(trait_decl.visibility, decl.friendly_name())
                        })
                        .unwrap_or(token_name)
                }
                ty::TyDeclaration::EnumDeclaration(decl_id) => {
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
