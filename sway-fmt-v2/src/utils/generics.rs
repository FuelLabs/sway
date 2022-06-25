use crate::{
    fmt::{Format, FormattedCode, Formatter},
    utils::bracket::AngleBracket,
};
use sway_parse::GenericParams;
use sway_types::Spanned;

// In the future we will need to determine whether the generic arguments
// are better suited as a `where` clause. At present they will be
// formatted in line.
//
impl Format for GenericParams {
    fn format(&self, formatter: &mut Formatter) -> FormattedCode {
        let mut formatted_code = String::new();
        let params = self.parameters.clone().into_inner().value_separator_pairs;
        Self::open_angle_bracket(self.clone(), &mut formatted_code, formatter);
        let mut buf = params
            .iter()
            .map(|param| format!("{}{}", param.0.as_str(), param.1.span().as_str()))
            .collect::<Vec<String>>()
            .join(" ");
        if params.len() == 1 {
            buf.pop(); // pop the ending comma
            formatted_code.push_str(&buf);
        } else {
            buf.pop(); // pop the ending space
            buf.pop(); // pop the ending comma
            formatted_code.push_str(&buf);
        }
        Self::close_angle_bracket(self.clone(), &mut formatted_code, formatter);
        formatted_code
    }
}

impl AngleBracket for GenericParams {
    fn open_angle_bracket(self, line: &mut String, _formatter: &mut Formatter) {
        line.push_str(self.parameters.open_angle_bracket_token.span().as_str())
    }
    fn close_angle_bracket(self, line: &mut String, _formatter: &mut Formatter) {
        line.push_str(self.parameters.close_angle_bracket_token.span().as_str())
    }
}
