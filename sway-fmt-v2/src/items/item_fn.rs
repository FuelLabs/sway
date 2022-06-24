use crate::{
    fmt::{Format, FormattedCode, Formatter},
    utils::bracket::Parenthesis,
};
use sway_parse::{token::Delimiter, FnArg, FnArgs, FnSignature, ItemFn};
use sway_types::Spanned;

impl Format for ItemFn {
    fn format(&self, _formatter: &mut Formatter) -> FormattedCode {
        todo!()
    }
}

pub(crate) trait FormatSig {
    fn format(&self, line: &mut String, formatter: &mut Formatter);
}

impl FormatSig for FnSignature {
    fn format(&self, line: &mut String, formatter: &mut Formatter) {
        // `pub `
        if let Some(visibility_token) = &self.visibility {
            line.push_str(visibility_token.span().as_str());
            line.push(' ');
        }
        // `fn `
        line.push_str(self.fn_token.span().as_str());
        line.push(' ');
        // name
        line.push_str(self.name.as_str());
        // `<T>`
        if let Some(generics) = &self.generics.clone() {
            line.push_str(&generics.format(formatter))
        }
        // `(`
        Self::open_parenthesis(line, formatter);
        // FnArgs
        match self.arguments.clone().into_inner() {
            FnArgs::Static(args) => {
                let mut buf = args
                    .value_separator_pairs
                    .iter()
                    .map(|arg| format!("{}{}", arg.0.format(formatter), arg.1.span().as_str()))
                    .collect::<Vec<String>>()
                    .join(" ");
            }
            FnArgs::NonStatic { .. } => {}
        }
        // `)`
        Self::close_parenthesis(line, formatter);
    }
}

impl Parenthesis for FnSignature {
    fn open_parenthesis(line: &mut String, _formatter: &mut Formatter) {
        line.push(Delimiter::Parenthesis.as_open_char())
    }
    fn close_parenthesis(line: &mut String, _formatter: &mut Formatter) {
        line.push(Delimiter::Parenthesis.as_close_char())
    }
}

trait FormatFnArg {
    fn format(&self, formatter: &mut Formatter) -> String;
}

impl FormatFnArg for FnArg {
    fn format(&self, _formatter: &mut Formatter) -> String {
        let formatted_code = String::new();
        formatted_code
    }
}
