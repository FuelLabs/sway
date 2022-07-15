use crate::{
    fmt::*,
    utils::comments::{CommentSpan, CommentVisitor},
};
use std::fmt::Write;
use sway_parse::{token::Delimiter, Pattern, PatternStructField};
use sway_types::Spanned;

use super::bracket::{CurlyBrace, Parenthesis};

impl Format for Pattern {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Wildcard { underscore_token } => {
                formatted_code.push_str(underscore_token.span().as_str())
            }
            Self::Var { mutable, name } => {
                if let Some(mut_token) = mutable {
                    write!(formatted_code, "{} ", mut_token.span().as_str())?;
                }
                // maybe add `Ident::format()`, not sure if needed yet.
                formatted_code.push_str(name.span().as_str());
            }
            Self::Literal(lit) => lit.format(formatted_code, formatter)?,
            Self::Constant(path) => path.format(formatted_code, formatter)?,
            Self::Constructor { path, args } => {
                path.format(formatted_code, formatter)?;
                Self::open_parenthesis(formatted_code, formatter)?;
                // need to add `<Pattern, CommaToken>` to `Punctuated::format()`
                args.clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                Self::close_parenthesis(formatted_code, formatter)?;
            }
            Self::Struct { path, fields } => {
                path.format(formatted_code, formatter)?;
                Self::open_curly_brace(formatted_code, formatter)?;
                // need to add `<PatternStructField, CommaToken>` to `Punctuated::format()`
                fields
                    .clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                Self::close_curly_brace(formatted_code, formatter)?;
            }
            Self::Tuple(args) => {
                Self::open_parenthesis(formatted_code, formatter)?;
                // need to add `<Pattern, CommaToken>` to `Punctuated::format()`
                args.clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                Self::close_parenthesis(formatted_code, formatter)?;
            }
        }
        Ok(())
    }
}

// Currently these just push their respective chars, we may need to change this
impl Parenthesis for Pattern {
    fn open_parenthesis(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Parenthesis.as_open_char())?;
        Ok(())
    }
    fn close_parenthesis(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Parenthesis.as_close_char())?;
        Ok(())
    }
}
impl CurlyBrace for Pattern {
    fn open_curly_brace(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Brace.as_open_char())?;
        Ok(())
    }
    fn close_curly_brace(
        line: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Brace.as_close_char())?;
        Ok(())
    }
}

impl Format for PatternStructField {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Rest { token } => {
                write!(formatted_code, "{}", token.span().as_str())?;
            }
            Self::Field {
                field_name,
                pattern_opt,
            } => {
                write!(formatted_code, "{}", field_name.span().as_str())?;
                if let Some(pattern) = pattern_opt {
                    write!(formatted_code, "{}", pattern.0.span().as_str())?;
                    pattern.1.format(formatted_code, formatter)?;
                }
            }
        }
        Ok(())
    }
}

impl CommentVisitor for Pattern {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        let mut collected_spans = Vec::new();
        match self {
            Pattern::Wildcard { underscore_token } => {
                collected_spans.push(CommentSpan::from_span(underscore_token.span()));
            }
            Pattern::Var { mutable, name } => {
                // Add mutable if it exists
                if let Some(mutable) = mutable {
                    collected_spans.push(CommentSpan::from_span(mutable.span()));
                }
                // Add name
                collected_spans.push(CommentSpan::from_span(name.span()));
            }
            Pattern::Literal(literal) => {
                collected_spans.append(&mut literal.collect_spans());
            }
            Pattern::Constant(constant) => {
                collected_spans.append(&mut constant.collect_spans());
            }
            Pattern::Constructor { path, args } => {
                collected_spans.append(&mut path.collect_spans());
                collected_spans.append(&mut args.collect_spans());
            }
            Pattern::Struct { path, fields } => {
                collected_spans.append(&mut path.collect_spans());
                collected_spans.append(&mut fields.collect_spans());
            }
            Pattern::Tuple(tuple) => {
                collected_spans.append(&mut tuple.collect_spans());
            }
        }
        collected_spans
    }
}

impl CommentVisitor for PatternStructField {
    fn collect_spans(&self) -> Vec<CommentSpan> {
        let mut collected_spans = Vec::new();
        match self {
            PatternStructField::Rest { token } => {
                collected_spans.push(CommentSpan::from_span(token.span()));
            }
            PatternStructField::Field {
                field_name,
                pattern_opt,
            } => {
                // Add field name CommentSpan
                collected_spans.push(CommentSpan::from_span(field_name.span()));
                // Add patern CommentSpan's if it exists
                if let Some(pattern) = pattern_opt {
                    // Add ColonToken's CommentSpan
                    collected_spans.push(CommentSpan::from_span(pattern.0.span()));
                    // Add patterns CommentSpan
                    collected_spans.append(&mut pattern.1.collect_spans());
                }
            }
        }
        collected_spans
    }
}
