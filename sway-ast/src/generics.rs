use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct GenericParams {
    pub parameters: AngleBrackets<Punctuated<Ident, CommaToken>>,
}

#[derive(Clone, Debug)]
pub struct GenericArgs {
    pub parameters: AngleBrackets<Punctuated<Ty, CommaToken>>,
}

impl Spanned for GenericArgs {
    fn span(&self) -> Span {
        self.parameters.span()
    }
}
