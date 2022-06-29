use crate::{
    fmt::{Format, FormattedCode, Formatter},
    FormatterError,
};
use std::fmt::Write;
use sway_parse::{
    attribute::{Annotated, AttributeDecl},
    token::Delimiter,
    Parse,
};
use sway_types::Spanned;

use super::bracket::{Parenthesis, SquareBracket};

impl<T: Parse + Format> Format for Annotated<T> {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        // format each `Attribute`
        for attr in &self.attribute_list {
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
        line.push_str(self.hash_token.span().as_str());
        // `[`
        Self::open_square_bracket(line, formatter)?;
        let attr = self.attribute.clone().into_inner();
        // name e.g. `storage`
        line.push_str(attr.name.span().as_str());
        // `(`
        Self::open_parenthesis(line, formatter)?;
        // format and add args e.g. `read, write`
        if let Some(args) = attr.args {
            args.into_inner().format(line, formatter)?;
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
        line.push(Delimiter::Bracket.as_open_char());
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
        line.push(Delimiter::Parenthesis.as_open_char());
        Ok(())
    }
    fn close_parenthesis(
        line: &mut String,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        line.push(Delimiter::Parenthesis.as_close_char());
        Ok(())
    }
}
