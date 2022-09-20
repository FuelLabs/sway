use crate::{Parse, ParseBracket, ParseResult, Parser};

use sway_ast::attribute::Annotated;
use sway_ast::keywords::{OpenAngleBracketToken, WhereToken};
use sway_ast::{Braces, FnSignature, ItemFn, ItemTrait, Traits};

impl Parse for ItemTrait {
    fn parse(parser: &mut Parser) -> ParseResult<ItemTrait> {
        let visibility = parser.take();
        let trait_token = parser.parse()?;
        let name = parser.parse()?;

        let generics = parser.guarded_parse::<OpenAngleBracketToken, _>()?;

        let super_traits = match parser.take() {
            Some(colon_token) => {
                let traits = parser.parse()?;
                Some((colon_token, traits))
            }
            None => None,
        };

        let where_clause_opt = parser.guarded_parse::<WhereToken, _>()?;

        let trait_items: Braces<Vec<(Annotated<FnSignature>, _)>> = parser.parse()?;
        for item in trait_items.get().iter() {
            let (fn_sig, _) = item;
            parser.ban_visibility_qualifier(&fn_sig.value.visibility)?;
        }

        let trait_defs_opt: Option<Braces<Vec<Annotated<ItemFn>>>> = Braces::try_parse(parser)?;
        if let Some(trait_defs) = &trait_defs_opt {
            for item in trait_defs.get().iter() {
                parser.ban_visibility_qualifier(&item.value.fn_signature.visibility)?;
            }
        }

        Ok(ItemTrait {
            visibility,
            trait_token,
            name,
            generics,
            where_clause_opt,
            super_traits,
            trait_items,
            trait_defs_opt,
        })
    }
}

impl Parse for Traits {
    fn parse(parser: &mut Parser) -> ParseResult<Traits> {
        let prefix = parser.parse()?;
        let mut suffixes = Vec::new();
        while let Some(add_token) = parser.take() {
            let suffix = parser.parse()?;
            suffixes.push((add_token, suffix));
        }
        let traits = Traits { prefix, suffixes };
        Ok(traits)
    }
}
