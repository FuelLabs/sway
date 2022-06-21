use crate::fmt::{Format, Formatter, FormattedCode};
use sway_parse::{
    attribute::{Annotated, AttributeDecl},
    Parse,
};
use sway_types::Spanned;

use super::bracket::{Parenthesis, SquareBracket};

impl<T: Parse> Format for Annotated<T> {
    fn format(&self, formatter: &mut Formatter) -> FormattedCode {
        let attributes = &self.attribute_list;
        let mut formatted_code = String::new();

        for attr in attributes {
            AttributeDecl::format(attr, &mut formatted_code, formatter);
        }

        formatted_code
    }
}

trait FormatDecl {
    fn format(&self, line: &mut String, formatter: &mut Formatter);
}

impl FormatDecl for AttributeDecl {
    fn format(&self, line: &mut String, formatter: &mut Formatter) {
        // At some point there will be enough attributes to warrant the need
        // of formatting the list according to `config::lists::ListTactic`.
        // For now the default implementation will be `Horizontal`.
        //
        // `#`
        line.push_str(self.hash_token.span().as_str());
        // `[`
        Self::open_square_bracket(line, formatter);
        let attr = self.attribute.clone().into_inner();
        // name e.g. `storage`
        line.push_str(attr.name.span().as_str());
        // `(`
        Self::open_parenthesis(line, formatter);
        // format and add args `read, write`
        if let Some(args) = attr.args {
            let mut args = args
                .into_inner()
                .value_separator_pairs
                .iter()
                .map(|arg| format!("{}{}", arg.0.as_str(), arg.1.span().as_str()))
                .collect::<Vec<String>>()
                .join(" ");
            args.pop(); // pop the ending space
            args.pop(); // pop the ending comma
            line.push_str(&args);
        }
        // ')'
        Self::close_parenthesis(line, formatter);
        // `]\n`
        Self::close_square_bracket(line, formatter);
    }
}

impl SquareBracket for AttributeDecl {
    fn open_square_bracket(line: &mut String, _formatter: &mut Formatter) {
        line.push('[');
    }
    fn close_square_bracket(line: &mut String, _formatter: &mut Formatter) {
        line.push_str("]\n");
    }
}

impl Parenthesis for AttributeDecl {
    fn open_parenthesis(line: &mut String, _formatter: &mut Formatter) {
        line.push('(')
    }
    fn close_parenthesis(line: &mut String, _formatter: &mut Formatter) {
        line.push(')')
    }
}
