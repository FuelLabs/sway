use crate::{
    core::{
        session::Session,
        token::{AstToken, Token, TypedAstToken},
    },
    utils::{
        attributes::doc_attributes,
        common::get_range_from_span,
        markdown,
        markup::Markup, 
        token::to_ident_key,
    },
};
use std::sync::Arc;
use sway_core::{
    declaration_engine,
    language::{parsed::Declaration, ty, Visibility},
};
use sway_types::{Ident, Span, Spanned};
use tower_lsp::lsp_types::{self, Position, Url};

/// Extracts the hover information for a token at the current position.
pub fn hover_data(session: Arc<Session>, url: Url, position: Position) -> Option<lsp_types::Hover> {
    let (ident, token) = session.token_at_position(&url, position)?;
    let range = get_range_from_span(&ident.span());
    let decl_ident = session.declared_token_ident(&token)?;
    let decl_token = session
        .token_map()
        .get(&to_ident_key(&decl_ident))
        .map(|item| item.value().clone())?;
    let contents = hover_format(&decl_token, &decl_ident);
    Some(lsp_types::Hover {
        contents,
        range: Some(range),
    })
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

fn format_doc_attributes(token: &Token) -> String {
    let mut doc_comment = String::new();
    if let Some(attributes) = doc_attributes(token) {
        doc_comment = attributes
            .iter()
            .map(|attribute| {
                let comment = attribute.args.first().unwrap().as_str();
                format!("{}\n", comment)
            })
            .collect()
    }
    doc_comment
}

fn markup_content(markup: Markup) -> lsp_types::MarkupContent {
    let kind = lsp_types::MarkupKind::Markdown;
    let value = markdown::format_docs(markup.as_str());
    lsp_types::MarkupContent { kind, value }
}

fn hover_format(token: &Token, ident: &Ident) -> lsp_types::HoverContents {
    let token_name: String = ident.as_str().into();
    let doc_comment = format_doc_attributes(token);

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

    // TODO implement this properly in a future PR
    let _value = match &token.typed {
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

    lsp_types::HoverContents::Markup(markup_content(Markup::from(doc_comment)))
}
