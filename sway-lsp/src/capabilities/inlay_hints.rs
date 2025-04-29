use crate::{
    config::InlayHintsConfig,
    core::{
        token::{get_range_from_span, TypedAstToken},
        token_map::TokenMap,
    },
};
use lsp_types::{self, Range, Url};
use sway_core::{
    language::ty::{TyDecl, TyExpression, TyExpressionVariant},
    type_system::TypeInfo, Engines,
};
use sway_types::{Ident, Spanned};

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

/// Generates inlay hints for the provided range.
pub fn inlay_hints(
    engines: &Engines,
    token_map: &TokenMap,
    uri: &Url,
    range: &Range,
    config: &InlayHintsConfig,
) -> Option<Vec<lsp_types::InlayHint>> {
    let _span = tracing::trace_span!("inlay_hints").entered();

    if !config.type_hints {
        return None;
    }

    // 1. Iterate through all tokens in the file
    // 2. Filter for TypedVariableDeclaration tokens within the provided range
    // 3. For each variable declaration:
    //    a. If it's a function application, generate parameter hints
    //    b. If it doesn't have a type ascription and its type is known:
    //       - Look up the type information
    //       - Generate a type hint
    // 4. Collect all generated hints into a single vector
    let hints: Vec<lsp_types::InlayHint> = token_map
        .tokens_for_file(uri)
        .filter_map(|item| {
            let token = item.value();
            token.as_typed().as_ref().and_then(|t| match t {
                TypedAstToken::TypedDeclaration(TyDecl::VariableDecl(var_decl)) => {
                    let var_range = get_range_from_span(&var_decl.name.span());
                    if var_range.start >= range.start && var_range.end <= range.end {
                        Some(var_decl.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            })
        })
        .flat_map(|var| {
            let mut hints = Vec::new();

            // Function parameter hints
            if let TyExpressionVariant::FunctionApplication { arguments, .. } = &var.body.expression
            {
                hints.extend(handle_function_parameters(arguments, config));
            }

            // Variable declaration hints
            if var.type_ascription.call_path_tree().is_none() {
                let type_info = engines
                    .te()
                    .get(var.type_ascription.type_id());
                if !matches!(
                    *type_info,
                    TypeInfo::Unknown | TypeInfo::UnknownGeneric { .. }
                ) {
                    let range = get_range_from_span(&var.name.span());
                    let kind = InlayKind::TypeHint;
                    let label = format!("{}", engines.help_out(var.type_ascription));
                    let inlay_hint = InlayHint { range, kind, label };
                    hints.push(self::inlay_hint(config, inlay_hint));
                }
            }
            hints
        })
        .collect();

    Some(hints)
}

fn handle_function_parameters(
    arguments: &[(Ident, TyExpression)],
    config: &InlayHintsConfig,
) -> Vec<lsp_types::InlayHint> {
    arguments
        .iter()
        .flat_map(|(name, exp)| {
            let mut hints = Vec::new();
            let (should_create_hint, span) = match &exp.expression {
                TyExpressionVariant::Literal(_)
                | TyExpressionVariant::ConstantExpression { .. }
                | TyExpressionVariant::Tuple { .. }
                | TyExpressionVariant::ArrayExplicit { .. }
                | TyExpressionVariant::ArrayIndex { .. }
                | TyExpressionVariant::FunctionApplication { .. }
                | TyExpressionVariant::StructFieldAccess { .. }
                | TyExpressionVariant::TupleElemAccess { .. } => (true, &exp.span),
                TyExpressionVariant::EnumInstantiation {
                    call_path_binding, ..
                } => (true, &call_path_binding.span),
                _ => (false, &exp.span),
            };
            if should_create_hint {
                let range = get_range_from_span(span);
                let kind = InlayKind::Parameter;
                let label = name.as_str().to_string();
                let inlay_hint = InlayHint { range, kind, label };
                hints.push(self::inlay_hint(config, inlay_hint));
            }
            // Handle nested function applications
            if let TyExpressionVariant::FunctionApplication {
                arguments: nested_args,
                ..
            } = &exp.expression
            {
                hints.extend(handle_function_parameters(nested_args, config));
            }
            hints
        })
        .collect::<Vec<_>>()
}

fn inlay_hint(config: &InlayHintsConfig, inlay_hint: InlayHint) -> lsp_types::InlayHint {
    let truncate_label = |label: String| -> String {
        if let Some(max_length) = config.max_length {
            if label.len() > max_length {
                format!("{}...", &label[..max_length.saturating_sub(3)])
            } else {
                label
            }
        } else {
            label
        }
    };

    let label = match inlay_hint.kind {
        InlayKind::TypeHint if config.render_colons => format!(": {}", inlay_hint.label),
        InlayKind::Parameter if config.render_colons => format!("{}: ", inlay_hint.label),
        _ => inlay_hint.label,
    };

    lsp_types::InlayHint {
        position: match inlay_hint.kind {
            // after annotated thing
            InlayKind::TypeHint => inlay_hint.range.end,
            InlayKind::Parameter => inlay_hint.range.start,
        },
        label: lsp_types::InlayHintLabel::String(truncate_label(label)),
        kind: match inlay_hint.kind {
            InlayKind::TypeHint => Some(lsp_types::InlayHintKind::TYPE),
            InlayKind::Parameter => Some(lsp_types::InlayHintKind::PARAMETER),
        },
        tooltip: None,
        padding_left: Some(match inlay_hint.kind {
            InlayKind::TypeHint => !config.render_colons,
            InlayKind::Parameter => false,
        }),
        padding_right: Some(match inlay_hint.kind {
            InlayKind::TypeHint => false,
            InlayKind::Parameter => !config.render_colons,
        }),
        text_edits: None,
        data: None,
    }
}
