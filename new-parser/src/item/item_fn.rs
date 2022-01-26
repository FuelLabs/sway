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

pub fn item_fn() -> impl Parser<Output = ItemFn> + Clone {
    fn_token()
    .then_whitespace()
    .then(ident())
    .then_optional_whitespace()
    .then(parens(padded(type_fields())))
    .then_optional_whitespace()
    .then(
        right_arrow_token()
        .then_optional_whitespace()
        .then(ty())
        .then_optional_whitespace()
        .optional()
    )
    .then(lazy(|| code_block()))
    .map(|((((fn_token, name), arguments), return_type_res), body): ((((_, _), _), Result<_, _>), _)| {
        ItemFn {
            fn_token,
            name,
            arguments,
            return_type_opt: return_type_res.ok(),
            body,
        }
    })
}

