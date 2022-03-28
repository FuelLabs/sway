use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemFn {
    pub fn_signature: FnSignature,
    pub body: Braces<CodeBlockContents>,
}

impl ItemFn {
    pub fn span(&self) -> Span {
        Span::join(self.fn_signature.span(), self.body.span())
    }
}

impl Parse for ItemFn {
    fn parse(parser: &mut Parser) -> ParseResult<ItemFn> {
        let fn_signature = parser.parse()?;
        let body = parser.parse()?;
        Ok(ItemFn { fn_signature, body })
    }
}

