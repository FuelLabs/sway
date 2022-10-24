use crate::{Parse, ParseBracket, ParseResult, Parser};

use sway_ast::{attribute::Annotated, Braces, FnSignature, ItemAbi, ItemFn};

impl Parse for ItemAbi {
    fn parse(parser: &mut Parser) -> ParseResult<ItemAbi> {
        let abi_token = parser.parse()?;
        let name = parser.parse()?;
        let abi_items: Braces<Vec<(Annotated<FnSignature>, _)>> = parser.parse()?;
        for (fn_signature, _) in abi_items.get().iter() {
            parser.ban_visibility_qualifier(&fn_signature.value.visibility)?;
        }
        let abi_defs_opt: Option<Braces<Vec<Annotated<ItemFn>>>> = Braces::try_parse(parser)?;
        if let Some(abi_defs) = &abi_defs_opt {
            for item_fn in abi_defs.get().iter() {
                parser.ban_visibility_qualifier(&item_fn.value.fn_signature.visibility)?;
            }
        }
        Ok(ItemAbi {
            abi_token,
            name,
            abi_items,
            abi_defs_opt,
        })
    }
}
