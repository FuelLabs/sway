use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct GenericParams {
    parameters: AngleBrackets<Punctuated<Ident, CommaToken>>,
}

impl Spanned for GenericParams {
    fn span(&self) -> Span {
        self.parameters.span()
    }
}

#[derive(Clone, Debug)]
pub struct GenericArgs {
    args: AngleBrackets<Punctuated<Ty, CommaToken>>,
}

impl Spanned for GenericArgs {
    fn span(&self) -> Span {
        self.args.span()
    }
}

pub struct ExpectedGenericParamsError {
    pub position: usize,
}

pub enum GenericParamsFatalError {
    UnclosedMultilineComment(UnclosedMultilineCommentError),
    ExpectedCloseAngleBracket { position: usize },
}

pub fn generic_params()
    -> impl Parser<Output = GenericParams, Error = ExpectedGenericParamsError, FatalError = GenericParamsFatalError> + Clone
{
    angle_brackets::<_, Infallible, _, _>(
        punctuated::<_, _, _, _, _, Infallible>(
            ident().map_err(|ExpectedIdentError { .. }| ()),
            comma_token().map_err(|ExpectedCommaTokenError { .. }| ()),
        )
        .then_optional_whitespace()
        .map_fatal_err(PaddedFatalError::flatten)
    )
    .map(|parameters| GenericParams { parameters })
    .map_err(|AngleBracketsError { position }| ExpectedGenericParamsError { position })
    .map_fatal_err(|error| match error {
        AngleBracketsFatalError::Inner(infallible) => infallible.unreachable(),
        AngleBracketsFatalError::InnerFatal(PaddedFatalError::Inner(infallible)) => infallible.unreachable(),
        AngleBracketsFatalError::InnerFatal(PaddedFatalError::UnclosedMultilineComment(error)) => {
            GenericParamsFatalError::UnclosedMultilineComment(error)
        },
        AngleBracketsFatalError::ExpectedCloseAngleBracket { position } => {
            GenericParamsFatalError::ExpectedCloseAngleBracket { position }
        },
    })
}

pub struct ExpectedGenericArgsError {
    pub position: usize,
}

pub enum GenericArgsFatalError {
    UnclosedMultilineComment(UnclosedMultilineCommentError),
    ExpectedCloseAngleBracket { position: usize },
}

pub fn generic_args()
    -> impl Parser<Output = GenericArgs, Error = ExpectedGenericArgsError, FatalError = GenericArgsFatalError> + Clone
{
    angle_brackets::<_, Infallible, _, _>(
        punctuated::<_, _, _, _, _, Infallible>(
            ty().map_err(|ExpectedTypeError { .. }| ()),
            comma_token().map_err(|ExpectedCommaTokenError { .. }| ()),
        )
        .then_optional_whitespace()
        .map_fatal_err(PaddedFatalError::flatten)
    )
    .map(|args| GenericArgs { args })
    .map_err(|AngleBracketsError { position }| ExpectedGenericArgsError { position })
    .map_fatal_err(|error| match error {
        AngleBracketsFatalError::Inner(infallible) => infallible.unreachable(),
        AngleBracketsFatalError::InnerFatal(PaddedFatalError::Inner(infallible)) => infallible.unreachable(),
        AngleBracketsFatalError::InnerFatal(PaddedFatalError::UnclosedMultilineComment(error)) => {
            GenericArgsFatalError::UnclosedMultilineComment(error)
        },
        AngleBracketsFatalError::ExpectedCloseAngleBracket { position } => {
            GenericArgsFatalError::ExpectedCloseAngleBracket { position }
        },
    })
}
