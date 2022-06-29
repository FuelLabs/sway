use crate::fmt::{Format, FormattedCode, Formatter, FormatterError};
use std::fmt::Write;
use sway_parse::{
    brackets::{Parens, SquareBrackets},
    expr::Expr,
    keywords::{StrToken, UnderscoreToken},
    path::PathType,
    ty::{Ty, TyArrayDescriptor, TyTupleDescriptor},
};
use sway_types::Spanned;
impl Format for Ty {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        _formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Array(arr_descriptor) => format_array(formatted_code, arr_descriptor.clone()),
            Self::Infer { underscore_token } => format_infer(formatted_code, underscore_token),
            Self::Path(path_ty) => format_path(formatted_code, path_ty),
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
    write!(formatted_code, "{}", underscore_token.ident().as_str())?;
    Ok(())
}

/// Currently does not apply formatting, just pushes the str version of span
fn format_array(
    formatted_code: &mut FormattedCode,
    array_descriptor: SquareBrackets<TyArrayDescriptor>,
) -> Result<(), FormatterError> {
    write!(formatted_code, "{}", array_descriptor.span().as_str())?;
    Ok(())
}

/// Currently does not apply formatting, just pushes the str version of span
fn format_path(
    formatted_code: &mut FormattedCode,
    path_type: &PathType,
) -> Result<(), FormatterError> {
    write!(formatted_code, "{}", path_type.span().as_str())?;
    Ok(())
}

/// Currently does not apply formatting, just pushes the str version of span
fn format_str(
    formatted_code: &mut FormattedCode,
    str_token: StrToken,
    _length: SquareBrackets<Box<Expr>>,
) -> Result<(), FormatterError> {
    write!(formatted_code, "{}", str_token.span().as_str())?;
    Ok(())
}
/// Currently does not apply formatting, just pushes the str version of span
fn format_tuple(
    formatted_code: &mut FormattedCode,
    tuple_descriptor: Parens<TyTupleDescriptor>,
) -> Result<(), FormatterError> {
    write!(formatted_code, "{}", tuple_descriptor.span().as_str())?;
    Ok(())
}
