use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub enum Pattern {
    Var {
        mutable: Option<MutToken>,
        name: Ident,
    },
}

impl Spanned for Pattern {
    fn span(&self) -> Span {
        match self {
            Pattern::Var { mutable, name } => {
                match mutable {
                    Some(mut_token) => Span::join(mut_token.span(), name.span()),
                    None => name.span(),
                }
            },
        }
    }
}

pub fn pattern() -> impl Parser<Output = Pattern> + Clone {
    mut_token()
    .then_whitespace()
    .optional()
    .then(ident())
    .map(|(mutable, name)| {
        Pattern::Var { mutable, name }
    })
}

