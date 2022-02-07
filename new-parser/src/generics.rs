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

pub fn generic_params() -> impl Parser<Output = GenericParams> + Clone {
    angle_brackets(padded(punctuated(ident(), padded(comma_token()))))
    .map(|parameters| GenericParams { parameters })
}

pub fn generic_args() -> impl Parser<Output = GenericArgs> + Clone {
    angle_brackets(padded(punctuated(ty(), padded(comma_token()))))
    .map(|args| GenericArgs { args })
}
