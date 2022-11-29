use crate::{
    core::{
        session::Session,
        token::{Token, TypedAstToken},
    },
    utils::{
        attributes::doc_attributes, markdown, markup::Markup, token::get_range_from_span,
        token::to_ident_key,
    },
};
use std::sync::Arc;
use sway_core::{
    declaration_engine,
    language::{ty, Visibility},
    TypeEngine, TypeId,
};
use sway_types::{Ident, Span, Spanned};
use tower_lsp::lsp_types::{self, Position, Url};

/// Extracts the hover information for a token at the current position.
pub fn hover_data(session: Arc<Session>, url: Url, position: Position) -> Option<lsp_types::Hover> {
    let (ident, token) = session.token_map().token_at_position(&url, position)?;
    let range = get_range_from_span(&ident.span());
    let (decl_ident, decl_token) = match token.declared_token_ident(&session.type_engine.read()) {
        Some(decl_ident) => {
            let decl_token = session
                .token_map()
                .get(&to_ident_key(&decl_ident))
                .map(|item| item.value().clone())?;
            (decl_ident, decl_token)
        }
        // The `TypeInfo` of the token does not contain an `Ident`. In this case,
        // we use the `Ident` of the token itself.
        None => (ident, token),
    };

    let contents = hover_format(&session.type_engine.read(), &decl_token, &decl_ident);
    Some(lsp_types::Hover {
        contents,
        range: Some(range),
    })
}

fn visibility_as_str(visibility: &Visibility) -> &'static str {
    match visibility {
        Visibility::Private => "",
        Visibility::Public => "pub ",
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

fn format_visibility_hover(visibility: Visibility, decl_name: &str, token_name: &str) -> String {
    format!(
        "{}{} {}",
        visibility_as_str(&visibility),
        decl_name,
        token_name
    )
}

fn format_variable_hover(is_mutable: bool, type_name: &str, token_name: &str) -> String {
    let mutability = match is_mutable {
        false => "",
        true => " mut",
    };
    format!("let{} {}: {}", mutability, token_name, type_name)
}

fn markup_content(markup: Markup) -> lsp_types::MarkupContent {
    let kind = lsp_types::MarkupKind::Markdown;
    let value = markdown::format_docs(markup.as_str());
    lsp_types::MarkupContent { kind, value }
}

fn hover_format(
    type_engine: &TypeEngine,
    token: &Token,
    ident: &Ident,
) -> lsp_types::HoverContents {
    let token_name: String = ident.as_str().into();
    let doc_comment = format_doc_attributes(token);

    let format_name_with_type = |name: &str, type_id: &TypeId| -> String {
        let type_name = format!("{}", type_engine.help_out(type_id));
        format!("{}: {}", name, type_name)
    };

    let value = token
        .typed
        .as_ref()
        .and_then(|typed_token| match typed_token {
            TypedAstToken::TypedDeclaration(decl) => match decl {
                ty::TyDeclaration::VariableDeclaration(var_decl) => {
                    let type_name = format!("{}", type_engine.help_out(var_decl.type_ascription));
                    Some(format_variable_hover(
                        var_decl.mutability.is_mutable(),
                        &type_name,
                        &token_name,
                    ))
                }
                ty::TyDeclaration::StructDeclaration(decl_id) => {
                    declaration_engine::de_get_struct(decl_id.clone(), &decl.span())
                        .map(|struct_decl| {
                            format_visibility_hover(
                                struct_decl.visibility,
                                decl.friendly_name(),
                                &token_name,
                            )
                        })
                        .ok()
                }
                ty::TyDeclaration::TraitDeclaration(ref decl_id) => {
                    declaration_engine::de_get_trait(decl_id.clone(), &decl.span())
                        .map(|trait_decl| {
                            format_visibility_hover(
                                trait_decl.visibility,
                                decl.friendly_name(),
                                &token_name,
                            )
                        })
                        .ok()
                }
                ty::TyDeclaration::EnumDeclaration(decl_id) => {
                    declaration_engine::de_get_enum(decl_id.clone(), &decl.span())
                        .map(|enum_decl| {
                            format_visibility_hover(
                                enum_decl.visibility,
                                decl.friendly_name(),
                                &token_name,
                            )
                        })
                        .ok()
                }
                _ => None,
            },
            TypedAstToken::TypedFunctionDeclaration(func) => {
                Some(extract_fn_signature(&func.span()))
            }
            TypedAstToken::TypedFunctionParameter(param) => {
                Some(format_name_with_type(param.name.as_str(), &param.type_id))
            }
            TypedAstToken::TypedStructField(field) => {
                Some(format_name_with_type(field.name.as_str(), &field.type_id))
            }
            TypedAstToken::TypedExpression(expr) => match expr.expression {
                ty::TyExpressionVariant::Literal { .. } => {
                    Some(format!("{}", type_engine.help_out(expr.return_type)))
                }
                _ => None,
            },
            _ => None,
        });

    let content = Markup::new().maybe_add_sway_block(value).text(&doc_comment);

    lsp_types::HoverContents::Markup(markup_content(content))
}
