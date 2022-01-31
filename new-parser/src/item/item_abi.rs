use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemAbi {
    pub abi_token: AbiToken,
    pub name: Ident,
    pub abi_items: Braces<Vec<(FnSignature, SemicolonToken)>>,
}

impl Spanned for ItemAbi {
    fn span(&self) -> Span {
        Span::join(self.abi_token.span(), self.abi_items.span())
    }
}

pub fn item_abi() -> impl Parser<Output = ItemAbi> + Clone {
    abi_token()
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
    .map(|((abi_token, name), abi_items)| {
        ItemAbi { abi_token, name, abi_items }
    })
}


