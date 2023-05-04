use crate::{
    core::{
        session::Session,
        token::{get_range_from_span, to_ident_key, AstToken, SymbolKind, Token, TypedAstToken},
        token_map::TokenMap,
        token_map_ext::TokenMapExt,
    },
    utils::{
        attributes::doc_comment_attributes, document::get_url_from_span, keyword_docs::KeywordDocs,
        markdown, markup::Markup,
    },
};
use serde::{Deserialize, Serialize};
use std::{any::Any, sync::Arc};
use sway_core::{
    language::{
        parsed::{AstNode, AstNodeContent, Declaration, ImplSelf, ImplTrait},
        ty::{self, TyDecl, TyTraitDecl},
        CallPath, Visibility,
    },
    Engines, TypeId, TypeInfo,
};

use sway_types::{Ident, Named, Span, Spanned};
use tower_lsp::lsp_types::{self, Location, Position, Range, Url};

#[derive(Debug, Clone)]
pub struct RelatedType {
    pub name: String,
    pub uri: Url,
    pub range: Range,
    pub callpath: CallPath,
}

#[derive(Debug, Clone)]
pub struct HoverLinkContents<'a> {
    pub related_types: Vec<RelatedType>,
    pub implementations: Vec<Span>,
    session: Arc<Session>,
    engines: Engines<'a>,
    token_map: &'a TokenMap,
    token: &'a Token,
}

impl<'a> HoverLinkContents<'a> {
    fn new(
        session: Arc<Session>,
        engines: Engines<'a>,
        token_map: &'a TokenMap,
        token: &'a Token,
    ) -> Self {
        Self {
            related_types: Vec::new(),
            implementations: Vec::new(),
            session,
            engines,
            token_map,
            token,
        }
    }

    fn add_related_type(&mut self, name: String, span: &Span, callpath: CallPath) {
        if let Ok(uri) = get_url_from_span(&span) {
            let range = get_range_from_span(&span);
            self.related_types.push(RelatedType {
                name,
                uri,
                range,
                callpath,
            });
        };
    }

    fn add_related_types(&mut self, type_id: &TypeId) {
        let type_info = self.engines.te().get(type_id.clone());
        match type_info {
            TypeInfo::Enum(decl_ref) => {
                let decl = self.engines.de().get_enum(&decl_ref);
                self.add_related_type(decl_ref.name().to_string(), &decl.span(), decl.call_path);
                decl.type_parameters
                    .iter()
                    .for_each(|type_param| self.add_related_types(&type_param.type_id));
            }
            TypeInfo::Struct(decl_ref) => {
                let decl = self.engines.de().get_struct(&decl_ref);
                self.add_related_type(decl_ref.name().to_string(), &decl.span(), decl.call_path);
                decl.type_parameters
                    .iter()
                    .for_each(|type_param| self.add_related_types(&type_param.type_id));
            }
            _ => {}
        }
    }

    fn add_implementations_for_trait(&mut self, trait_decl: &TyTraitDecl) {
        self.implementations.push(trait_decl.span());
        let mut impl_spans = self
            .session
            .impl_spans_for_trait_name(&trait_decl.name)
            .unwrap_or_default();
        self.implementations.append(&mut impl_spans);
    }

    fn add_implementations_for_decl(&mut self, ty_decl: &TyDecl) {
        self.implementations.push(ty_decl.span());
        let mut impl_spans = self
            .session
            .impl_spans_for_decl(ty_decl)
            .unwrap_or_default();
        self.implementations.append(&mut impl_spans);
    }

    fn add_implementations_for_type(&mut self, definition_span: Span, type_id: TypeId) {
        self.implementations.push(definition_span.clone());
        let mut impl_spans = self
            .session
            .impl_spans_for_type(type_id)
            .unwrap_or_default();
        self.implementations.append(&mut impl_spans);
    }
}

