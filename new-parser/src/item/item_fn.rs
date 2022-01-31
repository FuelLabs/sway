use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemFn {
    pub fn_signature: FnSignature,
    pub body: CodeBlock,
}

impl Spanned for ItemFn {
    fn span(&self) -> Span {
        Span::join(self.fn_signature.span(), self.body.span())
    }
}

pub fn item_fn() -> impl Parser<Output = ItemFn> + Clone {
    fn_signature()
    .then(lazy(|| code_block()))
    .map(|(fn_signature, body)| {
        ItemFn {
            fn_signature,
            body,
        }
    })
}

