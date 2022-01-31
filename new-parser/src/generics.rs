use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct Generics {
    parameters: AngleBrackets<Punctuated<Ident, CommaToken>>,
}

impl Spanned for Generics {
    fn span(&self) -> Span {
        self.parameters.span()
    }
}

pub fn generics() -> impl Parser<Output = Generics> + Clone {
    angle_brackets(padded(punctuated(ident(), padded(comma_token()))))
    .map(|parameters| Generics { parameters })
}
