use crate::core::token::SymbolKind;
use sway_core::{
    type_system::{TypeId, TypeInfo},
    TypeEngine,
};
use sway_types::{Ident, Span, Spanned};
use tower_lsp::lsp_types::{Position, Range};

// Check if the given method is a `core::ops` application desugared from short-hand syntax like / + * - etc.
pub(crate) fn desugared_op(prefixes: &[Ident]) -> bool {
    let prefix0 = prefixes.get(0).map(|ident| ident.as_str());
    let prefix1 = prefixes.get(1).map(|ident| ident.as_str());
    if let (Some("core"), Some("ops")) = (prefix0, prefix1) {
        return true;
    }

    false
}

// We need to do this work around as the custom PartialEq for Ident impl
// only checks for the string, not the span.
pub(crate) fn to_ident_key(ident: &Ident) -> (Ident, Span) {
    (ident.clone(), ident.span())
}

/// Use the TypeId to look up the associated TypeInfo and return the Ident if one is found.
pub(crate) fn ident_of_type_id(type_engine: &TypeEngine, type_id: &TypeId) -> Option<Ident> {
    match type_engine.look_up_type_id(*type_id) {
        TypeInfo::UnknownGeneric { name, .. }
        | TypeInfo::Enum { name, .. }
        | TypeInfo::Struct { name, .. }
        | TypeInfo::Custom { name, .. } => Some(name),
        _ => None,
    }
}

pub(crate) fn type_info_to_symbol_kind(
    type_engine: &TypeEngine,
    type_info: &TypeInfo,
) -> SymbolKind {
    match type_info {
        TypeInfo::UnsignedInteger(..) | TypeInfo::Boolean | TypeInfo::Str(..) | TypeInfo::B256 => {
            SymbolKind::BuiltinType
        }
        TypeInfo::Numeric => SymbolKind::NumericLiteral,
        TypeInfo::Custom { .. } | TypeInfo::Struct { .. } => SymbolKind::Struct,
        TypeInfo::Enum { .. } => SymbolKind::Enum,
        TypeInfo::Array(elem_ty, _) => {
            let type_info = type_engine.look_up_type_id(elem_ty.type_id);
            type_info_to_symbol_kind(type_engine, &type_info)
        }
        _ => SymbolKind::Unknown,
    }
}

/// Given a `Span`, convert into an `lsp_types::Range` and return.
pub(crate) fn get_range_from_span(span: &Span) -> Range {
    let start = span.start_pos().line_col();
    let end = span.end_pos().line_col();

    let start_line = start.0 as u32 - 1;
    let start_character = start.1 as u32 - 1;

    let end_line = end.0 as u32 - 1;
    let end_character = end.1 as u32 - 1;

    Range {
        start: Position::new(start_line, start_character),
        end: Position::new(end_line, end_character),
    }
}
