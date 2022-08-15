use crate::{
    fmt::*,
    utils::{
        bracket::{CurlyBrace, Parenthesis},
        comments::{ByteSpan, LeafSpans},
    },
};
use std::fmt::Write;
use sway_ast::{token::Delimiter, Pattern, PatternStructField};
use sway_types::Spanned;

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
            Self::Var {
                reference,
                mutable,
                name,
            } => {
                if let Some(ref_token) = reference {
                    write!(formatted_code, "{} ", ref_token.span().as_str())?;
                }
                if let Some(mut_token) = mutable {
                    write!(formatted_code, "{} ", mut_token.span().as_str())?;
                }
                name.format(formatted_code, formatter)?;
            }
            Self::Literal(lit) => lit.format(formatted_code, formatter)?,
            Self::Constant(path) => path.format(formatted_code, formatter)?,
            Self::Constructor { path, args } => {
                path.format(formatted_code, formatter)?;
                Self::open_parenthesis(formatted_code, formatter)?;
                // need to add `<Pattern, CommaToken>` to `Punctuated::format()`
                args.get().format(formatted_code, formatter)?;
                Self::close_parenthesis(formatted_code, formatter)?;
            }
            Self::Struct { path, fields } => {
                path.format(formatted_code, formatter)?;
                Self::open_curly_brace(formatted_code, formatter)?;
                // need to add `<PatternStructField, CommaToken>` to `Punctuated::format()`
                fields.get().format(formatted_code, formatter)?;
                Self::close_curly_brace(formatted_code, formatter)?;
            }
            Self::Tuple(args) => {
                Self::open_parenthesis(formatted_code, formatter)?;
                // need to add `<Pattern, CommaToken>` to `Punctuated::format()`
                args.get().format(formatted_code, formatter)?;
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

impl LeafSpans for Pattern {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        match self {
            Pattern::Wildcard { underscore_token } => {
                collected_spans.push(ByteSpan::from(underscore_token.span()));
            }
            Pattern::Var {
                reference,
                mutable,
                name,
            } => {
                if let Some(reference) = reference {
                    collected_spans.push(ByteSpan::from(reference.span()));
                }
                if let Some(mutable) = mutable {
                    collected_spans.push(ByteSpan::from(mutable.span()));
                }
                collected_spans.push(ByteSpan::from(name.span()));
            }
            Pattern::Literal(literal) => {
                collected_spans.append(&mut literal.leaf_spans());
            }
            Pattern::Constant(constant) => {
                collected_spans.append(&mut constant.leaf_spans());
            }
            Pattern::Constructor { path, args } => {
                collected_spans.append(&mut path.leaf_spans());
                collected_spans.append(&mut args.leaf_spans());
            }
            Pattern::Struct { path, fields } => {
                collected_spans.append(&mut path.leaf_spans());
                collected_spans.append(&mut fields.leaf_spans());
            }
            Pattern::Tuple(tuple) => {
                collected_spans.append(&mut tuple.leaf_spans());
            }
        }
        collected_spans
    }
}

impl LeafSpans for PatternStructField {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        match self {
            PatternStructField::Rest { token } => {
                collected_spans.push(ByteSpan::from(token.span()));
            }
            PatternStructField::Field {
                field_name,
                pattern_opt,
            } => {
                collected_spans.push(ByteSpan::from(field_name.span()));
                if let Some(pattern) = pattern_opt {
                    collected_spans.push(ByteSpan::from(pattern.0.span()));
                    collected_spans.append(&mut pattern.1.leaf_spans());
                }
            }
        }
        collected_spans
    }
}
