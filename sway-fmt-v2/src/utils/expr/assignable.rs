use crate::{
    fmt::*,
    utils::{
        bracket::SquareBracket,
        byte_span::{ByteSpan, LeafSpans},
    },
};
use std::fmt::Write;
use sway_ast::{expr::ReassignmentOp, Assignable, Expr};
use sway_types::Spanned;

impl Format for Assignable {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Assignable::Var(name) => {
                name.format(formatted_code, formatter)?;
            }
            Assignable::Index { target, arg } => {
                target.format(formatted_code, formatter)?;
                Expr::open_square_bracket(formatted_code, formatter)?;
                arg.get().format(formatted_code, formatter)?;
                Expr::close_square_bracket(formatted_code, formatter)?;
            }
            Assignable::FieldProjection {
                target,
                dot_token,
                name,
            } => {
                target.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", dot_token.span().as_str())?;
                name.format(formatted_code, formatter)?;
            }
            Assignable::TupleFieldProjection {
                target,
                dot_token,
                field: _,
                field_span,
            } => {
                target.format(formatted_code, formatter)?;
                write!(
                    formatted_code,
                    "{}{}",
                    dot_token.span().as_str(),
                    field_span.as_str()
                )?;
            }
        }
        Ok(())
    }
}

impl Format for ReassignmentOp {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, " {} ", self.span.as_str())?;
        Ok(())
    }
}

impl LeafSpans for Assignable {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        match self {
            Assignable::Var(var) => collected_spans.push(ByteSpan::from(var.span())),
            Assignable::Index { target, arg } => {
                collected_spans.append(&mut target.leaf_spans());
                collected_spans.append(&mut arg.leaf_spans());
            }
            Assignable::FieldProjection {
                target,
                dot_token,
                name,
            } => {
                collected_spans.append(&mut target.leaf_spans());
                collected_spans.push(ByteSpan::from(dot_token.span()));
                collected_spans.push(ByteSpan::from(name.span()));
            }
            Assignable::TupleFieldProjection {
                target,
                dot_token,
                field: _field,
                field_span,
            } => {
                collected_spans.append(&mut target.leaf_spans());
                collected_spans.push(ByteSpan::from(dot_token.span()));
                collected_spans.push(ByteSpan::from(field_span.clone()));
            }
        };
        collected_spans
    }
}
