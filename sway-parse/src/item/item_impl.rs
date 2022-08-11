use crate::{Parse, ParseResult, Parser};

use sway_ast::attribute::Annotated;
use sway_ast::keywords::{OpenAngleBracketToken, WhereToken};
use sway_ast::{Braces, ItemFn, ItemImpl, Ty};

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
        let contents: Braces<Vec<Annotated<ItemFn>>> = parser.parse()?;
        if trait_opt.is_some() {
            for item_fn in contents.get().iter() {
                parser.ban_visibility_qualifier(&item_fn.value.fn_signature.visibility)?;
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
