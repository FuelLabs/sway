use crate::Formatter;
use sway_parse::attribute::AttributeDecl;
use sway_types::Spanned;

use super::bracket::SquareBracket;

trait Format {
    fn format(&self, line: &mut String, formatter: &mut Formatter);
}

pub fn format_attributes(attributes: Vec<AttributeDecl>, formatter: &mut Formatter) -> String {
    let mut formatted_code = String::new();
    for attr in &attributes {
        AttributeDecl::format(attr, &mut formatted_code, formatter);
    }

    formatted_code
}

impl Format for AttributeDecl {
    fn format(&self, line: &mut String, _formatter: &mut Formatter) {
        line.push_str(self.hash_token.span().as_str());
        Self::open_square_bracket(line, _formatter);
        // TODO: attributes are joined with their args during `span()`
        // but we may need to do formatting there eventually
        line.push_str(self.attribute.span().as_str());
        Self::close_square_bracket(line, _formatter);
    }
}

impl SquareBracket for AttributeDecl {
    fn open_square_bracket(line: &mut String, _formatter: &mut Formatter) {
        line.push('[');
    }
    fn close_square_bracket(line: &mut String, _formatter: &mut Formatter) {
        line.push_str("test]\n");
    }
}
