use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemImpl {
    pub impl_token: ImplToken,
    pub trait_opt: Option<(PathType, ForToken)>,
    pub ty: Ty,
    pub contents: Braces<Vec<ItemFn>>,
}

impl Spanned for ItemImpl {
    fn span(&self) -> Span {
        Span::join(self.impl_token.span(), self.contents.span())
    }
}

pub fn item_impl() -> impl Parser<Output = ItemImpl> + Clone {
    impl_token()
    .then_whitespace()
    .commit()
    .then(
        path_type()
        .then_whitespace()
        .then(for_token())
        .then_whitespace()
        .optional()
    )
    .then(ty())
    .then_optional_whitespace()
    .then(braces(
        optional_leading_whitespace(lazy(|| item_fn()))
        .repeated()
        .then_optional_whitespace()
    ))
    .map(|(((impl_token, trait_opt), ty), contents)| {
        ItemImpl { impl_token, trait_opt, ty, contents }
    })
}

