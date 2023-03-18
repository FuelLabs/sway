use crate::{Parse, ParseBracket, ParseResult, Parser};

use sway_ast::attribute::Annotated;
use sway_ast::keywords::{FnToken, OpenAngleBracketToken, OpenCurlyBraceToken, WhereToken};
use sway_ast::{Braces, ItemFn, ItemTrait, ItemTraitItem, PubToken, Traits};
use sway_error::parser_error::ParseErrorKind;

impl Parse for ItemTraitItem {
    fn parse(parser: &mut Parser) -> ParseResult<ItemTraitItem> {
        if parser.peek::<PubToken>().is_some() || parser.peek::<FnToken>().is_some() {
            let fn_decl = parser.parse()?;
            Ok(ItemTraitItem::Fn(fn_decl))
        } else {
            Err(parser.emit_error(ParseErrorKind::ExpectedAnItem))
        }
    }
}

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

        let trait_items: Braces<Vec<(Annotated<ItemTraitItem>, _)>> = parser.parse()?;
        for (annotated, _) in trait_items.get().iter() {
            #[allow(irrefutable_let_patterns)]
            if let ItemTraitItem::Fn(fn_sig) = &annotated.value {
                parser.ban_visibility_qualifier(&fn_sig.visibility)?;
            }
        }

        let trait_defs_opt: Option<Braces<Vec<Annotated<ItemFn>>>> =
            parser.guarded_parse::<OpenCurlyBraceToken, _>()?;
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
