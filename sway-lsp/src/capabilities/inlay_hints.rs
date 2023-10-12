use crate::{
    config::InlayHintsConfig,
    core::{
        session::Session,
        token::{get_range_from_span, TypedAstToken},
    },
};
use lsp_types::{self, Range, Url};
use std::sync::Arc;
use sway_core::{
    language::ty::{TyDecl, TyVariableDecl},
    type_system::TypeInfo,
};
use sway_types::Spanned;

// Future PR's will add more kinds
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InlayKind {
    TypeHint,
    Parameter,
}

#[derive(Debug)]
pub struct InlayHint {
    pub range: Range,
    pub kind: InlayKind,
    pub label: String,
}

pub fn inlay_hints(
    session: Arc<Session>,
    uri: &Url,
    range: &Range,
    config: &InlayHintsConfig,
) -> Option<Vec<lsp_types::InlayHint>> {
    // 1. Loop through all our tokens and filter out all tokens that aren't TypedVariableDeclaration tokens
    // 2. Also filter out all tokens that have a span that fall outside of the provided range
    // 3. Filter out all variable tokens that have a type_ascription
    // 4. Look up the type id for the remaining tokens
    // 5. Convert the type into a string
    if !config.type_hints {
        return None;
    }

    let engines = session.engines.read();
    let hints: Vec<lsp_types::InlayHint> = session
        .token_map()
        .tokens_for_file(uri)
        // Filter out all tokens that have a span that fall outside of the provided range
        .filter_map(|(ident, token)| (ident.range.start >= range.start && ident.range.end <= range.end).then(|| (ident, token)))
        .filter_map(|(ident, token)| {
            token.typed.as_ref().and_then(|t| match t {
                TypedAstToken::TypedDeclaration(TyDecl::VariableDecl(var_decl)) => {
                    var_decl::hints(var_decl, ident.range, &config, &engines)
                }
                // TypedAstToken::TypedFunctionParameter(param) => {
                //     params::hints(param, &range)
                // }
                _ => None,
            })
        })
        .collect();

    Some(hints)
}

fn inlay_hint(render_colons: bool, inlay_hint: InlayHint) -> lsp_types::InlayHint {
    lsp_types::InlayHint {
        position: inlay_hint.range.end,
        label: lsp_types::InlayHintLabel::String(if render_colons {
            format!(": {}", inlay_hint.label)
        } else {
            inlay_hint.label
        }),
        kind: match inlay_hint.kind {
            InlayKind::TypeHint => Some(lsp_types::InlayHintKind::TYPE),
            InlayKind::Parameter => Some(lsp_types::InlayHintKind::PARAMETER),
        },
        tooltip: None,
        padding_left: Some(!render_colons),
        padding_right: Some(false),
        text_edits: None,
        data: None,
    }
}

mod var_decl {
    use sway_core::Engines;

    use super::*;

    pub fn hints(
        var_decl: &Box<TyVariableDecl>,
        range: Range,
        config: &InlayHintsConfig,
        engines: &Engines,
    ) -> Option<lsp_types::InlayHint> {
        if var_decl.type_ascription.call_path_tree.is_some() {
            return None;
        }
        match engines.te().get(var_decl.type_ascription.type_id) {
            TypeInfo::Unknown | TypeInfo::UnknownGeneric { .. } => None,
            _ => {
                let label = engines.help_out(&var_decl.type_ascription).to_string();
                let inlay_hint = InlayHint {
                    range,
                    kind: InlayKind::TypeHint,
                    label,
                };
                Some(self::inlay_hint(config.render_colons, inlay_hint))
            }
        }
    }
}
