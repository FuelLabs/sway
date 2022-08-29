use crate::{
    formatter::{shape::LineStyle, *},
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        {Parenthesis, SquareBracket},
    },
};
use std::fmt::Write;
use sway_ast::{token::Delimiter, ExprArrayDescriptor, ExprTupleDescriptor};
use sway_types::Spanned;

impl Format for ExprTupleDescriptor {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Nil => {}
            Self::Cons {
                head,
                comma_token,
                tail,
            } => {
                Self::open_parenthesis(formatted_code, formatter)?;
                match formatter.shape.code_line.line_style {
                    LineStyle::Multiline => {
                        write!(
                            formatted_code,
                            "{}",
                            formatter.shape.indent.to_string(&formatter.config)?
                        )?;
                        head.format(formatted_code, formatter)?;
                        write!(formatted_code, "{}", comma_token.span().as_str())?;
                        tail.format(formatted_code, formatter)?;
                    }
                    _ => {
                        head.format(formatted_code, formatter)?;
                        write!(formatted_code, "{} ", comma_token.span().as_str())?;
                        tail.format(formatted_code, formatter)?;
                    }
                }
                Self::close_parenthesis(formatted_code, formatter)?;
            }
        }

        Ok(())
    }
}

impl Parenthesis for ExprTupleDescriptor {
    fn open_parenthesis(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match formatter.shape.code_line.line_style {
            LineStyle::Multiline => {
                formatter.shape.block_indent(&formatter.config);
                writeln!(line, "{}", Delimiter::Parenthesis.as_open_char())?;
            }
            _ => write!(line, "{}", Delimiter::Parenthesis.as_open_char())?,
        }

        Ok(())
    }
    fn close_parenthesis(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match formatter.shape.code_line.line_style {
            LineStyle::Multiline => {
                formatter.shape.block_unindent(&formatter.config);
                write!(
                    line,
                    "{}{}",
                    formatter.shape.indent.to_string(&formatter.config)?,
                    Delimiter::Parenthesis.as_close_char()
                )?;
            }
            _ => write!(line, "{}", Delimiter::Parenthesis.as_close_char())?,
        }

        Ok(())
    }
}

impl Format for ExprArrayDescriptor {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        Self::open_square_bracket(formatted_code, formatter)?;
        match self {
            Self::Sequence(punct_expr) => {
                punct_expr.format(formatted_code, formatter)?;
            }
            Self::Repeat {
                value,
                semicolon_token,
                length,
            } => {
                value.format(formatted_code, formatter)?;
                write!(formatted_code, "{} ", semicolon_token.span().as_str())?;
                length.format(formatted_code, formatter)?;
            }
        }
        Self::close_square_bracket(formatted_code, formatter)?;

        Ok(())
    }
}

impl SquareBracket for ExprArrayDescriptor {
    fn open_square_bracket(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match formatter.shape.code_line.line_style {
            LineStyle::Multiline => {
                formatter.shape.block_indent(&formatter.config);
                write!(line, "{}", Delimiter::Bracket.as_open_char())?;
            }
            _ => write!(line, "{}", Delimiter::Bracket.as_open_char())?,
        }

        Ok(())
    }
    fn close_square_bracket(
        line: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match formatter.shape.code_line.line_style {
            LineStyle::Multiline => {
                formatter.shape.block_unindent(&formatter.config);
                write!(
                    line,
                    "{}{}",
                    formatter.shape.indent.to_string(&formatter.config)?,
                    Delimiter::Bracket.as_close_char()
                )?;
            }
            _ => write!(line, "{}", Delimiter::Bracket.as_close_char())?,
        }

        Ok(())
    }
}

impl LeafSpans for ExprTupleDescriptor {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        if let ExprTupleDescriptor::Cons {
            head,
            comma_token,
            tail,
        } = self
        {
            collected_spans.append(&mut head.leaf_spans());
            collected_spans.push(ByteSpan::from(comma_token.span()));
            collected_spans.append(&mut tail.leaf_spans());
        }
        collected_spans
    }
}

impl LeafSpans for ExprArrayDescriptor {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        if let ExprArrayDescriptor::Repeat {
            value,
            semicolon_token,
            length,
        } = self
        {
            collected_spans.append(&mut value.leaf_spans());
            collected_spans.push(ByteSpan::from(semicolon_token.span()));
            collected_spans.append(&mut length.leaf_spans());
        }
        collected_spans
    }
}