/// Extracts the hover information for a token at the current position.
pub fn hover_data(
    session: Arc<Session>,
    keyword_docs: &KeywordDocs,
    url: Url,
    position: Position,
) -> Option<lsp_types::Hover> {
    let (ident, token) = session.token_map().token_at_position(&url, position)?;
    let range = get_range_from_span(&ident.span());

    // check if our token is a keyword
    if matches!(
        token.kind,
        SymbolKind::BoolLiteral | SymbolKind::Keyword | SymbolKind::SelfKeyword
    ) {
        let name = ident.as_str();
        let documentation = keyword_docs.get(name).unwrap();
        let prefix = format!("\n```sway\n{name}\n```\n\n---\n\n");
        let formatted_doc = format!("{prefix}{documentation}");
        let content = Markup::new().text(&formatted_doc);
        let contents = lsp_types::HoverContents::Markup(markup_content(content));
        return Some(lsp_types::Hover {
            contents,
            range: Some(range),
        });
    }

    let te = session.type_engine.read();
    let de = session.decl_engine.read();
    let engines = Engines::new(&te, &de);
    let (decl_ident, decl_token) = match token.declared_token_ident(engines) {
        Some(decl_ident) => {
            let decl_token = session
                .token_map()
                .try_get(&to_ident_key(&decl_ident))
                .try_unwrap()
                .map(|item| item.value().clone())?;
            (decl_ident, decl_token)
        }
        // The `TypeInfo` of the token does not contain an `Ident`. In this case,
        // we use the `Ident` of the token itself.
        None => (ident, token),
    };

    let contents = hover_format(
        session.clone(),
        engines,
        session.token_map(),
        &decl_token,
        &decl_ident,
    );
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
    if let Some(attributes) = doc_comment_attributes(token) {
        doc_comment = attributes
            .iter()
            .map(|attribute| {
                let comment = attribute.args.first().unwrap().name.as_str();
                format!("{comment}\n")
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
    format!("let{mutability} {token_name}: {type_name}")
}

fn markup_content(markup: Markup) -> lsp_types::MarkupContent {
    let kind = lsp_types::MarkupKind::Markdown;
    let value = markdown::format_docs(markup.as_str());
    lsp_types::MarkupContent { kind, value }
}

fn hover_format(
    session: Arc<Session>,
    engines: Engines<'_>,
    token_map: &TokenMap,
    token: &Token,
    ident: &Ident,
) -> lsp_types::HoverContents {
    let decl_engine = engines.de();

    let token_name: String = ident.as_str().into();
    let doc_comment = format_doc_attributes(token);

    let format_name_with_type = |name: &str, type_id: &TypeId| -> String {
        let type_name = format!("{}", engines.help_out(type_id));
        format!("{name}: {type_name}")
    };

    // Used to collect all the information we need to generate links for the hover component.
    let mut hover_link_contents = HoverLinkContents::new(session, engines, token_map, token);

    let sway_block = token
        .typed
        .as_ref()
        .and_then(|typed_token| match typed_token {
            TypedAstToken::TypedDeclaration(decl) => match decl {
                ty::TyDecl::VariableDecl(var_decl) => {
                    let type_name =
                        format!("{}", engines.help_out(var_decl.type_ascription.type_id));
                    hover_link_contents.add_related_types(&var_decl.type_ascription.type_id);
                    Some(format_variable_hover(
                        var_decl.mutability.is_mutable(),
                        &type_name,
                        &token_name,
                    ))
                }
                ty::TyDecl::StructDecl(ty::StructDecl { decl_id, .. }) => {
                    let struct_decl = decl_engine.get_struct(decl_id);
                    hover_link_contents.add_implementations_for_decl(decl);
                    Some(format_visibility_hover(
                        struct_decl.visibility,
                        decl.friendly_type_name(),
                        &token_name,
                    ))
                }
                ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. }) => {
                    let trait_decl = decl_engine.get_trait(decl_id);
                    hover_link_contents.add_implementations_for_trait(&trait_decl);
                    Some(format_visibility_hover(
                        trait_decl.visibility,
                        decl.friendly_type_name(),
                        &token_name,
                    ))
                }
                ty::TyDecl::EnumDecl(ty::EnumDecl { decl_id, .. }) => {
                    let enum_decl = decl_engine.get_enum(decl_id);
                    hover_link_contents.add_implementations_for_decl(decl);
                    Some(format_visibility_hover(
                        enum_decl.visibility,
                        decl.friendly_type_name(),
                        &token_name,
                    ))
                }
                ty::TyDecl::AbiDecl(ty::AbiDecl { .. }) => {
                    hover_link_contents.add_implementations_for_decl(decl);
                    Some(format!("{} {}", decl.friendly_type_name(), &token_name))
                }
                _ => None,
            },
            TypedAstToken::TypedFunctionDeclaration(func) => {
                hover_link_contents.add_related_types(&func.return_type.type_id);
                Some(extract_fn_signature(&func.span()))
            }
            TypedAstToken::TypedFunctionParameter(param) => {
                hover_link_contents.add_related_types(&param.type_argument.type_id);
                Some(format_name_with_type(
                    param.name.as_str(),
                    &param.type_argument.type_id,
                ))
            }
            TypedAstToken::TypedStructField(field) => {
                hover_link_contents.add_related_types(&field.type_argument.type_id);
                Some(format_name_with_type(
                    field.name.as_str(),
                    &field.type_argument.type_id,
                ))
            }
            TypedAstToken::TypedExpression(expr) => match expr.expression {
                ty::TyExpressionVariant::Literal { .. } => {
                    Some(format!("{}", engines.help_out(expr.return_type)))
                }
                _ => None,
            },
            _ => None,
        });

    let content = Markup::new()
        .maybe_add_sway_block(sway_block)
        .text(&doc_comment)
        .maybe_add_links(
            hover_link_contents.related_types,
            hover_link_contents.implementations,
        );

    lsp_types::HoverContents::Markup(markup_content(content))
}
