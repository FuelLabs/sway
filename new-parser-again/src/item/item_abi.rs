use crate::priv_prelude::*;

#[derive(Clone, Debug)]
pub struct ItemAbi {
    pub abi_token: AbiToken,
    pub name: Ident,
    pub abi_items: Braces<Vec<(FnSignature, SemicolonToken)>>,
    pub abi_defs_opt: Option<Braces<Vec<ItemFn>>>,
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

