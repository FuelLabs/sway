use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemAbi {
    pub abi_token: AbiToken,
    pub name: Ident,
    pub abi_items: Braces<Vec<(Annotated<FnSignature>, SemicolonToken)>>,
    pub abi_defs_opt: Option<Braces<Vec<Annotated<ItemFn>>>>,
}

impl Spanned for ItemAbi {
    fn span(&self) -> Span {
        let start = self.abi_token.span();
        let end = match &self.abi_defs_opt {
            Some(abi_defs) => abi_defs.span(),
            None => self.abi_items.span(),
        };
        Span::join(start, end)
    }
}
