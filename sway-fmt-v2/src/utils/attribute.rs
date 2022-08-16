use crate::{
    fmt::*,
    utils::{
        bracket::{Parenthesis, SquareBracket},
        comments::{ByteSpan, LeafSpans},
    },
};
use std::fmt::Write;
use sway_ast::{
    attribute::{Annotated, Attribute, AttributeDecl},
    token::Delimiter,
};
use sway_parse::Parse;
use sway_types::Spanned;

impl<T: Parse + Format> Format for Annotated<T> {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // format each `Attribute`
        for attr in &self.attribute_list {
            write!(
                formatted_code,
                "{}",
                &formatter.shape.indent.to_string(&formatter.config)?,
            )?;
            attr.format(formatted_code, formatter)?;
        }
        // format `ItemKind`
        self.value.format(formatted_code, formatter)
    }
}

pub trait FormatDecl {
    fn format(&self, line: &mut String, formatter: &mut Formatter) -> Result<(), FormatterError>;
}

impl FormatDecl for AttributeDecl {
    fn format(&self, line: &mut String, formatter: &mut Formatter) -> Result<(), FormatterError> {
        // At some point there will be enough attributes to warrant the need
        // of formatting the list according to `config::lists::ListTactic`.
        // For now the default implementation will be `Horizontal`.
        //
        // `#`
        write!(line, "{}", self.hash_token.span().as_str())?;
        // `[`
        Self::open_square_bracket(line, formatter)?;
        let attr = self.attribute.get();
        // name e.g. `storage`
        write!(line, "{}", attr.name.span().as_str())?;
        // `(`
        Self::open_parenthesis(line, formatter)?;
        // format and add args e.g. `read, write`
        if let Some(args) = &attr.args {
            args.get().format(line, formatter)?;
        }
        // ')'
        Self::close_parenthesis(line, formatter)?;
        // `]\n`
        Self::close_square_bracket(line, formatter)?;
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
