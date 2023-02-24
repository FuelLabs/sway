use crate::{Parse, ParseResult, Parser};

use sway_ast::attribute::Annotated;
use sway_ast::keywords::{FnToken, OpenAngleBracketToken, WhereToken};
use sway_ast::{Braces, ItemImpl, ItemImplItem, PubToken, Ty};
use sway_error::parser_error::ParseErrorKind;

impl Parse for ItemImplItem {
    fn parse(parser: &mut Parser) -> ParseResult<ItemImplItem> {
        if parser.peek::<PubToken>().is_some() || parser.peek::<FnToken>().is_some() {
            let fn_decl = parser.parse()?;
            Ok(ItemImplItem::Fn(fn_decl))
        } else {
            Err(parser.emit_error(ParseErrorKind::ExpectedAnItem))
        }
    }
}

impl Parse for ItemImpl {
    fn parse(parser: &mut Parser) -> ParseResult<ItemImpl> {
        let impl_token = parser.parse()?;
        let generic_params_opt = parser.guarded_parse::<OpenAngleBracketToken, _>()?;
        let path_type = parser.parse()?;
        let (trait_opt, ty) = match parser.take() {
            Some(for_token) => (Some((path_type, for_token)), parser.parse()?),
            None => (None, Ty::Path(path_type)),
        };
        let where_clause_opt = parser.guarded_parse::<WhereToken, _>()?;
        let contents: Braces<Vec<Annotated<ItemImplItem>>> = parser.parse()?;
        if trait_opt.is_some() {
            for annotated in contents.get().iter() {
                #[allow(irrefutable_let_patterns)]
                if let ItemImplItem::Fn(item_fn) = &annotated.value {
                    parser.ban_visibility_qualifier(&item_fn.fn_signature.visibility)?;
                }
            }
        }
        Ok(ItemImpl {
            impl_token,
            generic_params_opt,
            trait_opt,
            ty,
            where_clause_opt,
            contents,
        })
    }
}
