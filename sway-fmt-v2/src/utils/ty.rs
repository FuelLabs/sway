use crate::fmt::{Format, FormattedCode, Formatter, FormatterError};
use sway_parse::{
    brackets::SquareBrackets,
    expr::Expr,
    keywords::{StrToken, UnderscoreToken},
    path::PathType,
    ty::{Ty, TyArrayDescriptor},
};
impl Format for Ty {
    fn format(
        &self,
        formatted_code: &mut FormattedCode,
        formatter: &mut Formatter,
    ) -> Result<(), FormatterError> {
        match self {
            Self::Array(arr_descriptor) => {
                format_array(formatted_code, formatter, arr_descriptor.clone())
            }
            Self::Infer { underscore_token } => format_infer(formatted_code, underscore_token),
            Self::Path(path_ty) => format_path(formatted_code, formatter, path_ty),
            Self::Str { str_token, length } => {
                format_str(formatted_code, formatter, str_token.clone(), length.clone())
            }
            Self::Tuple(_tup_descriptor) => format_tuple(formatted_code, formatter),
        }
    }
}

fn format_infer(
    _formatted_code: &mut FormattedCode,
    _underscore_token: &UnderscoreToken,
) -> Result<(), FormatterError> {
    Ok(())
}

fn format_array(
    _formatted_code: &mut FormattedCode,
    _formatter: &mut Formatter,
    _underscore_token: SquareBrackets<TyArrayDescriptor>,
) -> Result<(), FormatterError> {
    todo!()
}

fn format_path(
    _formatted_code: &mut FormattedCode,
    _formatter: &mut Formatter,
    _path_type: &PathType,
) -> Result<(), FormatterError> {
    todo!()
}

fn format_str(
    _formatted_code: &mut FormattedCode,
    _formatter: &mut Formatter,
    _str_token: StrToken,
    _length: SquareBrackets<Box<Expr>>,
) -> Result<(), FormatterError> {
    todo!()
}

fn format_tuple(
    _formatted_code: &mut FormattedCode,
    _formatter: &mut Formatter,
) -> Result<(), FormatterError> {
    todo!()
}
