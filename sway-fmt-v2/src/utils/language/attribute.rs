use crate::{
    formatter::{shape::LineStyle, *},
    utils::{
        map::byte_span::{ByteSpan, LeafSpans},
        {Parenthesis, SquareBracket},
    },
};
use std::fmt::Write;
use sway_ast::{
    attribute::{Annotated, Attribute, AttributeDecl},
    token::Delimiter,
};
use sway_types::Spanned;

impl<T: Format> Format for Annotated<T> {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // format each `Attribute`
        for attr in &self.attribute_list {
            attr.format(formatted_code, formatter)?;
            write!(
                formatted_code,
                "{}",
                &formatter.shape.indent.to_string(&formatter.config)?,
            )?;
        }
        // format `ItemKind`
        self.value.format(formatted_code, formatter)?;

        Ok(())
    }
}

impl Format for AttributeDecl {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        let prev_state = formatter.shape.code_line;
        formatter
            .shape
            .code_line
            .update_line_style(LineStyle::Normal);
        // `#`
        write!(formatted_code, "{}", self.hash_token.span().as_str())?;
        // `[`
        Self::open_square_bracket(formatted_code, formatter)?;
        let attr = self.attribute.get();
        // name e.g. `storage`
        write!(formatted_code, "{}", attr.name.span().as_str())?;
        // `(`
        Self::open_parenthesis(formatted_code, formatter)?;
        // format and add args e.g. `read, write`
        if let Some(args) = &attr.args {
            args.get().format(formatted_code, formatter)?;
        }
        // ')'
        Self::close_parenthesis(formatted_code, formatter)?;
        // `]\n`
        Self::close_square_bracket(formatted_code, formatter)?;
        formatter.shape.update_line_settings(prev_state);

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
        let mut collected_spans = vec![ByteSpan::from(self.hash_token.span())];
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
