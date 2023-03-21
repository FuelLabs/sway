use crate::{Parse, ParseBracket, ParseResult, Parser};

use sway_ast::attribute::Annotated;
use sway_ast::{Braces, ItemAbi, ItemFn, ItemTraitItem};

impl Parse for ItemAbi {
    fn parse(parser: &mut Parser) -> ParseResult<ItemAbi> {
        let abi_token = parser.parse()?;
        let name = parser.parse()?;
        let super_traits = match parser.take() {
            Some(colon_token) => {
                let traits = parser.parse()?;
                Some((colon_token, traits))
            }
            None => None,
        };
        let abi_items: Braces<Vec<(Annotated<ItemTraitItem>, _)>> = parser.parse()?;
        for (annotated, _) in abi_items.get().iter() {
            #[allow(irrefutable_let_patterns)]
            if let ItemTraitItem::Fn(fn_signature) = &annotated.value {
                parser.ban_visibility_qualifier(&fn_signature.visibility)?;
            }
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
            super_traits,
            abi_items,
            abi_defs_opt,
        })
    }
}
