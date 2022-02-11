use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemTrait {
    pub visibility: Option<PubToken>,
    pub trait_token: TraitToken,
    pub name: Ident,
    pub trait_items: Braces<Vec<(FnSignature, SemicolonToken)>>,
}

impl Spanned for ItemTrait {
    fn span(&self) -> Span {
        match &self.visibility {
            Some(pub_token) => Span::join(pub_token.span(), self.trait_items.span()),
            None => Span::join(self.trait_token.span(), self.trait_items.span()),
        }
    }
}

pub fn item_trait() -> impl Parser<Output = ItemTrait> + Clone {
    pub_token()
    .then_whitespace()
    .optional()
    .then(trait_token())
    .then_whitespace()
    .commit()
    .then(ident())
    .then_optional_whitespace()
    .then(braces(optional_leading_whitespace(
        fn_signature()
        .then_optional_whitespace()
        .then(semicolon_token())
        .then_optional_whitespace()
        .repeated()
    )))
    .map(|(((visibility, trait_token), name), trait_items)| {
        ItemTrait { visibility, trait_token, name, trait_items }
    })
}

