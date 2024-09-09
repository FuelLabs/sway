use crate::{
    formatter::*,
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        SquareBracket,
    },
};
use std::fmt::Write;
use sway_ast::{assignable::ElementAccess, expr::ReassignmentOp, Assignable, Expr};
use sway_types::Spanned;

impl Format for ElementAccess {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            ElementAccess::Var(name) => {
                name.format(formatted_code, formatter)?;
            }
            ElementAccess::Index { target, arg } => {
                target.format(formatted_code, formatter)?;
                Expr::open_square_bracket(formatted_code, formatter)?;
                arg.get().format(formatted_code, formatter)?;
                Expr::close_square_bracket(formatted_code, formatter)?;
            }
            ElementAccess::FieldProjection {
                target,
                dot_token,
                name,
            } => {
                target.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", dot_token.span().as_str())?;
                name.format(formatted_code, formatter)?;
            }
            ElementAccess::TupleFieldProjection {
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

impl Format for Assignable {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Assignable::ElementAccess(element_access) => {
                element_access.format(formatted_code, formatter)?
            }
            Assignable::Deref { star_token, expr } => {
                write!(formatted_code, "{}", star_token.span().as_str())?;
                expr.format(formatted_code, formatter)?;
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

impl LeafSpans for ElementAccess {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        match self {
            ElementAccess::Var(var) => collected_spans.push(ByteSpan::from(var.span())),
            ElementAccess::Index { target, arg } => {
                collected_spans.append(&mut target.leaf_spans());
                collected_spans.append(&mut arg.leaf_spans());
            }
            ElementAccess::FieldProjection {
                target,
                dot_token,
                name,
            } => {
                collected_spans.append(&mut target.leaf_spans());
                collected_spans.push(ByteSpan::from(dot_token.span()));
                collected_spans.push(ByteSpan::from(name.span()));
            }
            ElementAccess::TupleFieldProjection {
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

impl LeafSpans for Assignable {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        match self {
            Assignable::ElementAccess(element_access) => element_access.leaf_spans(),
            Assignable::Deref { star_token, expr } => {
                let mut collected_spans = Vec::new();
                collected_spans.push(ByteSpan::from(star_token.span()));
                collected_spans.append(&mut expr.leaf_spans());
                collected_spans
            }
        }
    }
}
