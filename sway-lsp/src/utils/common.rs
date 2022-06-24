use crate::core::token_type::VarBody;
use crate::core::token::TokenMap;
use sway_core::{Expression, Literal, VariableDeclaration, Visibility};
use sway_types::{Ident, Span};
use tower_lsp::lsp_types::{Position, Range};

pub(crate) fn extract_visibility(visibility: &Visibility) -> String {
    match visibility {
        Visibility::Private => "".into(),
        Visibility::Public => "pub ".into(),
    }
}

pub(crate) fn extract_var_body(var_dec: &VariableDeclaration) -> VarBody {
    match &var_dec.body {
        Expression::FunctionApplication { name, .. } => {
            VarBody::FunctionCall(name.suffix.as_str().into())
        }
        Expression::StructExpression { struct_name, .. } => {
            VarBody::Type(struct_name.suffix.0.to_string())
        }
        Expression::Literal { value, .. } => match value {
            Literal::U8(_) => VarBody::Type("u8".into()),
            Literal::U16(_) => VarBody::Type("u16".into()),
            Literal::U32(_) => VarBody::Type("u32".into()),
            Literal::U64(_) => VarBody::Type("u64".into()),
            Literal::Numeric(_) => VarBody::Type("u64".into()),
            Literal::String(len) => VarBody::Type(format!("str[{}]", len.as_str().len())),
            Literal::Boolean(_) => VarBody::Type("bool".into()),
            Literal::Byte(_) => VarBody::Type("u8".into()),
            Literal::B256(_) => VarBody::Type("b256".into()),
        },
        _ => VarBody::Other,
    }
}

pub(crate) fn ident_and_span_at_position(
    cursor_position: Position,
    tokens: &TokenMap,
) -> Option<(Ident, Span)> {
    for (ident, span) in tokens.keys() {
        let range = get_range_from_span(span);
        if cursor_position >= range.start && cursor_position <= range.end {
            return Some((ident.clone(), span.clone()));
        }
    }
    None
}

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
