use crate::priv_prelude::*;

pub struct ItemFn {
    pub fn_token: FnToken,
    pub name: Ident,
    pub arguments: Parens<TypeFields>,
    pub return_type_opt: Option<(RightArrowToken, Ty)>,
    pub body: CodeBlock,
}

impl Spanned for ItemFn {
    fn span(&self) -> Span {
        Span::join(self.fn_token.span(), self.body.span())
    }
}

pub fn item_fn() -> impl Parser<char, ItemFn, Error = Cheap<char, Span>> + Clone {
    fn_token()
    .then_whitespace()
    .then(ident())
    .then_optional_whitespace()
    .then(parens(padded(type_fields())))
    .then(right_arrow_token().then(ty()).or_not())
    .then(code_block())
    .map(|((((fn_token, name), arguments), return_type_opt), body)| {
        ItemFn { fn_token, name, arguments, return_type_opt, body }
    })
}

