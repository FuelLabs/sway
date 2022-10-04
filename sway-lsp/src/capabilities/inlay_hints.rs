use crate::core::{
    session::Session,
    token::{SymbolKind, Token, TypedAstToken},
};
use sway_core::{
    type_system::TypeInfo,
    TypedDeclaration
};
use sway_types::Spanned;
use tower_lsp::lsp_types::{
    self, Range, InlayHintParams, Url,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InlayHintsConfig {
    /// Whether to render leading colons for type hints, and trailing colons for parameter hints.
    pub render_colons: bool,
    /// Whether to show inlay type hints for variables.
    pub type_hints: bool,
    /// Maximum length for inlay hints. Set to null to have an unlimited length.
    pub max_length: Option<usize>,
}

// Future PR's will add more kinds
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InlayKind {
    TypeHint,
}

#[derive(Debug)]
pub struct InlayHint {
    pub range: Range,
    pub kind: InlayKind,
    pub label: String,
}

pub(crate) fn inlay_hints(session: &Session, uri: &Url, range: &Range, config: &InlayHintsConfig) -> Option<Vec<lsp_types::InlayHint>> {    
    // 1. Loop through all our tokens and filter out all tokens that aren't TypedVariableDeclaration tokens
    // 2. Also filter out all tokens that have a span that fall outside of the provide range
    // 3. Filter out all variable tokens that have a type_ascription
    // 4. Look up the type id for the remaining tokens
    // 5. Convert the type into a string

    let hints: Vec<lsp_types::InlayHint> = session.tokens_for_file(uri)
        .iter()
        .filter_map(|item| {
            let token = item.value();
            match &token.typed {
                Some(t) => match t {
                    TypedAstToken::TypedDeclaration(decl) => match decl {
                        TypedDeclaration::VariableDeclaration(var_decl) => {
                            match var_decl.type_ascription_span {
                                Some(_) => None,
                                None => {
                                    let var_range = crate::utils::common::get_range_from_span(&var_decl.name.span());
                                    if var_range.start >= range.start && var_range.end <= range.end {
                                        Some(var_decl.clone())
                                    } else {
                                        None
                                    }
                                }
                            }
                        }
                        _ => None,
                    }
                    _ => None,
                }
                None => None,
            }
        })
        .filter_map(|var| {
            let type_info = sway_core::type_system::look_up_type_id(var.type_ascription);
            match type_info {
                TypeInfo::Numeric | TypeInfo::Unknown | TypeInfo::UnknownGeneric { .. } => None,
                _ => Some(var)
            }
        })
        .map(|var| {
            let range = crate::utils::common::get_range_from_span(&var.name.span());
            let kind = InlayKind::TypeHint;
            let label = format!("{}",var.type_ascription);
            let inlay_hint = InlayHint {
                range,
                kind,
                label,
            };
            self::inlay_hint(config.render_colons, inlay_hint)
        }).collect();
    
    Some(hints)

    // let v = document.get_token_map()
    //     .iter()
    //     .map(|((ident, span), token)| {
    //         let range = crate::utils::common::get_range_from_span(span);
    //         let kind = InlayKind::TypeHint;
    //         //let label = "$$$$".to_string();

    //         let label = match crate::core::traverse_typed_tree::get_type_id(token) {
    //             Some(type_id) => {
    //                 tracing::info!("type_id = {:#?}", type_id);

    //                 // Use the TypeId to look up the actual type (I think there is a method in the type_engine for this)
    //                 let type_info = sway_core::type_engine::look_up_type_id(type_id);
    //                 tracing::info!("type_info = {:#?}", type_info);
    //                 type_info.friendly_type_str()
    //             }
    //             None => "".to_string()
    //         };
    //         let inlay_hint = InlayHint {
    //             range,
    //             kind,
    //             label,
    //         };
    //         self::inlay_hint(config.render_colons, inlay_hint)
    // }).collect();

    //return Ok(Some(v));
    

    // iter over all tokens in out token_map.
    // filter_map? all tokens that are outside of the params.range 
    // map the remaining tokens into an LSP InlayHint
    // collect all these into a vector 
    // return
}

pub(crate) fn inlay_hint(
    render_colons: bool,
    inlay_hint: InlayHint,
) -> lsp_types::InlayHint {
    lsp_types::InlayHint {
        position: match inlay_hint.kind {
            // after annotated thing
            InlayKind::TypeHint => inlay_hint.range.end,
        },
        label: lsp_types::InlayHintLabel::String(match inlay_hint.kind {
            InlayKind::TypeHint if render_colons => format!(": {}", inlay_hint.label),
            _ => inlay_hint.label.to_string(),
        }),
        kind: match inlay_hint.kind {
            InlayKind::TypeHint => Some(lsp_types::InlayHintKind::TYPE)
        },
        tooltip: None,
        padding_left: Some(match inlay_hint.kind {
            InlayKind::TypeHint => !render_colons,
        }),
        padding_right: Some(match inlay_hint.kind {
            InlayKind::TypeHint => false,
        }),
        text_edits: None,
        data: None,
    }
}

impl Default for InlayHintsConfig {
    fn default() -> Self {
        Self {
            render_colons: true,
            type_hints: true,
            max_length: Some(25),
        }
    }
}