use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemFn {
    pub fn_signature: FnSignature,
    pub body:         Braces<CodeBlockContents>,
}

impl Spanned for ItemFn {
    fn span(&self) -> Span {
        Span::join(self.fn_signature.span(), self.body.span())
    }
}
