use crate::{
    formatter::*,
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        {Parenthesis, SquareBracket},
    },
};
use std::fmt::Write;
use sway_ast::attribute::{Annotated, Attribute, AttributeArg, AttributeDecl, AttributeHashKind};
use sway_types::{
    ast::{Delimiter, PunctKind},
    constants::DOC_COMMENT_ATTRIBUTE_NAME,
    Spanned,
};

impl<T: Format + Spanned> Format for Annotated<T> {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // format each `Attribute`
        for attr in &self.attribute_list {
            formatter.write_indent_into_buffer(formatted_code)?;
            attr.format(formatted_code, formatter)?;
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
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        write!(formatted_code, "{}", self.name.span().as_str())?;
        if let Some(value) = &self.value {
            write!(formatted_code, " = {}", value.span().as_str())?;
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
            .partition(|a| a.name.as_str() == DOC_COMMENT_ATTRIBUTE_NAME);

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
        let hash_type_token_span = match &self.hash_kind {
            AttributeHashKind::Inner(_) => Err(FormatterError::HashBangAttributeError),
            AttributeHashKind::Outer(hash_token) => Ok(hash_token.span()),
        };
        write!(formatted_code, "{}", hash_type_token_span?.as_str())?;
        // `[`
        Self::open_square_bracket(formatted_code, formatter)?;
        let mut regular_attrs = regular_attrs.iter().peekable();
        while let Some(attr) = regular_attrs.next() {
            formatter.with_shape(
                formatter.shape.with_default_code_line(),
                |formatter| -> Result<(), FormatterError> {
                    // name e.g. `storage`
                    write!(formatted_code, "{}", attr.name.span().as_str())?;
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
                write!(formatted_code, "{} ", PunctKind::Comma.as_char())?;
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
