use crate::priv_prelude::*;

pub struct ItemTrait {
    pub trait_token: TraitToken,
    pub name: Ident,
    pub trait_items: Braces<Vec<(FnSignature, SemicolonToken)>>,
}

impl Spanned for ItemTrait {
    fn span(&self) -> Span {
        Span::join(self.trait_token.span(), self.trait_items.span())
    }
}

pub fn item_trait() -> impl Parser<Output = ItemTrait> + Clone {
    trait_token()
    .then_whitespace()
    .then(ident())
    .then_optional_whitespace()
    .then(braces(optional_leading_whitespace(
        fn_signature()
        .then_optional_whitespace()
        .then(semicolon_token())
        .then_optional_whitespace()
        .repeated()
    )))
    .map(|((trait_token, name), trait_items)| {
        ItemTrait { trait_token, name, trait_items }
    })
}

