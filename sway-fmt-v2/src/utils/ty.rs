use crate::fmt::{Format, FormattedCode, Formatter, FormatterError};
use std::fmt::Write;
use sway_parse::{
    brackets::{Parens, SquareBrackets},
    expr::Expr,
    keywords::{StrToken, UnderscoreToken},
    token::Delimiter,
    ty::{Ty, TyArrayDescriptor, TyTupleDescriptor},
};
use sway_types::Spanned;
impl Format for Ty {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Array(arr_descriptor) => {
                formatted_code.push(Delimiter::Bracket.as_open_char());
                arr_descriptor
                    .clone()
                    .into_inner()
                    .format(formatted_code, formatter)?;
                formatted_code.push(Delimiter::Bracket.as_close_char());
                Ok(())
            }
            Self::Infer { underscore_token } => format_infer(formatted_code, underscore_token),
            Self::Path(path_ty) => path_ty.format(formatted_code, formatter),
            Self::Str { str_token, length } => {
                format_str(formatted_code, str_token.clone(), length.clone())
            }
            Self::Tuple(tup_descriptor) => format_tuple(formatted_code, tup_descriptor.clone()),
        }
    }
}

/// Simply inserts a `_` token to the `formatted_code`.
fn format_infer(
    formatted_code: &mut FormattedCode,
    underscore_token: &UnderscoreToken,
) -> Result<(), FormatterError> {
    formatted_code.push_str(underscore_token.ident().as_str());
    Ok(())
}

impl Format for TyArrayDescriptor {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        self.ty.format(formatted_code, formatter)?;
        write!(
            formatted_code,
            "{} {}",
            self.semicolon_token.span().as_str(),
            self.length.span().as_str()
        )?;
        Ok(())
    }
}

fn format_str(
    formatted_code: &mut FormattedCode,
    str_token: StrToken,
    length: SquareBrackets<Box<Expr>>,
) -> Result<(), FormatterError> {
    write!(
        formatted_code,
        "{}{}{}{}",
        str_token.span().as_str(),
        Delimiter::Bracket.as_open_char(),
        length.into_inner().span().as_str(),
        Delimiter::Bracket.as_close_char()
    )?;
    Ok(())
}
/// Currently does not apply formatting, just pushes the str version of span
fn format_tuple(
    formatted_code: &mut FormattedCode,
    tuple_descriptor: Parens<TyTupleDescriptor>,
) -> Result<(), FormatterError> {
    formatted_code.push_str(tuple_descriptor.span().as_str());
    Ok(())
}
