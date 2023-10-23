use crate::{Parse, ParseBracket, ParseResult, Parser};

use sway_ast::attribute::Annotated;
use sway_ast::keywords::{ConstToken, FnToken, OpenAngleBracketToken, TypeToken, WhereToken};
use sway_ast::{Braces, ItemFn, ItemTrait, ItemTraitItem, PubToken, Traits};
use sway_error::parser_error::ParseErrorKind;

impl Parse for ItemTraitItem {
    fn parse(parser: &mut Parser) -> ParseResult<ItemTraitItem> {
        if parser.peek::<PubToken>().is_some() || parser.peek::<FnToken>().is_some() {
            let fn_decl = parser.parse()?;
            let semicolon = parser.parse().ok();
            Ok(ItemTraitItem::Fn(fn_decl, semicolon))
        } else if let Some(_const_keyword) = parser.peek::<ConstToken>() {
            let const_decl = parser.parse()?;
            let semicolon = parser.parse().ok();
            Ok(ItemTraitItem::Const(const_decl, semicolon))
        } else if let Some(_type_keyword) = parser.peek::<TypeToken>() {
            let type_decl = parser.parse()?;
            let semicolon = parser.parse().ok();
            Ok(ItemTraitItem::Type(type_decl, semicolon))
        } else {
            Err(parser.emit_error(ParseErrorKind::ExpectedAnItem))
        }
    }

    fn error(
        spans: Box<[sway_types::Span]>,
        error: sway_error::handler::ErrorEmitted,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        Some(ItemTraitItem::Error(spans, error))
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

        let trait_items: Braces<Vec<Annotated<ItemTraitItem>>> = parser.parse()?;
        for item in trait_items.get().iter() {
            if let ItemTraitItem::Fn(fn_sig, _) = &item.value {
                parser.ban_visibility_qualifier(&fn_sig.visibility)?;
            }
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
