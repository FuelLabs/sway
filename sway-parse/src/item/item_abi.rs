use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemAbi {
    pub abi_token: AbiToken,
    pub name: Ident,
    pub abi_items: Braces<Vec<(FnSignature, SemicolonToken)>>,
    pub abi_defs_opt: Option<Braces<Vec<ItemFn>>>,
}

impl ItemAbi {
    pub fn span(&self) -> Span {
        let start = self.abi_token.span();
        let end = match &self.abi_defs_opt {
            Some(abi_defs) => abi_defs.span(),
            None => self.abi_items.span(),
        };
        Span::join(start, end)
    }
}

impl Parse for ItemAbi {
    fn parse(parser: &mut Parser) -> ParseResult<ItemAbi> {
        let abi_token = parser.parse()?;
        let name = parser.parse()?;
        let abi_items = parser.parse()?;
        let abi_defs_opt = Braces::try_parse(parser)?;
        Ok(ItemAbi { abi_token, name, abi_items, abi_defs_opt })
    }
}

