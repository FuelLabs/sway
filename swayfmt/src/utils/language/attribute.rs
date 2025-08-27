use crate::{
    comments::write_comments,
    constants::NEW_LINE,
    formatter::*,
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        {Parenthesis, SquareBracket},
    },
};
use std::fmt::Write;
use sway_ast::{
    attribute::{Annotated, Attribute, AttributeArg, AttributeDecl, AttributeHashKind},
    keywords::{HashBangToken, HashToken, Token},
    CommaToken,
};
use sway_types::{ast::Delimiter, Spanned};

impl<T: Format + Spanned + std::fmt::Debug> Format for Annotated<T> {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // format each `Attribute`
        let mut start = None;
        for attr in &self.attributes {
            if let Some(start) = start {
                // Write any comments that may have been defined in between the
                // attributes and the value
                write_comments(formatted_code, start..attr.span().start(), formatter)?;
                if !formatted_code.ends_with(NEW_LINE) {
                    write!(formatted_code, "{NEW_LINE}")?;
                }
            }
            formatter.write_indent_into_buffer(formatted_code)?;
            attr.format(formatted_code, formatter)?;
            start = Some(attr.span().end());
        }
        if let Some(start) = start {
            // Write any comments that may have been defined in between the
            // attributes and the value
            write_comments(formatted_code, start..self.value.span().start(), formatter)?;
            if !formatted_code.ends_with(NEW_LINE) {
                write!(formatted_code, "{NEW_LINE}")?;
            }
        }
        // format `ItemKind`
        formatter.write_indent_into_buffer(formatted_code)?;
        self.value.format(formatted_code, formatter)?;

        Ok(())
    }
}

impl Format for AttributeArg {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, "{}", self.name.as_str())?;
        if let Some(value) = &self.value {
            write!(formatted_code, " = ")?;
            value.format(formatted_code, formatter)?;
        }

        Ok(())
    }
}
impl LeafSpans for AttributeArg {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = Vec::new();
        collected_spans.push(ByteSpan::from(self.name.span()));
        if let Some(value) = &self.value {
            collected_spans.push(ByteSpan::from(value.span()));
        }
        collected_spans
    }
}

impl Format for AttributeDecl {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let (doc_comment_attrs, regular_attrs): (Vec<_>, _) = self
            .attribute
            .get()
            .into_iter()
            .partition(|a| a.is_doc_comment());

        // invariant: doc comment attributes are singleton lists
        if let Some(attr) = doc_comment_attrs.into_iter().next() {
            if let Some(Some(doc_comment)) = attr
                .args
                .as_ref()
                .map(|args| args.inner.final_value_opt.as_ref())
            {
                match self.hash_kind {
                    AttributeHashKind::Inner(_) => writeln!(
                        formatted_code,
                        "//!{}",
                        doc_comment.name.as_str().trim_end()
                    )?,
                    AttributeHashKind::Outer(_) => writeln!(
                        formatted_code,
                        "///{}",
                        doc_comment.name.as_str().trim_end()
                    )?,
                }
            }
            return Ok(());
        }

        // invariant: attribute lists cannot be empty
        // `#`
        match &self.hash_kind {
            AttributeHashKind::Inner(_hash_bang_token) => {
                write!(formatted_code, "{}", HashBangToken::AS_STR)?;
            }
            AttributeHashKind::Outer(_hash_token) => {
                write!(formatted_code, "{}", HashToken::AS_STR)?;
            }
        };
        // `[`
        Self::open_square_bracket(formatted_code, formatter)?;
        let mut regular_attrs = regular_attrs.iter().peekable();
        while let Some(attr) = regular_attrs.next() {
            formatter.with_shape(
                formatter.shape.with_default_code_line(),
                |formatter| -> Result<(), FormatterError> {
                    // name e.g. `storage`
                    write!(formatted_code, "{}", attr.name.as_str())?;
                    if let Some(args) = &attr.args {
                        // `(`
                        Self::open_parenthesis(formatted_code, formatter)?;
                        // format and add args e.g. `read, write`
                        args.get().format(formatted_code, formatter)?;
                        // ')'
                        Self::close_parenthesis(formatted_code, formatter)?;
                    };
                    Ok(())
                },
            )?;
            // do not put a separator after the last attribute
            if regular_attrs.peek().is_some() {
                write!(formatted_code, "{} ", CommaToken::AS_STR)?;
            }
        }
        // `]\n`
        Self::close_square_bracket(formatted_code, formatter)?;
        Ok(())
    }
}

impl SquareBracket for AttributeDecl {
    fn open_square_bracket(
        line: &mut String,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Bracket.as_open_char())?;
        Ok(())
    }
    fn close_square_bracket(
        line: &mut String,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        writeln!(line, "{}", Delimiter::Bracket.as_close_char())?;
        Ok(())
    }
}

impl Parenthesis for AttributeDecl {
    fn open_parenthesis(
        line: &mut String,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Parenthesis.as_open_char())?;
        Ok(())
    }
    fn close_parenthesis(
        line: &mut String,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(line, "{}", Delimiter::Parenthesis.as_close_char())?;
        Ok(())
    }
}
impl LeafSpans for AttributeDecl {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let hash_type_token_span = match &self.hash_kind {
            AttributeHashKind::Inner(hash_bang_token) => hash_bang_token.span(),
            AttributeHashKind::Outer(hash_token) => hash_token.span(),
        };
        let mut collected_spans = vec![ByteSpan::from(hash_type_token_span)];
        collected_spans.append(&mut self.attribute.leaf_spans());
        collected_spans
    }
}
impl LeafSpans for Attribute {
    fn leaf_spans(&self) -> Vec<ByteSpan> {
        let mut collected_spans = vec![ByteSpan::from(self.name.span())];
        if let Some(args) = &self.args {
            collected_spans.append(&mut args.leaf_spans());
        }
        collected_spans
    }
}
