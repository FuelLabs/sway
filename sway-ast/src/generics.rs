use crate::priv_prelude::*;

#[derive(Clone, Debug, Serialize)]
pub enum GenericParam {
    Trait {
        ident: Ident
    },
    Const {
        ident: Ident,
        ty: Ident,
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct GenericParams {
    pub parameters: AngleBrackets<Punctuated<GenericParam, CommaToken>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GenericArgs {
    pub parameters: AngleBrackets<Punctuated<Ty, CommaToken>>,
}

impl Spanned for GenericArgs {
    fn span(&self) -> Span {
        self.parameters.span()
    }
}
