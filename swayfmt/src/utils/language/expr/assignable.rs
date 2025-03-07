use crate::{
    formatter::*,
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        SquareBracket,
    },
};
use std::fmt::Write;
use sway_ast::{
    assignable::ElementAccess,
    expr::ReassignmentOp,
    keywords::{DotToken, StarToken, Token},
    Assignable, Expr,
};
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
                dot_token: _,
                name,
            } => {
                target.format(formatted_code, formatter)?;
                write!(formatted_code, "{}", DotToken::AS_STR)?;
                name.format(formatted_code, formatter)?;
            }
            ElementAccess::TupleFieldProjection {
                target,
                dot_token: _,
                field,
                field_span: _,
            } => {
                target.format(formatted_code, formatter)?;
                write!(formatted_code, "{}{}", DotToken::AS_STR, field)?;
            }
            ElementAccess::Deref {
                target,
                star_token: _,
                is_root_element: root_element,
            } => {
                if *root_element {
                    write!(formatted_code, "(")?;
                }
                write!(formatted_code, "{}", StarToken::AS_STR)?;
                target.format(formatted_code, formatter)?;
                if *root_element {
                    write!(formatted_code, ")")?;
                }
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
            Assignable::Deref {
                star_token: _,
                expr,
            } => {
                write!(formatted_code, "{}", StarToken::AS_STR)?;
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
        write!(formatted_code, " {} ", self.variant.as_str())?;
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
            ElementAccess::Deref {
                target,
                star_token,
                is_root_element: _,
            } => {
                collected_spans.push(ByteSpan::from(star_token.span()));
                collected_spans.append(&mut target.leaf_spans());
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
